use eyre::Result;
use image_hasher::{Hasher, ImageHash, HasherConfig, HashAlg};
use rayon::prelude::*;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;
use tokio::sync::watch;

use crate::cache::Cache;
use crate::disjoint_set;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct FileInfo {
    path: PathBuf,
    size: u64,
    date: u64,
}

impl FileInfo {
    pub fn from_entry(entry: DirEntry) -> Result<Self> {
        let metadata = entry.metadata()?;
        let size = metadata.len();
        let ctime = metadata.created()?;
        let ctime = ctime.duration_since(SystemTime::UNIX_EPOCH)?;
        Ok(Self {
            path: entry.path(),
            size,
            date: ctime.as_millis() as u64,
        })
    }
}

fn list_dir_rec(files: &mut Vec<FileInfo>, dir: &Path) -> Result<()> {
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
                    let info = FileInfo::from_entry(entry)?;
                    files.push(info);
                }
            }
        }
    }

    Ok(())
}

pub fn list_dir(dir: &Path) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    list_dir_rec(&mut files, dir)?;
    Ok(files)
}

type Hashes = Vec<(FileInfo, ImageHash)>;

pub type Groups = Vec<Vec<FileInfo>>;

fn create_groups(hashes: &Hashes, max_dist: u32) -> Groups {
    let mut ds = disjoint_set::DisjointSet::new();

    for (k, _) in hashes {
        ds.insert(k.clone());
    }

    for (k1, h1) in hashes {
        for (k2, h2) in hashes {
            if k1.path != k2.path && h1.dist(h2) <= max_dist {
                ds.union(k1, k2);
            }
        }
    }

    ds
        .to_vec()
        .into_iter()
        .filter(|v| v.len() > 1)
        .collect()
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

    fn compute_hashes(&self, req: &AnalyzeRequest, files: Vec<FileInfo>, tx: watch::Sender<usize>) -> Result<Hashes> {
        let hasher = Self::make_hasher(req);
        let total = files.len();

        //let iter = files.into_iter();
        let iter = files.into_par_iter();
        let counter = AtomicUsize::new(0);

        let result = iter.filter_map(|file| {
            tracing::info!(path = file.path.to_str(), "analyzing");

            let prev = counter.fetch_add(1, Ordering::Relaxed);
            let progress = prev * 100 / total;
            if let Err(_) = tx.send(progress) {
                tracing::error!(path = file.path.to_str(), "unable to report progress");
            }

            let key = (req.hash_type, req.hash_size, file.path.clone());
            if let Ok(Some(hash)) = self.cache.get(key) {
                Some((file, hash))
            } else if let Ok(image) = image::open(&file.path) {
                let hash = hasher.hash_image(&image);
                Some((file, hash))
            } else {
                None
            }
        }).collect();

        let progress = counter.into_inner() * 100 / total;
        tx.send(progress)?;

        Ok(result)
    }

    fn update_cache(&self, req: &AnalyzeRequest, hashes: Hashes) -> Result<()> {
        for (file, hash) in hashes {
            let key = (req.hash_type, req.hash_size, file.path);
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