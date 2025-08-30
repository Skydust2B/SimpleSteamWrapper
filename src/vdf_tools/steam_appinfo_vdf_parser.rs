use std::io::{self, Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::vdf_tools::binary_vdf_parser::{read_magic, BinaryVdfValue};

#[derive(Debug)]
pub struct AppInfoSection {
    pub app_id: u32,
    pub size: u32,
    pub info_state: u32,
    pub last_updated: u32,
    pub access_token: u64,
    pub sha1: [u8; 20],
    pub change_number: u32,
    pub vdf: BinaryVdfValue
}

#[derive(Debug, Default)]
pub struct AppInfoFile {
    pub version: u32,
    pub string_table: Option<Vec<String>>,
    pub apps: Vec<AppInfoSection>,
}


fn hexdump(prefix: &str, buf: &[u8]) {
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

/// Parse the string table at the given offset in the file.
pub fn parse_string_table<R: Read + Seek>(reader: &mut R, offset: u32) -> io::Result<Vec<String>> {
    // Seek to the string table
    reader.seek(SeekFrom::Start(offset as u64))?;

    // First 4 bytes: number of strings in the table (u32)
    let num_strings = reader.read_u32::<LittleEndian>()?;

    let mut strings = Vec::with_capacity(num_strings as usize);

    for _ in 0..num_strings {
        let mut buf = Vec::new();
        loop {
            let mut byte = [0u8; 1];
            match reader.read_exact(&mut byte) {
                Ok(_) => {
                    if byte[0] == 0 {
                        // Null-terminated
                        break;
                    } else {
                        buf.push(byte[0]);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        // Convert UTF-8 bytes into a String
        let s = String::from_utf8_lossy(&buf).to_string();
        strings.push(s);
    }

    Ok(strings)
}

pub fn parse_appinfo<R: Read + Seek>(reader: &mut R) -> io::Result<AppInfoFile> {
    let magic = read_magic(reader, 0x075644)?;

    let mut file = AppInfoFile {
        version: magic.version as u32,
        string_table: None,
        apps: Vec::new(),
    };

    if magic.version >= 41 {
        let string_table_offset = reader.read_u32::<LittleEndian>()?;
        file.string_table = Some(parse_string_table(reader, string_table_offset)?);
    }

    loop {
        let app_id = match reader.read_u32::<LittleEndian>() {
            Ok(id) => id,
            Err(e) => { if e.kind() == io::ErrorKind::UnexpectedEof { break; } else { return Err(e); } }
        };
        if app_id == 0 { break; }

        let size = reader.read_u32::<LittleEndian>()?;
        if size < 36 {
            let mut skip_buf = vec![0u8; size as usize];
            let _ = reader.read_exact(&mut skip_buf);
            continue;
        }

        let mut section_buf = vec![0u8; size as usize];
        reader.read_exact(&mut section_buf)?;
        let mut cursor = Cursor::new(section_buf);

        let info_state = cursor.read_u32::<LittleEndian>()?;
        let last_updated = cursor.read_u32::<LittleEndian>()?;
        let access_token = cursor.read_u64::<LittleEndian>()?;
        let mut sha1 = [0u8; 20];
        cursor.read_exact(&mut sha1)?;
        let change_number = cursor.read_u32::<LittleEndian>()?;
        let binary_vdf_size = cursor.read_u32::<LittleEndian>()?;
        let remaining_bytes = cursor.get_ref().len().saturating_sub(cursor.position() as usize);
        let vdf_size = std::cmp::min(binary_vdf_size as usize, remaining_bytes);

        let mut vdf_buf = vec![0u8; vdf_size];
        cursor.read_exact(&mut vdf_buf)?;

        let vdf = BinaryVdfValue::parse(&mut Cursor::new(vdf_buf)).unwrap_or_else(|_| BinaryVdfValue::default());

        file.apps.push(AppInfoSection {
            app_id,
            info_state,
            last_updated,
            size,
            access_token,
            sha1,
            change_number,
            vdf,
        });
    }

    Ok(file)
}
