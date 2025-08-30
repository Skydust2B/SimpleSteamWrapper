use std::collections::HashMap;
use std::io::{self, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::vdf_tools::binary_vdf_parser::{read_magic, BinaryVdfValue};

/// Package Entry
#[derive(Debug)]
pub struct PackageEntry {
    pub package_id: u32,
    pub sha1_hash: Vec<u8>,
    pub change_number: u32,
    pub pics_token: Option<u64>,
    pub binary_vdf: BinaryVdfValue,
}

/// Package Info VDF
#[derive(Debug)]
pub struct PackageInfoVdf {
    pub universe: u32,
    pub packages: HashMap<u32, PackageEntry>,
}

pub fn parse_packageinfo<R: Read>(reader: &mut R) -> io::Result<PackageInfoVdf> {
    let magic = read_magic(reader, 0x065655)?;

    let universe = reader.read_u32::<LittleEndian>()?;
    let mut packages = HashMap::new();

    loop {
        let package_id = match reader.read_u32::<LittleEndian>() {
            Ok(id) => id,
            Err(_) => break, // EOF
        };

        if package_id == 0xFFFFFFFF {
            break; // end of file
        }

        let mut sha1_hash = [0u8; 20];
        reader.read_exact(&mut sha1_hash)?;

        let change_number = reader.read_u32::<LittleEndian>()?;

        let pics_token = if magic.version >= 40 {
            Some(reader.read_u64::<LittleEndian>()?)
        } else {
            None
        };

        // Valve parsing ignores binary_vdf_length, so parse until 0x0B
        let binary_vdf = match BinaryVdfValue::parse(reader) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "Failed to parse Binary VDF for package {}: {}",
                    package_id, e
                );
                BinaryVdfValue::default()
            }
        };

        packages.insert(package_id, PackageEntry {
            package_id,
            sha1_hash: sha1_hash.to_vec(),
            change_number,
            pics_token,
            binary_vdf,
        });
    }

    Ok(PackageInfoVdf {
        universe,
        packages,
    })
}
