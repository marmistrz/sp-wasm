use std::collections::{BTreeMap, VecDeque};
use std::error::Error as StdError;
use std::fs;
use std::io::{self, Read, Write};
use std::path;

pub enum FSNode {
    File(Vec<u8>),
    Dir,
}

pub struct VirtualFS {
    pub mapping: BTreeMap<String, FSNode>,
}

impl VirtualFS {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn map_file<P: AsRef<path::Path>>(
        &mut self,
        abs_path: P,
        rel_path: P,
    ) -> io::Result<&FSNode> {
        let contents = read_file(&abs_path)?;
        self.mapping.insert(
            rel_path.as_ref().to_string_lossy().into(),
            FSNode::File(contents),
        );
        Ok(&self.mapping[rel_path.as_ref().to_str().unwrap()])
    }

    pub fn map_dir<P: AsRef<path::Path>>(&mut self, path: P) -> io::Result<&FSNode> {
        self.mapping
            .insert(path.as_ref().to_string_lossy().into(), FSNode::Dir);
        Ok(&self.mapping[path.as_ref().to_str().unwrap()])
    }

    pub fn map_path<P: AsRef<path::Path>>(
        &mut self,
        path: P,
        cb: &mut FnMut(&path::Path, &FSNode),
    ) -> Result<(), Box<dyn StdError>> {
        let mut rel_path = path::PathBuf::from("/");
        rel_path.push(path.as_ref().file_name().ok_or(error::RelativePathError)?);
        let abs_path = path::PathBuf::from(path.as_ref());

        let mut fifo = VecDeque::new();
        fifo.push_back((abs_path, rel_path));

        while let Some(path) = fifo.pop_front() {
            let (abs_path, rel_path) = path;
            log::debug!("abs_path = {:?}, rel_path = {:?}", abs_path, rel_path);

            if abs_path.is_dir() {
                cb(&rel_path, self.map_dir(&rel_path)?);
                log::debug!("mapped dir = {:?}", rel_path);

                for entry in fs::read_dir(abs_path)? {
                    let entry = entry?;
                    let abs_path = entry.path();

                    let mut rel_path = rel_path.clone();
                    rel_path.push(abs_path.file_name().ok_or(error::RelativePathError)?);

                    fifo.push_back((abs_path, rel_path));
                }
            } else {
                cb(&rel_path, self.map_file(&abs_path, &rel_path)?);
                log::debug!("mapped file {:?} => {:?}", abs_path, rel_path);
            }
        }
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Option<&[u8]> {
        self.mapping
            .get(&String::from(path))
            .and_then(|node| match node {
                FSNode::File(ref contents) => Some(contents.as_slice()),
                FSNode::Dir => None,
            })
    }
}

impl Default for VirtualFS {
    fn default() -> Self {
        Self {
            mapping: BTreeMap::new(),
        }
    }
}

pub fn read_file<P: AsRef<path::Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn write_file<P: AsRef<path::Path>>(path: P, contents: &[u8]) -> io::Result<()> {
    let mut file = fs::File::create(path.as_ref())?;
    file.write_all(contents)?;
    Ok(())
}

pub mod error {
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    pub struct RelativePathError;

    impl Error for RelativePathError {}

    impl fmt::Display for RelativePathError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "couldn't extract relative path")
        }
    }
}