use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::{error, fs};

use crate::archive::FileType;

fn u64_from_le_slice(slice: &[u8]) -> Result<u64, Box<dyn error::Error>> {
    Ok(u64::from_le_bytes(slice.try_into()?))
}
pub fn from_buffer(buffer: &[u8], extract_path: PathBuf) -> Result<(), Box<dyn error::Error>> {
    let file_offset = u64_from_le_slice(&buffer[0..8])?;
    let name_offset = u64_from_le_slice(&buffer[8..16])?;
    let content_offset = u64_from_le_slice(&buffer[16..24])?;
    let file_type = buffer[24];
    let name = &buffer[(name_offset as usize)..(content_offset as usize)];
    let contents = &buffer[(content_offset as usize)..(file_offset as usize)];
    let next_path = extract_path.join(OsStr::from_bytes(name));
    if file_type == FileType::File as u8 {
        fs::write(next_path, contents).map_err(Into::into)
    } else if file_type == FileType::Directory as u8 {
        fs::create_dir_all(&next_path)?;

        let entry_len = u64_from_le_slice(&contents[0..8])?;
        for i in 0..entry_len {
            let entry_start =
                u64_from_le_slice(&contents[(8 + i as usize * 8)..(16 + i as usize * 8)])? as usize;
            let entry_end =
                u64_from_le_slice(&contents[(16 + i as usize * 8)..(24 + i as usize * 8)])?
                    as usize;
            from_buffer(&contents[entry_start..entry_end], next_path.clone())?;
        }

        Ok(())
    } else {
        Err(format!("Unexpected file type {:x}!", file_type).into())
    }
}
