use std::{
    fs::{self, File},
    io,
    path::Path,
    thread,
};

use super::{copy_file, time_execution};

// WIP attempt to process files copying in parallel
// simple "benchmark tests" showcased that eventually when processing on single machine,
// the bottleneck is about disk IO, CPU processing, hense such parallelism is less beneficial
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

/// Non-recursive shallow flat list copying
pub fn copy_folder_sync(src_folder_path: &str, dest_folder_path: &str) -> io::Result<()> {
    let entries = fs::read_dir(src_folder_path)?;

    for entry in entries {
        if let Ok(entry) = entry {
            let file_name = entry.file_name();
            println!("Processing file sync: {:?}", &file_name);

            let file_name_str = file_name.to_str().unwrap(); // TODO: safe to use unwrap?

            let src_path = entry.path();
            let src_file = File::open(src_path)?;

            let dest_path = Path::new(dest_folder_path).join(file_name_str);

            let dest_file = File::create(dest_path)?;

            copy_file(src_file, dest_file)?;
        }
    }

    Ok(())
}

fn _process_single_sync() -> io::Result<()> {
    let src_file = File::open("./folders/from/text.txt")?;
    let dest_file = File::create("./folders/to/text.txt")?;
    copy_file(src_file, dest_file)?;

    Ok(())
}

pub fn synchronous() -> io::Result<()> {
    let time = time_execution!(copy_folder_sync("./folders/from", "./folders/to")?);

    dbg!(time);

    super::cleaunp()?;

    Ok(())
}
