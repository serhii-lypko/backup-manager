use std::{fs, io, path::Path, sync::Arc};

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::log_time_execution;

// TODO: can be possible to introduce traits?

struct MsgLogBoundary(usize);

// ugly imperative shit
impl From<usize> for MsgLogBoundary {
    fn from(value: usize) -> Self {
        // 1GB
        if value > 1e10 as usize {
            MsgLogBoundary(1000)
        } else {
            MsgLogBoundary(100)
        }
    }
}

#[derive(Clone)]
pub struct CopyHandler;

#[derive(Debug)]
pub struct Msg {
    pub spawn_id: usize, // TODO: better name?
    pub progress: usize,
}

// TODO: is AtomicSender a good name?
type AtomicSender = Arc<Sender<Msg>>;

impl CopyHandler {
    pub fn new() -> Self {
        CopyHandler {}
    }

    pub async fn execute(self) -> io::Result<()> {
        use std::io::Write;

        // TODO: correct buffer size?
        let (sender, mut receiver): (Sender<Msg>, Receiver<Msg>) = mpsc::channel(1000);

        let sender = Arc::new(sender);

        tokio::spawn(async move {
            let mut messagess_received = 0;

            while let Some(msg) = receiver.recv().await {
                messagess_received += 1;

                // TODO: works great, but how to calculate progress for a bunch of files?
                print!("\rProgress: {}%", msg.progress);
                io::stdout().flush().unwrap();
            }

            dbg!(messagess_received);
        });

        log_time_execution!(
            // self.copy_folder_flat(sender, "./folders/from", "./folders/to")
            self.copy_folder_flat(sender, "./folders/tmp", "./folders/to")
                .await?
        );

        super::cleaunp()?;

        Ok(())
    }

    pub async fn copy_folder_flat(
        self,
        sender: AtomicSender,
        src_folder_path: &str,
        dest_folder_path: &str,
    ) -> io::Result<()> {
        let entries = fs::read_dir(src_folder_path)?;
        // let entrires_count = &entries.count();

        let mut handlers = vec![];

        for (index, entry) in entries.enumerate() {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                // println!("Processing file async: {:?}", &file_name);
                let file_name_str = file_name.to_str().unwrap(); // TODO: safe to use unwrap?

                let src_path = entry.path();
                let src_file = File::open(src_path).await?;

                let dest_path = Path::new(dest_folder_path).join(file_name_str);
                let dest_file = File::create(dest_path).await?;

                let sender = sender.clone();

                let handle = tokio::spawn(async move {
                    CopyHandler::copy_file(sender, src_file, dest_file, index)
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
        sender: AtomicSender,
        src_file: File,
        dest_file: File,
        index: usize,
    ) -> io::Result<()> {
        // TODO?
        // let sender = sender.clone();

        let metadata = src_file.metadata().await?;
        let file_size = metadata.len() as usize;

        let log_boundary: MsgLogBoundary = file_size.into();

        let mut buf_reader = BufReader::new(src_file);
        let mut buf_writer = BufWriter::new(dest_file);

        let mut buffer = [0; 8192];

        let mut total_read = 0;
        let mut message_counter = 0;

        while let Ok(bytes_read) = buf_reader.read(&mut buffer).await {
            total_read += bytes_read;

            if bytes_read == 0 {
                break;
            }

            // TODO: works great, but how to calculate progress for a bunch of files?
            let persentage = total_read * 100 / file_size;

            // skipping every N message
            if message_counter % log_boundary.0 == 0 {
                let msg = Msg {
                    progress: persentage,
                    spawn_id: index,
                };

                if let Err(_) = sender.send(msg).await {
                    println!("receiver dropped");
                    return Ok(());
                }
            }

            buf_writer.write_all(&buffer[..bytes_read]).await?;

            message_counter += 1;
        }

        buf_writer.flush().await?;

        Ok(())
    }
}
