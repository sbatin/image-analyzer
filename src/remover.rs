use eyre::Result;
use serde::{Serialize, de::DeserializeOwned};
use std::{path::{PathBuf, Path}, fs};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct RemovedFile {
    id: String,
    path: PathBuf,
}

/// "removes" files by placing them into a designated directory
/// and remembering the original location.
/// Emulates OS recycled bin.
#[derive(Debug)]
pub struct Remover {
    root: PathBuf,
}

impl Remover {
    pub fn new<T>(root: T) -> Self
    where
        PathBuf: From<T>
    {
        Self { root: PathBuf::from(root) }
    }

    fn meta_path(&self, id: &str) -> PathBuf {
        self.root.join(id).with_extension("json")
    }

    fn data_path(&self, id: &str) -> PathBuf {
        self.root.join(id).with_extension("dat")
    }

    fn read_meta<T: DeserializeOwned>(&self, id: &str) -> Result<T> {
        let path = self.meta_path(id);
        let content = fs::read(path)?;
        let meta = serde_json::from_slice(&content)?;
        Ok(meta)
    }

    fn write_meta<T: Serialize + ?Sized>(&self, id: &str, meta: &T) -> Result<()> {
        let path = self.meta_path(id);
        let content = serde_json::to_string(meta)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn remove_meta(&self, id: &str) -> Result<()> {
        let path = self.meta_path(id);
        fs::remove_file(path)?;
        Ok(())
    }

    fn read_entry(&self, path: PathBuf) -> Option<RemovedFile> {
        let id = path.file_stem().and_then(|s| s.to_str())?;
        let ext = path.extension()?;
        if ext == "json" {
            // read the original file path
            let path = self.read_meta(id).ok()?;
            Some(RemovedFile {
                id: id.to_owned(),
                path,
            })
        } else {
            None
        }
    }

    pub fn resolve(&self, id: &str) -> Result<PathBuf> {
        // we could have some checks here to make sure file exists
        Ok(self.data_path(id))
    }

    pub fn remove(&self, path: &Path) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.write_meta(&id, path)?;

        // move the file
        let dest = self.data_path(&id);
        tracing::info!(src = path.to_str(), dest = dest.to_str(), "moving file");
        fs::rename(path, dest)?;
        Ok(id)
    }

    pub fn restore(&self, id: &str) -> Result<PathBuf> {
        let dest: PathBuf = self.read_meta(id)?;
        let src = self.data_path(id);
        tracing::info!(src = src.to_str(), dest = dest.to_str(), "moving file");
        fs::rename(src, &dest)?;
        self.remove_meta(id)?;
        Ok(dest)
    }

    pub fn list_removed(&self) -> Result<Vec<RemovedFile>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(file) = self.read_entry(path) {
                files.push(file);
            }
        }

        Ok(files)
    }

    pub fn restore_all(&self) -> Result<()> {
        let files = self.list_removed()?;
        for file in files {
            self.restore(&file.id)?;
        }

        Ok(())
    }
}
