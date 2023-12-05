use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::thread;

use tokio::join;

/*
    - ✅ move single file from dir A to dir B
    - ✅ move all files from dir A to dir B (flat)
    - measure performance of current single threaded implementation
    - use multithreading for flat files? (try with heavy ones to measure performance)
    - traverse directory and move all data from A to B (recreate structure) ⭐️  ||  https://github.com/saschagrunert/indextree

    - CLI (clean, regular run, run with compression etc.)
    - testing with time performance
    - logging
    - progress feedback ⭐️
    - compression?

    - implement with async?
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

fn process_many_parallel() -> io::Result<()> {
    let dest_dir_paths = vec![
        "./folders/to/",
        "./folders/to_1/",
        "./folders/to_2/",
        "./folders/to_3/",
    ];

    let mut handles = vec![];

    for dest_dir_path in dest_dir_paths {
        let handle = thread::spawn(move || {
            copy_folder("./folders/large", dest_dir_path).unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}

fn process_many() -> io::Result<()> {
    let dest_dir_paths = vec![
        "./folders/to/",
        "./folders/to_1/",
        "./folders/to_2/",
        "./folders/to_3/",
    ];

    for dest_dir_path in dest_dir_paths {
        copy_folder("./folders/large", dest_dir_path)?;
    }

    Ok(())
}

// Single thread: 12.779892166s | 12.7 | 12.6
// Multy threaded: 7.990087375s | 8.431220375s | 8.7

fn main() -> io::Result<()> {
    time_execution!(process_many()?);
    // time_execution!(process_many_parallel()?);

    cleaunp()?;

    // channels_playground()?;
    // shared_memory_playground()?;

    Ok(())
}

fn cleaunp() -> io::Result<()> {
    let dir_paths = vec![
        Path::new("./folders/to"),
        Path::new("./folders/to_1"),
        Path::new("./folders/to_2"),
        Path::new("./folders/to_3"),
    ];

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

// ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ---- ----

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    fn setup_test_environment() -> (String, String) {
        let src_folder = "test_src";
        let dest_folder = "test_dest";

        // Create source and destination directories
        fs::create_dir_all(src_folder).unwrap();
        fs::create_dir_all(dest_folder).unwrap();

        // Create mock files in the source directory
        for i in 0..3 {
            let mut file = File::create(format!("{}/file{}.txt", src_folder, i)).unwrap();
            writeln!(file, "This is a test file.").unwrap();
        }

        (src_folder.to_string(), dest_folder.to_string())
    }

    fn teardown_test_environment(src_folder: &str, dest_folder: &str) {
        fs::remove_dir_all(src_folder).unwrap();
        fs::remove_dir_all(dest_folder).unwrap();
    }

    #[test]
    fn test_copy_folder() {
        let (src_folder, dest_folder) = setup_test_environment();

        time_execution!(copy_folder(&src_folder, &dest_folder).unwrap());

        todo!();

        teardown_test_environment(&src_folder, &dest_folder);
    }
}
