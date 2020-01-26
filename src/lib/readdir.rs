use std::io;
use std::fs;
use std::path::{Path, PathBuf};
use std::cmp::Ordering;

/// Read a directory's files, recursively
/// Files list comes in the provided fs::read_dir() order, which means there is no guarantee it is sorted in any way
/// Absolute paths to the files is returned as a vector
pub fn readdir_files_recursive<F: Fn(&PathBuf) -> bool>(dir: impl AsRef<Path>, filter: Option<&F>) -> Result<Vec<PathBuf>, io::Error> {
    let mut files = vec![];

    for entry in fs::read_dir(dir.as_ref())? {
        let path = entry?.path();

        if path.is_dir() {
            files.extend_from_slice(&readdir_files_recursive(&path, filter)?);
        }

        else if path.is_file() {
            if filter.map(|filter| filter(&path)).unwrap_or(true) {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Compare two paths using natural order
/// See the "natsort::natural_cmp()" function for more informations
pub fn natural_paths_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    let mut a = a.components();
    let mut b = b.components();

    loop {
        return match (a.next(), b.next()) {
            (Some(a_cp), Some(b_cp)) => match super::natural_cmp(&a_cp.as_os_str().to_string_lossy(), &b_cp.as_os_str().to_string_lossy()) {
                Ordering::Equal => continue,
                ordering @ _ => ordering
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}
