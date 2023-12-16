use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct FolderTree {
    root: Box<Node>,
}

impl FolderTree {
    pub fn new(src_folder_path: &str) -> Self {
        let mut root_node = Node::create_root(src_folder_path);
        root_node.index();

        dbg!(&root_node);

        FolderTree {
            root: Box::new(root_node),
        }
    }
}

#[derive(Debug)]
pub struct Node {
    entity_kind: EntityKind,
    children: Option<Vec<Box<Node>>>,
}

#[derive(Debug)]
pub enum EntityKind {
    Dir(Entity),
    File(Entity),
}

#[derive(Debug, Clone)]
pub struct Entity {
    path: String,
    name: String,
}

impl Node {
    pub fn create_root(src_folder_path: &str) -> Self {
        let root_dir_name = Path::new(src_folder_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let entity = Entity {
            path: src_folder_path.to_string(),
            name: root_dir_name,
        };

        Node {
            entity_kind: EntityKind::Dir(entity),
            children: Some(vec![]),
        }
    }

    pub fn new(is_dir: bool, entity: Entity) -> Self {
        Node {
            entity_kind: if is_dir {
                EntityKind::Dir(entity)
            } else {
                EntityKind::File(entity)
            },
            children: if is_dir { Some(vec![]) } else { None },
        }
    }

    // TODO: error handling
    pub fn index(&mut self) {
        if let EntityKind::Dir(entity) = &mut self.entity_kind {
            let entries = fs::read_dir(&entity.path).unwrap();

            for entry in entries.into_iter() {
                let dir_entry = entry.unwrap();

                let path = dir_entry.path();
                let path = path.to_string_lossy().to_string();

                let name = dir_entry.file_name().to_string_lossy().to_string();

                let entity = Entity { path, name };

                let file_type = dir_entry.file_type().unwrap();

                let mut new_node = Node::new(file_type.is_dir(), entity);

                if file_type.is_dir() {
                    new_node.index();
                }

                if let Some(children) = &mut self.children {
                    children.push(Box::new(new_node));
                }
            }
        }
    }
}
