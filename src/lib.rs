use std::ops::{Add, Rem, Div};
use std::cmp::{PartialEq, Ordering};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::str::Chars;
use std::iter::Peekable;

/// Perform a ceiling division of the provided number by the divider
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(2 / 3, 2);
/// assert_eq!(ceil_div(2, 3), 3);
/// ```
pub fn ceil_div<D: Div<Output=O> + Rem<Output=O> + Copy, O: Add<Output=V> + PartialEq + From<u8>, V>(num: D, divider: D) -> V {
    num / divider + if num % divider != O::from(0) { O::from(1) } else { O::from(0) }
}

/// Check if a path has a common image format extension
/// Additional formats that may not be widely supported can be accepted using the `extended` parameter
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(has_image_ext(Path::new("file.png"), false), true);
/// assert_eq!(has_image_ext(Path::new("file.Jpeg"), false), true);
/// assert_eq!(has_image_ext(Path::new("file.bgp"), false), false);
///
/// // With extended image formats
/// assert_eq!(has_image_ext(Path::new("file.bgp"), true), false);
/// ```
pub fn has_image_ext(path: impl AsRef<Path>, extended: bool) -> bool {
    match path.as_ref().extension() {
        None => false,
        Some(ext) => match ext.to_str() {
            None => false,
            Some(ext) => match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" | "png" | "bmp" => true,

                "tif" | "tiff" | "gif" | "eps" | "raw" | "cr2" | "nef"  | "orf" | "sr2" |
                "ppm" | "webp" | "pgm" | "pbm" | "pnm" | "ico" | "flif" | "pam" | "pcx" |
                "pgf" | "sgi"  | "sid" | "bgp" => extended,

                _ => false
            }
        }
    }
}

/// Check if a comic format is supported for decoding
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(is_supported_for_decoding("zip"), true);
/// assert_eq!(is_supported_for_decoding("PdF"), true);
/// assert_eq!(is_supported_for_decoding("mp3"), false);
/// ```
pub fn is_supported_for_decoding(ext: &str) -> bool {
    match ext.to_lowercase().as_str() {
        // Common archive formats
        "zip" => true,

        // Common archive formats with comic-related extension
        "cbz" => true,

        // Non-archive formats
        "pdf" => true,

        // Every other format is not supported
        _ => false
    }
}

/// Get the largest possible number from the first characters of the provided characters iterator
/// The iterator *will* advance up to the first non-digit character
/// Only integers are supported, but there is no size limit
/// A vector containing the digits of the number are returned, the first digit in the string being the first digit in the returned vector
fn take_num(chars: &mut Peekable<Chars>) -> Vec<u8> {
    let mut digits = vec![];

    loop {
        match chars.peek() {
            Some(c) if c.is_ascii_digit() => {
                let code = u32::from(chars.next().unwrap());
                assert!(code >= 0x30 && code <= 0x39);

                let num = code as u8 - 0x30;

                if num != 0 || digits.len() != 0 {
                    digits.push(code as u8 - 0x30);
                }
            },

            _ => break
        }
    }
    
    digits
}

/// Compare two strings using natural order, which is equivalent to traditional UTF-8 sorting \
/// but compares whole numbers instead of single digits
/// 
/// # Examples
/// 
/// ```
/// let mut directories = vec![ "Folder 20", "Folder 1", "Folder 100" ];
/// 
/// // Native sort
/// directories.sort();
/// println!("{:?}", directories); // ["Folder 1", "Folder 100", "Folder 20"]
/// 
/// // Natural sort
/// directories.sort_by(natural_cmp);
/// println!("{:?}", directories); // ["Folder 1", "Folder 20", "Folder 100"]
/// ```
/// 
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
    let left = left.to_lowercase();
    let right = right.to_lowercase();

    let mut left = left.chars().peekable();
    let mut right = right.chars().peekable();

    loop {
        return match (left.peek().copied(), right.peek().copied()) {
            (Some(lc), Some(rc)) => {
                if lc.is_ascii_digit() && rc.is_ascii_digit() {
                    let lnum = take_num(&mut left);
                    let rnum = take_num(&mut right);

                    let cmp = lnum.len().cmp(&rnum.len());

                    if cmp != Ordering::Equal {
                        return cmp;
                    }

                    for (ldig, rdig) in lnum.iter().zip(rnum.iter()) {
                        let cmp = ldig.cmp(rdig);
                        
                        if cmp != Ordering::Equal {
                            return cmp;
                        }
                    }

                    Ordering::Equal
                } else {
                    left.next().unwrap();
                    right.next().unwrap();

                    match lc.cmp(&rc) {
                        Ordering::Equal => continue,
                        ordering @ _ => ordering
                    }
                }
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal
        }
    }
}

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
/// See the "natural_cmp" function for more informations
pub fn natural_paths_cmp(a: &PathBuf, b: &PathBuf) -> Ordering {
    let mut a = a.components();
    let mut b = b.components();

    loop {
        return match (a.next(), b.next()) {
            (Some(a_cp), Some(b_cp)) => match natural_cmp(&a_cp.as_os_str().to_string_lossy(), &b_cp.as_os_str().to_string_lossy()) {
                Ordering::Equal => continue,
                ordering @ _ => ordering
            },
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}
