use std::fs;
use std::io;

fn read_file_to_byte_array(file_path: &str) -> io::Result<Vec<u8>> {
    let contents = fs::read(file_path)?;
    Ok(contents)
}

fn main() {
    let file_data = read_file_to_byte_array("");
}
