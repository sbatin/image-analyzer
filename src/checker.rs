use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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