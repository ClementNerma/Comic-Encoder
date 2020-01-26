use std::path::Path;

/// Check if a path has a common image format extension
/// Additional formats that may not be widely supported can be accepted using the `extended` parameter
/// 
/// # Examples
/// 
/// ```
/// assert_eq!(has_image_ext(Path::new("file.png"), false), true);
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
            Some(ext) => match ext {
                "jpg" | "jpeg" | "png" | "bmp" => true,

                "tif" | "tiff" | "gif" | "eps" | "raw" | "cr2" | "nef"  | "orf" | "sr2" |
                "ppm" | "webp" | "pgm" | "pbm" | "pnm" | "ico" | "flif" | "pam" | "pcx" |
                "pgf" | "sgi"  | "sid" | "bgp" => extended,

                _ => false
            }
        }
    }
}
