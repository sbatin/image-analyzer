use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs::DirEntry;
use image_hasher::{Hasher, ImageHash};
use eyre::Result;

fn list_dir_rec(files: &mut Vec<PathBuf>, dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Err(_) = list_dir_rec(files, &path) {
                tracing::error!("error reading folder contents {:?}", path);
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

type Hash = ImageHash;

fn make_hash(hasher: &Hasher, entry: &DirEntry) -> Option<Hash> {
    //sha256::try_digest(entry.path())
    let image = image::open(entry.path()).ok()?;
    Some(hasher.hash_image(&image))
}

type Files = HashMap<Hash, PathBuf>;

pub type Hashes = HashMap<PathBuf, Hash>;

fn analyze_files_rec(hasher: &Hasher, m: &mut Hashes, dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            analyze_files_rec(hasher, m, &path)?;
        } else {
            tracing::info!("analyzing {:?}", path);
            if let Some(hash) = make_hash(hasher, &entry) {
                m.insert(path, hash);
            }
        }
    }
    Ok(())
}

pub fn analyze_files(hasher: &Hasher, dir: &Path) -> Result<Hashes> {
    let mut m = HashMap::new();
    analyze_files_rec(hasher, &mut m, dir)?;
    Ok(m)
}

pub fn create_groups(hashes: &Hashes, max_dist: u32) -> Vec<Vec<PathBuf>> {
    let mut result = Vec::new();
    let mut ignore = HashSet::new();

    for (k1, h1) in hashes.iter() {
        if ignore.contains(k1) {
            continue;
        }

        let mut matches = Vec::new();
        for (k2, h2) in hashes.iter() {
            if !ignore.contains(k2) && k1 != k2 && h1.dist(h2) <= max_dist {
                matches.push(k2.to_path_buf());
                ignore.insert(k2);
            }
        }

        if !matches.is_empty() {
            matches.push(k1.to_path_buf());
            result.push(matches);
        }
    }

    result
}

/// returns true if all files in this dir are duplicates
pub fn check_dirs(hasher: &Hasher, root: &Path, visited: &mut Files, dir: &Path, remove: bool) -> Result<bool> {
    // all files in this dir are duplicates
    let mut all_dups = true;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            all_dups &= check_dirs(hasher, root, visited, &path, remove)?;
        } else {
            let path = entry.path().strip_prefix(root).unwrap().to_path_buf();
            if let Some(hash) = make_hash(hasher, &entry) {
                for other_hash in visited.keys() {
                    if hash.dist(other_hash) < 10 {
                        let other = visited.get(other_hash).unwrap();
                        println!("Duplicate found {:?} -> {:?}", path, other);
                    }
                }
                visited.insert(hash, path);
                /*if let Some(other) = visited.get(&hash) {
                    println!("Duplicate found {:?} -> {:?}", path, other);
                    if remove {
                        println!("removing {:?}", entry.path());
                        fs::remove_file(entry.path())?;
                    }
                } else {
                    all_dups &= false;
                    visited.insert(hash, path);
                }*/
            }
        }
    }

    if all_dups {
        println!("all files in {:?} are duplicates", dir);
    }

    Ok(all_dups)
}