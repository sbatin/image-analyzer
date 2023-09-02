use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use image_hasher::{Hasher, ImageHash, HasherConfig, HashAlg};
use eyre::Result;

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

pub fn make_engine() -> Hasher {
    HasherConfig::new()
        .hash_size(16, 16)
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher()
}

pub struct AnalyzedData(Vec<(PathBuf, ImageHash)>);

pub fn analyze_files(hasher: &Hasher, dir: &Path) -> Result<AnalyzedData> {
    let mut result = Vec::new();

    for path in list_dir(dir) {
        tracing::info!("analyzing {:?}", path);

        if let Ok(image) = image::open(&path) {
            let hash = hasher.hash_image(&image);
            result.push((path, hash));
        }
    }

    Ok(AnalyzedData(result))
}

pub fn create_groups(hashes: &AnalyzedData, max_dist: u32) -> Vec<Vec<PathBuf>> {
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

type Files = HashMap<String, PathBuf>;

/// returns true if all files in this dir are duplicates
pub fn check_dirs(visited: &mut Files, dir: &Path, remove: bool) -> Result<bool> {
    // all files in this dir are duplicates
    let mut all_dups = true;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            all_dups &= check_dirs(visited, &path, remove)?;
        } else {
            if let Ok(hash) = sha256::try_digest(entry.path()) {
                if let Some(other) = visited.get(&hash) {
                    println!("duplicate found {:?} -> {:?}", entry.path(), other);
                    if remove {
                        println!("removing {:?}", entry.path());
                        fs::remove_file(entry.path())?;
                    }
                } else {
                    all_dups &= false;
                    visited.insert(hash, entry.path());
                }
            }
        }
    }

    if all_dups {
        println!("all files in {:?} are duplicates", dir);
    }

    Ok(all_dups)
}