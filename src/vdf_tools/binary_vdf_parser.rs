use std::collections::HashMap;
use std::{io};
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};
use log::info;

pub struct MagicHeader {
    pub ident: u32,
    pub version: u8,
}

/// Helps reading magic from valve's binary files
pub fn read_magic<R: Read>(reader: &mut R, expected_ident: u32) -> io::Result<MagicHeader> {
    let raw = reader.read_u32::<LittleEndian>()?;
    let version = (raw & 0xFF) as u8;
    let ident = raw >> 8;

    if ident != expected_ident {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
                                  format!("Invalid ident: expected {:06X}, got {:06X}", expected_ident, ident)));
    }

    Ok(MagicHeader { ident, version })
}

/// Binary VDF Value
#[derive(Debug,Clone)]
pub enum BinaryVdfValue {
    None(HashMap<String, BinaryVdfValue>), // nested document
    String(String),
    WString(String),
    Int(i32),
    Float(f32),
    Color([u8; 4]),
    UInt64(u64),
    Ptr(u32)
}

/// Safe reading helpers
fn safe_read_u8<R: Read>(reader: &mut R) -> Option<u8> {
    let mut buf = [0u8; 1];
    match reader.read(&mut buf) {
        Ok(1) => Some(buf[0]),
        _ => None,
    }
}

fn safe_read_i32<R: Read>(reader: &mut R) -> Option<i32> {
    let mut buf = [0u8; 4];
    match reader.read_exact(&mut buf) {
        Ok(_) => Some(i32::from_le_bytes(buf)),
        Err(_) => None,
    }
}

fn safe_read_f32<R: Read>(reader: &mut R) -> Option<f32> {
    let mut buf = [0u8; 4];
    match reader.read_exact(&mut buf) {
        Ok(_) => Some(f32::from_le_bytes(buf)),
        Err(_) => None,
    }
}

fn safe_read_u64<R: Read>(reader: &mut R) -> Option<u64> {
    let mut buf = [0u8; 8];
    match reader.read_exact(&mut buf) {
        Ok(_) => Some(u64::from_le_bytes(buf)),
        Err(_) => None,
    }
}

fn read_wstring<R: Read>(reader: &mut R) -> io::Result<String> {
    // Read number of characters as i16
    let length = reader.read_i16::<LittleEndian>()?;

    if length <= 0 {
        // Valve sometimes writes 0 here → just return empty
        return Ok(String::new());
    }

    // Clamp to a reasonable max (say 32k characters)
    let len = length as usize;
    if len > 32768 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unreasonable WString length {}", len)));
    }

    let mut chars = Vec::with_capacity(length as usize);

    for _ in 0..length {
        let codepoint = reader.read_u16::<LittleEndian>()?;
        chars.push(codepoint);
    }

    // Convert UCS-2 codepoints to Rust String
    Ok(String::from_utf16_lossy(&chars))
}

impl Default for BinaryVdfValue {
    fn default() -> Self {
        BinaryVdfValue::None(HashMap::new()) // wrap the HashMap in the `None` variant
    }
}

impl BinaryVdfValue {
    pub fn parse<R: Read>(reader: &mut R) -> io::Result<Self> {
        Self::parse_internal(reader, 0)
    }

    fn parse_internal<R: Read>(reader: &mut R, depth: usize) -> io::Result<Self> {
        if depth > 100 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Binary VDF: maximum recursion depth exceeded",
            ));
        }

        let mut map = HashMap::new();

        loop {
            let type_byte = match safe_read_u8(reader) {
                Some(b) => b,
                None => break, // EOF
            };

            if type_byte == 0x0B {
                break; // TYPE_NUMTYPES
            }

            // Read null-terminated name
            let mut name_bytes = Vec::new();
            loop {
                let b = match safe_read_u8(reader) {
                    Some(v) => v,
                    None => break, // EOF
                };
                if b == 0 { break; }
                name_bytes.push(b);
            }
            let name = String::from_utf8(name_bytes).unwrap_or_default();

            let value = match type_byte {
                0x00 => Self::parse_internal(reader, depth+1)?, // nested
                0x01 => {
                    // TYPE_STRING
                    let mut val_bytes = Vec::new();
                    loop {
                        let b = match safe_read_u8(reader) {
                            Some(v) => v,
                            None => break,
                        };
                        if b == 0 { break; }
                        val_bytes.push(b);
                    }
                    Self::String(String::from_utf8(val_bytes).unwrap_or_default())
                }
                0x02 => Self::Int(safe_read_i32(reader).unwrap_or(0)), // TYPE_INT
                0x03 => Self::Float(safe_read_f32(reader).unwrap_or(0.0)), // TYPE_FLOAT
                0x04 => Self::Ptr(reader.read_u32::<LittleEndian>()?), // TYPE_PTR
                0x05 => Self::WString(read_wstring(reader)?), // TYPE_WSTRING
                0x06 => { // TYPE_COLOR
                    let mut rgba = [0u8; 4];
                    if reader.read_exact(&mut rgba).is_ok() {
                        Self::Color(rgba)
                    } else {
                        Self::Color([0, 0, 0, 0])
                    }
                }
                0x07 => Self::UInt64(safe_read_u64(reader).unwrap_or(0)), // TYPE_UINT64
                0x08 => {
                    // TYPE_COMPILED_INT_BYTE
                    let byte_val = match safe_read_u8(reader) {
                        Some(b) => b as i8,
                        None => 0,
                    };
                    Self::Int(byte_val as i32)
                }
                0x09 => Self::Int(0), // TYPE_COMPILED_INT_0
                0x0A => Self::Int(1), // TYPE_COMPILED_INT_1
                b if b >= 0x0B => {
                    // Extended compiled integers (small int encoded as type byte)
                    let int_val = if b <= 0x7F { b as i32 } else { (b as i8) as i32 };
                    Self::Int(int_val)
                }
                _ => {
                    eprintln!("Warning: unsupported type {:02X} for key '{}'", type_byte, name);
                    continue;
                }
            };

            map.insert(name, value);
        }
        Ok(Self::None(map))
    }
}
