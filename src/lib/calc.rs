use std::ops::{Add, Rem, Div};
use std::cmp::PartialEq;

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
