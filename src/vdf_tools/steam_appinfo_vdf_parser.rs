use std::io::{self, Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::vdf_tools::binary_vdf_parser::{read_magic, BinaryVdfParserOptions, BinaryVdfValue};

#[derive(Debug)]
pub struct AppInfoSection {
    pub app_id: u32,
    pub size: u32,
    pub info_state: u32,
    pub last_updated: u32,
    pub pics_token: u64,
    pub appinfo_sha1: [u8; 20],
    pub vdfbin_sha1: [u8; 20],
    pub change_number: u32,
    pub vdf: BinaryVdfValue
}

#[derive(Debug, Default)]
pub struct AppInfoFile {
    pub version: u32,
    pub string_table: Option<Vec<String>>,
    pub apps: Vec<AppInfoSection>,
}

/// Parse the string table at the given offset in the file.
pub fn parse_string_table<R: Read + Seek>(reader: &mut R, offset: u64) -> io::Result<Vec<String>> {
    // Seek to the string table
    let original_pos = reader.stream_position()?;
    reader.seek(SeekFrom::Start(offset))?;

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
    reader.seek(SeekFrom::Start(original_pos))?;

    Ok(strings)
}

pub fn parse_appinfo<R: Read + Seek>(reader: &mut R) -> io::Result<AppInfoFile> {
    let magic = read_magic(reader, 0x075644)?;

    let mut file = AppInfoFile {
        version: magic.version as u32,
        string_table: None,
        apps: Vec::new(),
    };

    let _universe = reader.read_u32::<LittleEndian>()?;

    if magic.version >= 41 {
        let string_table_offset = reader.read_u64::<LittleEndian>()?;
        file.string_table = Some(parse_string_table(reader, string_table_offset)?);
    }
    let parser_opts = &BinaryVdfParserOptions {
        string_table: file.string_table.clone()
    };

    loop {
        let app_id = match reader.read_u32::<LittleEndian>() {
            Ok(id) => id,
            Err(e) => { if e.kind() == io::ErrorKind::UnexpectedEof { break; } else { return Err(e); } }
        };
        if app_id == 0 { break; }

        let size = reader.read_u32::<LittleEndian>()?;
        let mut section_buf = vec![0u8; size as usize];
        reader.read_exact(&mut section_buf)?;
        let mut cursor = Cursor::new(section_buf);

        let info_state = cursor.read_u32::<LittleEndian>()?;
        let last_updated = cursor.read_u32::<LittleEndian>()?;
        let pics_token = cursor.read_u64::<LittleEndian>()?;
        let mut appinfo_sha1 = [0u8; 20];
        cursor.read_exact(&mut appinfo_sha1)?;
        let change_number = cursor.read_u32::<LittleEndian>()?;

        let mut vdfbin_sha1 = [0u8; 20];
        cursor.read_exact(&mut vdfbin_sha1)?;
        //info!("appid: {:?}, size: {:?}, info_state: {:?}, last_update: {:?}, pics_token: {:?}, appinfo_sha1: {:?}, change_number: {:?}", app_id, size, info_state, last_updated, pics_token, appinfo_sha1, change_number);

        let mut vdf_buf = Vec::new();
        cursor.read_to_end(&mut vdf_buf)?;

        let vdf = BinaryVdfValue::parse_with_opts(&mut Cursor::new(vdf_buf), parser_opts).unwrap_or_else(|_| BinaryVdfValue::default());

        file.apps.push(AppInfoSection {
            app_id,
            info_state,
            last_updated,
            pics_token,
            appinfo_sha1,
            vdfbin_sha1,
            size,
            change_number,
            vdf,
        });
    }

    Ok(file)
}
