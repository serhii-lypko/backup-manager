use std::{fs, io, path::Path};

mod copy_handler;
mod folder_tree;

use copy_handler::CopyHandler;

/*
    Version 0.1.0: Local backup manager

    - ✅ move single file from dir A to dir B
    - ✅ move all files from dir A to dir B (flat copying)
    - ✅ implement async copying flat folder
    - ✅ progress feedback ⭐️
    - async traverse nested data (recreate structure) ⭐️  ||  https://github.com/saschagrunert/indextree

    - tests for async traverse nested data
    - logging
    - compression?
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let copy_handler = CopyHandler::new();
    copy_handler.execute().await?;

    Ok(())
}

#[macro_export]
macro_rules! log_time_execution {
    ($x:expr) => {{
        let start = std::time::Instant::now();
        let result = $x;
        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
        result
    }};
}

pub fn cleaunp() -> io::Result<()> {
    // let dir_paths = vec![Path::new("./folders/to")];
    let dir_paths = vec![Path::new("./folders/tree_to")];

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

    // #[tokio::test]
    // async fn test_copy_folder_sync() {
    //     use sha2::{Digest, Sha256};
    //     use std::io::{BufReader, Read};

    //     let (src_folder_path, dest_folder_path) = setup_test_environment();

    //     let copy_handler = CopyHandler::new();

    //     let time_taken = time_execution!(copy_handler
    //         .copy_folder_flat(&src_folder_path, &dest_folder_path)
    //         .await
    //         .unwrap());

    //     dbg!(time_taken);

    //     let src_entries = fs::read_dir(src_folder_path).unwrap();
    //     let dest_entries = fs::read_dir(dest_folder_path).unwrap();

    //     for (src_entry, dest_entry) in src_entries.zip(dest_entries) {
    //         let src_entry = src_entry.unwrap();
    //         let src_filename = src_entry.file_name();
    //         let src_filesize = src_entry.metadata().unwrap().len();

    //         let dest_entry = dest_entry.unwrap();
    //         let dest_filename = dest_entry.file_name();
    //         let dest_filesize = dest_entry.metadata().unwrap().len();

    //         assert_eq!(src_filename, dest_filename);
    //         assert_eq!(src_filesize, dest_filesize);

    //         // comparing files data

    //         let mut src_buf = Vec::new();
    //         let mut src_buf_reader = BufReader::new(File::open(src_entry.path()).unwrap());
    //         src_buf_reader.read_to_end(&mut src_buf).unwrap();

    //         let mut src_hasher = Sha256::new();
    //         src_hasher.update(src_buf);
    //         let src_hash_result = src_hasher.finalize();

    //         let mut dest_buf = Vec::new();
    //         let mut dest_buf_reader = BufReader::new(File::open(dest_entry.path()).unwrap());
    //         dest_buf_reader.read_to_end(&mut dest_buf).unwrap();

    //         let mut dest_hasher = Sha256::new();
    //         dest_hasher.update(dest_buf);
    //         let dest_hash_result = dest_hasher.finalize();

    //         assert_eq!(src_hash_result, dest_hash_result);
    //     }

    //     teardown_test_environment(src_folder_path, dest_folder_path);
    // }
}
