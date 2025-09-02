// Binary VDF parser, mainly based on https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/tier1/kvpacker.cpp

use std::collections::HashMap;
use std::{io};
use std::io::{BufRead, Error, Read};
use byteorder::{LittleEndian, ReadBytesExt};

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
        return Err(Error::new(io::ErrorKind::InvalidData,
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

/// Equivalent to the C++ PACKTYPE_WSTRING case
fn read_wstring<R: Read>(reader: &mut R) -> io::Result<Option<String>> {
    let length = reader.read_i16::<LittleEndian>()?;

    if length < 0 {
        // C++: silently ignore → None
        return Ok(None);
    }

    let len = length as usize;

    if len == 0 {
        return Ok(Some(String::new()));
    }

    let mut raw_units = Vec::with_capacity(len);
    for _ in 0..len {
        let unit = reader.read_u16::<LittleEndian>()?;
        raw_units.push(unit);
    }

    Ok(Some(String::from_utf16_lossy(&raw_units)))
}

impl Default for BinaryVdfValue {
    fn default() -> Self {
        BinaryVdfValue::None(HashMap::new()) // wrap the HashMap in the `None` variant
    }
}

pub fn hex_dump<R: BufRead>(prefix: &str, reader: &mut R, len: usize) {
    let buffer = reader.fill_buf().unwrap_or(&[]);
    let dump_len = buffer.len().min(len);
    let buf = &buffer[..dump_len];

    let mut i = 0;
    while i < buf.len() {
        let line = &buf[i..buf.len().min(i + 16)];
        print!("{prefix}{:08x}: ", i);

        // hex part
        for b in line {
            print!("{:02x} ", b);
        }
        for _ in 0..(16 - line.len()) {
            print!("   ");
        }

        // ascii part
        print!(" |");
        for b in line {
            let c = *b as char;
            if c.is_ascii_graphic() || c == ' ' {
                print!("{}", c);
            } else {
                print!(".");
            }
        }
        println!("|");

        i += 16;
    }
}

fn read_null_terminated_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut bytes = Vec::new();

    loop {
        let b = match reader.read_u8() {
            Ok(v) => v,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // EOF
            Err(e) => return Err(e)
        };
        if b == 0 { break; }
        bytes.push(b);
    }
    let utf8 = match String::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => return Err(Error::new(io::ErrorKind::InvalidData, e))
    };
    Ok(utf8)
}

fn read_string_from_table<R: Read>(reader: &mut R, string_table: &Vec<String>) -> io::Result<String> {
    // Read 4 bytes for symbol/index (assuming 32-bit index in binary)
    let index = reader.read_u32::<LittleEndian>()?;
    // Look up string in table
    if let Some(name) = string_table.get(index as usize) {
        Ok(name.clone())
    } else {
        Ok(String::new()) // fallback if index invalid
    }
}

pub struct BinaryVdfParserOptions {
    pub string_table: Option<Vec<String>>
}

impl BinaryVdfValue {

    pub fn parse<R: Read>(reader: &mut R) -> io::Result<Self> {
        Self::parse_internal(reader, None, 0)
    }

    pub fn parse_with_opts<R: Read>(reader: &mut R, options: &BinaryVdfParserOptions) -> io::Result<Self> {
        Self::parse_internal(reader, Some(options), 0)
    }

    fn parse_internal<R: Read>(reader: &mut R, options: Option<&BinaryVdfParserOptions>, depth: usize) -> io::Result<Self> {
        if depth > 100 {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                "Binary VDF: maximum recursion depth exceeded",
            ));
        }

        let string_table = options.as_ref().and_then(|a| a.string_table.as_ref());
        let mut map = HashMap::new();

        loop {
            let type_byte = reader.read_u8()?;

            if type_byte == 0x08 {
                break; // TYPE_NUMTYPES
            }

            let name = {
                if string_table.is_some() {
                    read_string_from_table(reader, &string_table.unwrap())?
                } else {
                    read_null_terminated_string(reader)?
                }
            };

            let value = match type_byte {
                0x00 => Self::parse_internal(reader, options, depth + 1)?, // nested
                0x01 => Self::String(read_null_terminated_string(reader).unwrap_or_default()), // TYPE_STRING
                0x02 => Self::Int(reader.read_i32::<LittleEndian>().unwrap_or(0)), // TYPE_INT
                0x03 => Self::Float(reader.read_f32::<LittleEndian>().unwrap_or(0.0)), // TYPE_FLOAT
                0x04 => Self::Ptr(reader.read_u32::<LittleEndian>()?), // TYPE_PTR
                0x05 => match read_wstring(reader)? { // TYPE_WSTRING
                    Some(s) => Self::WString(s),
                    None => continue, // skip negative-length WString
                },
                0x06 => { // TYPE_COLOR
                    let mut rgba = [0u8; 4];
                    if reader.read_exact(&mut rgba).is_ok() {
                        Self::Color(rgba)
                    } else {
                        Self::Color([0, 0, 0, 0])
                    }
                }
                0x07 => Self::UInt64(reader.read_u64::<LittleEndian>().unwrap_or(0)), // TYPE_UINT64
                0x08 => {
                    // https://github.com/ValveSoftware/source-sdk-2013/blob/68c8b82fdcb41b8ad5abde9fe1f0654254217b8e/src/tier1/KeyValues.cpp#L2715
                    break;
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

/// Pretty-print a BinaryVdfValue with indentation
pub fn print_binary_vdf(map: &HashMap<String, BinaryVdfValue>, indent: usize) {
    let indent_str = "  ".repeat(indent);

    for (key, value) in map {
        match value {
            BinaryVdfValue::None(sub_map) => {
                println!("{}{}: {{", indent_str, key);
                print_binary_vdf(sub_map, indent + 1);
                println!("{}}}", indent_str);
            }
            BinaryVdfValue::String(s) => println!("{}{}: \"{}\"", indent_str, key, s),
            BinaryVdfValue::Int(i) => println!("{}{}: {}", indent_str, key, i),
            BinaryVdfValue::Float(f) => println!("{}{}: {}", indent_str, key, f),
            BinaryVdfValue::Color(rgba) => println!(
                "{}{}: rgba({}, {}, {}, {})",
                indent_str, key, rgba[0], rgba[1], rgba[2], rgba[3]
            ),
            BinaryVdfValue::UInt64(u) => println!("{}{}: {}", indent_str, key, u),
            BinaryVdfValue::WString(s) => println!("{}{}: \"{}\"", indent_str, key, s),
            BinaryVdfValue::Ptr(p) => println!("{}{}: \"{}\"", indent_str, key, p),
        }
    }
}
