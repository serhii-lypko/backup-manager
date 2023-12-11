use std::{fs, io, path::Path, sync::Arc};

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::time_execution;

#[derive(Clone)]
pub struct CopyHandler;

struct Msg {
    // id
    // progress
    // filename?
}

impl CopyHandler {
    pub fn new() -> Self {
        CopyHandler {}
    }

    pub async fn execute(self) -> io::Result<()> {
        let (sender, mut receiver): (Sender<usize>, Receiver<usize>) = mpsc::channel(1000);

        let sender = Arc::new(sender);

        tokio::spawn(async move {
            while let Some(foo) = receiver.recv().await {
                println!("Receiving = {}", foo);
            }
        });

        let time = time_execution!(
            self.copy_folder_flat(sender, "./folders/from", "./folders/to")
                .await?
        );
        dbg!(time);

        super::cleaunp()?;

        Ok(())
    }

    pub async fn copy_folder_flat(
        self,
        sender: Arc<Sender<usize>>,
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
                let src_file = File::open(src_path).await?;

                let dest_path = Path::new(dest_folder_path).join(file_name_str);
                let dest_file = File::create(dest_path).await?;

                let sender = sender.clone();

                let handle = tokio::spawn(async move {
                    CopyHandler::copy_file(sender, src_file, dest_file)
                        .await
                        .unwrap();
                });

                handlers.push(handle);
            }
        }

        for handle in handlers {
            handle.await?;
        }

        Ok(())
    }

    async fn copy_file(
        sender: Arc<Sender<usize>>,
        src_file: File,
        dest_file: File,
    ) -> io::Result<()> {
        // TODO: need to use extra clone?
        // let sender = sender.clone();

        let metadata = src_file.metadata().await?;
        let file_size = metadata.len() as usize;

        let mut buf_reader = BufReader::new(src_file);
        let mut buf_writer = BufWriter::new(dest_file);

        // let mut buffer = [0; 1024];
        let mut buffer = [0; 256];

        let mut total_read = 0;

        while let Ok(bytes_read) = buf_reader.read(&mut buffer).await {
            total_read += bytes_read;

            if bytes_read == 0 {
                break;
            }

            let bytes_left = file_size - total_read;

            // dbg!(bytes_left);

            if let Err(_) = sender.send(bytes_left).await {
                println!("receiver dropped");
                return Ok(());
            }

            buf_writer.write_all(&buffer[..bytes_read]).await?;
        }

        buf_writer.flush().await?;

        Ok(())
    }
}
