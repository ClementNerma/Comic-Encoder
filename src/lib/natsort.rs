use std::cmp::Ordering;
use std::str::Chars;
use std::iter::Peekable;

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
