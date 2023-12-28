use std::path::PathBuf;
use std::{fs, io, path::Path, sync::Arc};

use std::collections::HashMap;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::mpsc::{self, Receiver, Sender};

use async_recursion::async_recursion;

use crate::folder_tree::{FolderTree, FolderTreeNode, FsNodeKind};

struct MsgLogBoundary(usize);

// TODO: ugly imperative shit
impl From<usize> for MsgLogBoundary {
    fn from(value: usize) -> Self {
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
    pub id: usize,
    pub file_name: String,
    pub progress: usize,
}

type AtomicSender = Arc<Sender<Msg>>; // TODO: naming?

// TODO: implment also with arena index tree? (what is index tree?)

impl CopyHandler {
    pub fn new() -> Self {
        CopyHandler {}
    }

    pub async fn execute(self) -> io::Result<()> {
        use std::io::Write;

        let mut read_table: HashMap<String, usize> = HashMap::new();

        let (sender, mut receiver): (Sender<Msg>, Receiver<Msg>) = mpsc::channel(1000);

        let sender = Arc::new(sender);

        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                read_table.insert(msg.file_name, msg.progress);

                print!("\r");
                for (key, &value) in &read_table {
                    print!("{}: {:.2}% ", key, value);
                }

                io::stdout().flush().unwrap();
            }
        });

        let index = FolderTree::new("./folders/tree_from")?;
        let base_path = Path::new("./folders/tree_to/");

        CopyHandler::copy_folder_nested(*index.root, base_path, sender).await?;

        Ok(())
    }

    #[async_recursion]
    pub async fn copy_folder_nested(
        node: FolderTreeNode,
        base_path: &Path,
        sender: AtomicSender,
    ) -> io::Result<()> {
        let dir_path = Path::new(base_path).join(node.name);
        fs::create_dir(dir_path.clone())?;

        let mut spawn_handlers = vec![];

        if let Some(children) = node.children {
            for child in children {
                let sender = sender.clone();
                let child = *child;
                let dir_path = dir_path.clone();

                // TODO: fixing unwraps for handlers

                let handle = match child.kind {
                    FsNodeKind::Dir => tokio::spawn(async move {
                        CopyHandler::copy_folder_nested(child, dir_path.as_path(), sender)
                            .await
                            .unwrap();
                    }),
                    FsNodeKind::File => tokio::spawn(async move {
                        CopyHandler::copy_file(child, &dir_path, sender)
                            .await
                            .unwrap();
                    }),
                };

                spawn_handlers.push(handle);
            }
        };

        for spawn_handle in spawn_handlers {
            spawn_handle.await?;
        }

        Ok(())
    }

    pub async fn copy_folder_flat(
        node: FolderTreeNode,
        base_path: &Path,
        sender: AtomicSender,
    ) -> io::Result<()> {
        let dir_path = Path::new(base_path).join(node.name);
        fs::create_dir(dir_path.clone())?;

        let mut spawn_handlers = vec![];

        if let Some(children) = node.children {
            for child in children {
                let child = *child;
                let dir_path = dir_path.clone();
                let sender = sender.clone();

                let handle = tokio::spawn(async move {
                    CopyHandler::copy_file(child, &dir_path, sender)
                        .await
                        .unwrap();
                });

                spawn_handlers.push(handle);
            }
        }

        for spawn_handle in spawn_handlers {
            spawn_handle.await?;
        }

        Ok(())
    }

    async fn copy_file(
        node: FolderTreeNode,
        dir_path: &PathBuf,
        sender: AtomicSender,
    ) -> io::Result<()> {
        let src_file = File::open(&node.path).await?;
        let dest_path = Path::new(dir_path.as_path()).join(node.name);
        let dest_file = File::create(&dest_path).await?;

        let metadata = src_file.metadata().await?;
        let file_size = metadata.len() as usize;

        let log_boundary: MsgLogBoundary = file_size.into();

        let mut buf_reader = BufReader::new(src_file);
        let mut buf_writer = BufWriter::new(dest_file);

        let mut buffer = [0; 8192];

        let mut total_read = 0;
        let mut message_counter = 0;

        while let Ok(bytes_read) = buf_reader.read(&mut buffer).await {
            // TODO: unwrap & comlicated conversions?
            let filename = dest_path.file_name().unwrap().to_string_lossy().to_string();

            total_read += bytes_read;

            if bytes_read == 0 {
                break;
            }

            let persentage = total_read * 100 / file_size;

            // skipping every N message
            if message_counter % log_boundary.0 == 0 {
                let msg = Msg {
                    id: 0,
                    file_name: filename,
                    progress: persentage,
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
