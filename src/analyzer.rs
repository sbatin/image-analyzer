use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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

pub fn list_dir(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Err(_) = list_dir_rec(&mut files, dir) {
        tracing::error!("unable to list dir {:?}", dir);
    }
    files
}

pub struct AnalyzedData(Vec<(PathBuf, ImageHash)>);

pub struct Analyzer {
    hasher: Hasher,
    cache: HashMap<PathBuf, ImageHash>,
}

impl Analyzer {
    pub fn new() -> Self {
        let hasher = HasherConfig::new()
            .hash_size(16, 16)
            .hash_alg(HashAlg::DoubleGradient)
            .to_hasher();

        Analyzer {
            hasher,
            cache: HashMap::new(),
        }
    }

    pub fn analyze(&self, dir: &Path, tx: watch::Sender<usize>) -> Result<AnalyzedData> {
        let mut result = Vec::new();
        let files = list_dir(dir);
        let n = files.len();

        for (i, path) in files.into_iter().enumerate() {
            tracing::info!(path = path.to_str(), "analyzing");

            if let Some(hash) = self.cache.get(&path) {
                result.push((path, hash.clone()));
            } else if let Ok(image) = image::open(&path) {
                let hash = self.hasher.hash_image(&image);
                result.push((path, hash));
            }

            let progress = (i + 1) * 100 / n;
            tx.send(progress)?;
        }

        Ok(AnalyzedData(result))
    }

    pub fn update_cache(&mut self, data: &AnalyzedData) {
        for (path, hash) in &data.0 {
            if !self.cache.contains_key(path) {
                self.cache.insert(path.clone(), hash.clone());
            }
        }
    }
}

pub type Groups = Vec<Vec<PathBuf>>;

pub fn create_groups(hashes: &AnalyzedData, max_dist: u32) -> Groups {
    let xs = &hashes.0;
    let mut ds = disjoint_set::DisjointSet::new();

    for (k, _) in xs {
        ds.insert(k.to_path_buf());
    }

    for (k1, h1) in xs {
        for (k2, h2) in xs {
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