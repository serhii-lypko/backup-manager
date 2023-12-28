use std::{env, fs, io, path::Path};

mod copy_handler;
mod folder_tree;

use copy_handler::CopyHandler;
use folder_tree::FsNodeKind;

/*
    Version 0.1.0: Local backup manager

    - ✅ move single file from dir A to dir B
    - ✅ move all files from dir A to dir B (flat copying)
    - ✅ implement async copying flat folder
    - ✅ progress feedback ⭐️
    - ✅ async traverse nested data (recreate structure) ⭐️  ||  https://github.com/saschagrunert/indextree

    - compression?
    - tests for async traverse nested data
    - logging
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(arg) => match arg.as_str() {
            "copy" => {
                let copy_handler = CopyHandler::new();
                log_time_execution!(copy_handler.execute().await?);
            }
            "cleanup" => {
                let cleanup_dir = args.get(2).unwrap();
                cleaunp(cleanup_dir)?;
            }
            _ => {
                println!("Unknown command");
            }
        },
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}

pub fn cleaunp(path: &String) -> io::Result<()> {
    println!("Performing cleanup...");
    cleanup_dir(Path::new(path), false)?;
    println!("Done");

    Ok(())
}

fn cleanup_dir(path: &Path, remove_dir: bool) -> io::Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;

        let kind: FsNodeKind = entry.file_type()?.into();
        let path = entry.path();

        match kind {
            FsNodeKind::Dir => {
                cleanup_dir(&path, true)?;
            }
            FsNodeKind::File => {
                fs::remove_file(path)?;
            }
        }
    }

    if remove_dir {
        fs::remove_dir(path)?;
    }

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
