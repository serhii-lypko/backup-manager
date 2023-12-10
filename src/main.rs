use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::thread;

use tokio::join;

/*
    - ✅ move single file from dir A to dir B
    - ✅ move all files from dir A to dir B (flat copying)
    - implement flat copying folder with async
    - async traverse nested data (recreate structure) ⭐️  ||  https://github.com/saschagrunert/indextree

    - CLI (clean, regular run, run with compression etc.)
    - testing with time performance
    - logging
    - progress feedback ⭐️
    - compression?
*/

macro_rules! time_execution {
    ($x:expr) => {{
        let start = std::time::Instant::now();
        let result = $x;
        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
        result
    }};
}

fn _process_many_parallel() -> io::Result<()> {
    let dest_dir_paths = vec![
        "./folders/to/",
        "./folders/to_1/",
        "./folders/to_2/",
        "./folders/to_3/",
    ];

    let mut handles = vec![];

    for dest_dir_path in dest_dir_paths {
        let handle = thread::spawn(move || {
            copy_folder_sync("./folders/large", dest_dir_path).unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn copy_file_sync(src_file: File, dest_file: File) -> io::Result<()> {
    use std::io::{BufReader, BufWriter, Read, Write};

    let mut buf_reader = BufReader::new(src_file);
    let mut buf_writer = BufWriter::new(dest_file);

    let mut buffer = [0; 1024]; // 8192

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

/// Non-recursive shallow flat list copying
fn copy_folder_sync(src_folder_path: &str, dest_folder_path: &str) -> io::Result<()> {
    let entries = fs::read_dir(src_folder_path)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().unwrap(); // TODO: safe to use unwrap?

            let src_path = entry.path();
            let src_file = File::open(src_path)?;

            let dest_path = Path::new(dest_folder_path).join(file_name_str);

            let dest_file = File::create(dest_path)?;

            copy_file_sync(src_file, dest_file)?;
        }
    }

    Ok(())
}

fn _process_single_sync() -> io::Result<()> {
    let src_file = File::open("./folders/from/text.txt")?;
    let dest_file = File::create("./folders/to/text.txt")?;
    copy_file_sync(src_file, dest_file)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let time = time_execution!(copy_folder_sync("./folders/from", "./folders/to")?);

    dbg!(time);

    cleaunp()?;

    Ok(())
}

fn cleaunp() -> io::Result<()> {
    let dir_paths = vec![Path::new("./folders/to")];

    for dir_path in dir_paths {
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;

    fn setup_test_environment() -> (&'static str, &'static str) {
        let src_folder = "test_src";
        let dest_folder = "test_dest";

        fs::create_dir_all(src_folder).unwrap();
        fs::create_dir_all(dest_folder).unwrap();

        for i in 0..3 {
            let mut file = File::create(format!("{}/file{}.txt", src_folder, i)).unwrap();
            writeln!(file, "This is a test file. {}", i * 100).unwrap();
        }

        (src_folder, dest_folder)
    }

    fn teardown_test_environment(src_folder_path: &str, dest_folder_path: &str) {
        fs::remove_dir_all(src_folder_path).unwrap();
        fs::remove_dir_all(dest_folder_path).unwrap();
    }

    #[test]
    fn test_copy_folder_sync() {
        use sha2::{Digest, Sha256};
        use std::io::{BufReader, Read};

        let (src_folder_path, dest_folder_path) = setup_test_environment();

        let time_taken =
            time_execution!(copy_folder_sync(&src_folder_path, &dest_folder_path).unwrap());

        dbg!(time_taken);

        let src_entries = fs::read_dir(src_folder_path).unwrap();
        let dest_entries = fs::read_dir(dest_folder_path).unwrap();

        for (src_entry, dest_entry) in src_entries.zip(dest_entries) {
            let src_entry = src_entry.unwrap();
            let src_filename = src_entry.file_name();
            let src_filesize = src_entry.metadata().unwrap().len();

            let dest_entry = dest_entry.unwrap();
            let dest_filename = dest_entry.file_name();
            let dest_filesize = dest_entry.metadata().unwrap().len();

            assert_eq!(src_filename, dest_filename);
            assert_eq!(src_filesize, dest_filesize);

            // comparing files data

            let mut src_buf = Vec::new();
            let mut src_buf_reader = BufReader::new(File::open(src_entry.path()).unwrap());
            src_buf_reader.read_to_end(&mut src_buf).unwrap();

            let mut src_hasher = Sha256::new();
            src_hasher.update(src_buf);
            let src_hash_result = src_hasher.finalize();

            let mut dest_buf = Vec::new();
            let mut dest_buf_reader = BufReader::new(File::open(dest_entry.path()).unwrap());
            dest_buf_reader.read_to_end(&mut dest_buf).unwrap();

            let mut dest_hasher = Sha256::new();
            dest_hasher.update(dest_buf);
            let dest_hash_result = dest_hasher.finalize();

            assert_eq!(src_hash_result, dest_hash_result);
        }

        teardown_test_environment(src_folder_path, dest_folder_path);
    }
}
