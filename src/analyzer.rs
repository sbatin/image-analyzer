use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use image_hasher::{Hasher, ImageHash, HasherConfig, HashAlg};
use eyre::Result;
use tokio::sync::watch;

use crate::disjoint_set;

fn list_dir_rec(files: &mut Vec<PathBuf>, dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Err(_) = list_dir_rec(files, &path) {
                tracing::error!("error reading folder content {:?}", path);
            }
        } else {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("jpg")
                    || ext.eq_ignore_ascii_case("jpeg")
                    || ext.eq_ignore_ascii_case("png")
                {
                    files.push(path);
                }
            }
        }
    }

    Ok(())
}

pub fn list_dir(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    list_dir_rec(&mut files, dir)?;
    Ok(files)
}

type Hashes = Vec<(PathBuf, ImageHash)>;

pub type Groups = Vec<Vec<PathBuf>>;

fn create_groups(hashes: &Hashes, max_dist: u32) -> Groups {
    let mut ds = disjoint_set::DisjointSet::new();

    for (k, _) in hashes {
        ds.insert(k.to_path_buf());
    }

    for (k1, h1) in hashes {
        for (k2, h2) in hashes {
            if k1 != k2 && h1.dist(h2) <= max_dist {
                ds.union(k1, k2);
            }
        }
    }

    let mut result: Vec<_> = ds
        .to_vec()
        .into_iter()
        .filter(|v| v.len() > 1)
        .collect();

    for v in &mut result {
        v.sort();
    }

    result.sort();
    result
}

pub struct Analyzer {
    hasher: Hasher,
    cache: RwLock<HashMap<PathBuf, ImageHash>>,
}

impl Analyzer {
    pub fn new() -> Self {
        let hasher = HasherConfig::new()
            .hash_size(16, 16)
            .hash_alg(HashAlg::DoubleGradient)
            .to_hasher();

        Analyzer {
            hasher,
            cache: RwLock::new(HashMap::new()),
        }
    }

    fn compute_hashes(&self, files: Vec<PathBuf>, tx: watch::Sender<usize>) -> Result<Hashes> {
        let mut result = Vec::new();
        let n = files.len();
        let cache = self.cache.read().unwrap();

        for (i, path) in files.into_iter().enumerate() {
            tracing::info!(path = path.to_str(), "analyzing");

            if let Some(hash) = cache.get(&path) {
                result.push((path, hash.clone()));
            } else if let Ok(image) = image::open(&path) {
                let hash = self.hasher.hash_image(&image);
                result.push((path, hash));
            }

            let progress = (i + 1) * 100 / n;
            tx.send(progress)?;
        }

        Ok(result)
    }

    fn update_cache(&self, hashes: Hashes) {
        let mut cache = self.cache.write().unwrap();
        for (path, hash) in hashes {
            if !cache.contains_key(&path) {
                cache.insert(path, hash);
            }
        }
    }

    pub fn analyze(&self, dir: &Path, max_dist: u32, tx: watch::Sender<usize>) -> Result<Groups> {
        let files = list_dir(dir)?;
        let hashes = self.compute_hashes(files, tx)?;
        let result = create_groups(&hashes, max_dist);
        self.update_cache(hashes);
        Ok(result)
    }
}