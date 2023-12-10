use std::{
    fs::{self, File},
    io,
    path::Path,
};

use crate::time_execution;

#[derive(Clone)]
pub struct CopyHandler;

impl CopyHandler {
    pub fn new() -> Self {
        CopyHandler {}
    }

    pub async fn execute(self) -> io::Result<()> {
        let time = time_execution!(
            self.copy_folder_flat("./folders/from", "./folders/to")
                .await?
        );
        dbg!(time);

        super::cleaunp()?;

        Ok(())
    }

    pub async fn copy_folder_flat(
        self,
        src_folder_path: &str,
        dest_folder_path: &str,
    ) -> io::Result<()> {
        let entries = fs::read_dir(src_folder_path)?;

        let mut handlers = vec![];

        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                println!("Processing file async: {:?}", &file_name);

                let file_name_str = file_name.to_str().unwrap(); // TODO: safe to use unwrap?

                let src_path = entry.path();
                let src_file = File::open(src_path)?;

                let dest_path = Path::new(dest_folder_path).join(file_name_str);

                let dest_file = File::create(dest_path)?;

                let handle = tokio::spawn(async {
                    CopyHandler::copy_file(src_file, dest_file).unwrap();
                });

                handlers.push(handle);
            }
        }

        for handle in handlers {
            handle.await?;
        }

        Ok(())
    }

    fn copy_file(src_file: File, dest_file: File) -> io::Result<()> {
        use std::io::{BufReader, BufWriter, Read, Write};

        let mut buf_reader = BufReader::new(src_file);
        let mut buf_writer = BufWriter::new(dest_file);

        let mut buffer = [0; 1024];

        while let Ok(bytes_read) = buf_reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }

            buf_writer.write_all(&buffer[..bytes_read])?;
        }

        buf_writer.flush()?;

        Ok(())
    }
}
