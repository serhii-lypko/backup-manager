use std::fs::FileType;
use std::path::Path;
use std::{fs, io};

#[derive(Debug)]
pub struct FolderTree {
    pub root: Box<FolderTreeNode>,
}

impl FolderTree {
    pub fn new(src_folder_path: &str) -> io::Result<Self> {
        let mut root_node = FolderTreeNode::create_root(src_folder_path);
        root_node.build_index()?;

        Ok(FolderTree {
            root: Box::new(root_node),
        })
    }
}

type FolderChildren = Option<Vec<Box<FolderTreeNode>>>;

#[derive(Debug)]
pub struct FolderTreeNode {
    pub kind: FsNodeKind,
    pub relative_path: String, // relative?
    pub name: String,
    pub children: FolderChildren,
}

#[derive(Debug, Clone)]
pub enum FsNodeKind {
    Dir,
    File,
}

impl From<FileType> for FsNodeKind {
    fn from(value: FileType) -> Self {
        if value.is_dir() {
            Self::Dir
        } else {
            Self::File
        }
    }
}

impl FolderTreeNode {
    pub fn create_root(src_folder_path: &str) -> Self {
        let root_dir_name = Path::new(src_folder_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        FolderTreeNode {
            kind: FsNodeKind::Dir,
            relative_path: src_folder_path.to_string(),
            name: root_dir_name,
            children: Some(vec![]),
        }
    }

    pub fn new(kind: FsNodeKind, name: String, relative_path: String) -> Self {
        let children: FolderChildren = match &kind {
            FsNodeKind::Dir => Some(vec![]),
            FsNodeKind::File => None,
        };

        FolderTreeNode {
            kind,
            name,
            relative_path,
            children,
        }
    }

    pub fn build_index(&mut self) -> io::Result<()> {
        let entries = fs::read_dir(&self.relative_path)?;

        for child_entry in entries.into_iter() {
            let child_entry = child_entry?;

            let path = child_entry.path().to_string_lossy().to_string();
            let name = child_entry.file_name().to_string_lossy().to_string();
            let kind: FsNodeKind = child_entry.file_type()?.into();

            let mut new_node = FolderTreeNode::new(kind.clone(), name, path);

            if let FsNodeKind::Dir = kind {
                new_node.build_index()?;
            }

            if let Some(children) = &mut self.children {
                children.push(Box::new(new_node));
            }
        }

        Ok(())
    }
}
