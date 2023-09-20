use std::fs;
use std::path::{Path, PathBuf};
use image_hasher::{Hasher, ImageHash, HasherConfig, HashAlg};
use eyre::Result;
use tokio::sync::watch;

use crate::cache::Cache;
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

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, serde::Deserialize)]
pub enum HashType {
    AHash,
    PHash,
    DHash,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeRequest {
    pub dist: u32,
    pub path: PathBuf,
    pub hash_type: HashType,
    pub hash_size: u32,
}

type CacheKey = (HashType, u32, PathBuf);

pub struct Analyzer {
    cache: Cache<CacheKey, ImageHash>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self { cache: Cache::new() }
    }

    fn make_hasher(req: &AnalyzeRequest) -> Hasher {
        let (hash_alg, dct) = match req.hash_type {
            HashType::AHash => (HashAlg::Mean, false),
            HashType::PHash => (HashAlg::Mean, true),
            HashType::DHash => (HashAlg::Gradient, false),
        };

        let mut config = HasherConfig::new()
            .hash_size(req.hash_size, req.hash_size)
            .hash_alg(hash_alg);

        if dct {
            config = config.preproc_dct();
        }

        config.to_hasher()
    }

    fn compute_hashes(&self, req: &AnalyzeRequest, files: Vec<PathBuf>, tx: watch::Sender<usize>) -> Result<Hashes> {
        let hasher = Self::make_hasher(req);
        let mut result = Vec::new();
        let n = files.len();

        for (i, path) in files.into_iter().enumerate() {
            tracing::info!(path = path.to_str(), "analyzing");

            let key = (req.hash_type, req.hash_size, path.clone());
            if let Some(hash) = self.cache.get(key)? {
                result.push((path, hash));
            } else if let Ok(image) = image::open(&path) {
                let hash = hasher.hash_image(&image);
                result.push((path, hash));
            }

            let progress = (i + 1) * 100 / n;
            tx.send(progress)?;
        }

        Ok(result)
    }

    fn update_cache(&self, req: &AnalyzeRequest, hashes: Hashes) -> Result<()> {
        for (path, hash) in hashes {
            let key = (req.hash_type, req.hash_size, path);
            self.cache.set(key, hash)?;
        }

        Ok(())
    }

    pub fn analyze(&self, req: &AnalyzeRequest, tx: watch::Sender<usize>) -> Result<Groups> {
        let files = list_dir(&req.path)?;
        let hashes = self.compute_hashes(req, files, tx)?;
        let result = create_groups(&hashes, req.dist);
        self.update_cache(req, hashes)?;
        Ok(result)
    }
}