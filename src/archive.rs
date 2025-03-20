use std::{
    fs::{File, read_dir},
    io::{self, Read},
    path::PathBuf,
};

#[repr(u8)]
pub enum FileType {
    File,
    Directory,
}
pub fn to_buffer(
    file_path: PathBuf,
    buffer: &mut Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(&file_path)?;
    let metadata = file.metadata()?;

    let file_offset_index = buffer.len();
    let name_offset_index = file_offset_index + 8;
    let content_offset_index = file_offset_index + 16;
    buffer.extend_from_slice(&[0; 8 * 3]);

    if metadata.is_dir() {
        buffer.push(FileType::Directory as u8);
    } else if metadata.is_file() {
        buffer.push(FileType::File as u8);
    } else {
        panic!("Unsupported file type {:?}", metadata.file_type());
    }

    let name_offset = buffer.len() - file_offset_index;
    buffer[name_offset_index..content_offset_index]
        .copy_from_slice(&(name_offset as u64).to_le_bytes());
    buffer.extend_from_slice(
        file_path
            .file_name()
            .expect("Cannot read file path")
            .as_encoded_bytes(),
    );

    let content_offset = buffer.len() - file_offset_index;
    buffer[content_offset_index..(content_offset_index + 8)]
        .copy_from_slice(&(content_offset as u64).to_le_bytes());
    if metadata.is_file() {
        file.read_to_end(buffer)?;
    } else if metadata.is_dir() {
        let content_start_index = buffer.len();

        let entries =
            read_dir(&file_path)?.fold(Ok(vec![]) as io::Result<Vec<_>>, |vec, entry| {
                let mut vec = vec?;
                vec.push(entry?);
                Ok(vec)
            })?;

        let entry_len = entries.len();

        buffer.extend_from_slice(&(entry_len as u64).to_le_bytes());

        let entry_offset_indices = buffer.len();
        for _ in 0..=entry_len {
            buffer.extend_from_slice(&0_u64.to_le_bytes());
        }
        for (i, entry) in entries.iter().enumerate() {
            let entry_offset = buffer.len() - content_start_index;
            to_buffer(file_path.join(entry.path()), buffer)?;
            buffer[(entry_offset_indices + i * 8)..(entry_offset_indices + i * 8 + 8)]
                .copy_from_slice(&(entry_offset as u64).to_le_bytes());
            if i + 1 >= entry_len {
                let end_offset = buffer.len() - content_start_index;
                buffer[(entry_offset_indices + i * 8 + 8)..(entry_offset_indices + i * 8 + 16)]
                    .copy_from_slice(&(end_offset as u64).to_le_bytes());
            }
        }
    } else {
        unreachable!();
    }

    let file_offset = buffer.len() - file_offset_index;
    buffer[file_offset_index..(file_offset_index + 8)]
        .copy_from_slice(&(file_offset as u64).to_le_bytes());

    Ok(())
}
