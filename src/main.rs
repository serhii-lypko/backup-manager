use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};

/*
    - ✅ move single file from dir A to dir B
    - ✅ move all files from dir A to dir B (flat)
    - use multithreading for flat files? (try with heavy ones to measure performance)
    - traverse directory and move all data from A to B (recreate structure) ⭐️

    - testing
    - logging
    - progress feedback ⭐️

    - implement with async?
*/

fn copy_file(src_file: File, dest_file: File) -> io::Result<()> {
    let mut buf_reader = BufReader::new(src_file);
    let mut buf_writer = BufWriter::new(dest_file);

    let mut buffer = [0; 1024];

    while let Ok(bytes_read) = buf_reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        buf_writer.write_all(&buffer[..bytes_read])?;
    }

    /*
        BufWriter in Rust is designed to reduce the number of write operations by
        buffering data. When write data to a BufWriter, it doesn't immediately
        write the data to the file or stream. Instead, it stores the data in an
        in-memory buffer. When the buffer fills up, BufWriter automatically writes
        the buffered data to the underlying writer in a single operation.

        However, there are situations where the buffer might not be full, but still
        need to ensure that all data written so far is sent to the underlying writer.
        This is where flush() comes into play.
    */
    buf_writer.flush()?;

    Ok(())
}

fn copy_folder(src_folder: &str, dest_folder: &str) -> io::Result<()> {
    let entries = fs::read_dir(src_folder)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().unwrap(); // TODO: safe to use unwrap?

            let src_path = entry.path();
            let src_file = File::open(src_path)?;

            let mut dest_path = String::from(dest_folder);
            dest_path.push_str(file_name_str);

            let dest_file = File::create(dest_path)?;

            copy_file(src_file, dest_file)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    copy_folder("./folders/from", "./folders/to/")?;

    Ok(())
}
