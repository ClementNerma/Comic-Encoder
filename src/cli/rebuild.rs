use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use clap::ArgMatches;
use super::decode;
use super::encode;
use super::error::RebuildingError;

/// Rebuild configuration
pub struct Config<'a> {
    input: &'a Path,
    output: Option<&'a Path>,
    overwrite: bool,
    temporary_dir: Option<&'a Path>,
    only_extract_images: bool,
    extended_image_formats: bool,
    disable_nat_sort: bool,
    compress_losslessly: bool
}

/// Rebuild a comic using the provided configuration
/// 
/// This action is achieved in 3 steps:
/// 1. Decode the comic, which means extracting its pages to a temporary directory
/// 2. Encode extracted pages to a single comic file
/// 3. Remove the temporary directory
pub fn rebuild(c: &Config) -> Result<PathBuf, RebuildingError> {
    // To get a better log output, pages are not directly put into a temporary directory
    // Else, user would see messages from decoding and encoding sub-commands like "encoding chapter '__tmp_comic_encoder_extract'..."
    //  which is not very nice.
    // Instead, we create put the images inside another folder in the temporary directory, with the same name than the input file
    // Only the extension is removed to get a proper name
    // This means that with a file named "MyComic.cbz", the temporary directory will be "__tmp_comic_encoder_extract/MyComic"

    // Get absolute path to the input for path manipulation
    let input = env::current_dir().map_err(RebuildingError::FailedToGetCWD)?.join(c.input);

    // Get the temporary directory's wrapper (the one with the ugly name)
    let tmp_dir_wrapper = match c.temporary_dir {
        Some(path) => path.to_path_buf(),
        None => input.parent().ok_or(RebuildingError::InputFileIsRootDirectory)?.join("__tmp_comic_encoder_extract")
    };

    // Get the directory inside the temporary directory when we will put all extracted pages
    let tmp_dir_pages = tmp_dir_wrapper.join(input.with_extension("").file_name().ok_or(RebuildingError::InputFileIsRootDirectory)?);

    // Get the path to the output directory
    let output = match c.output {
        Some(path) => path.to_path_buf(),
        None => input.with_extension("cbz")
    };

    info!("==> (1/2) Extracting images...");

    // Extract all images from the input comic
    decode::decode(&decode::Config {
        input: &input,
        output: Some(&tmp_dir_pages),
        create_output_dir: true,
        only_extract_images: c.only_extract_images,
        extended_image_formats: c.extended_image_formats,
        disable_nat_sort: c.disable_nat_sort
    }).map_err(RebuildingError::DecodingError)?;

    info!("==> (2/2) Encoding images in a book...");

    // Put all images from the input comic in a single comic file
    let path = encode::encode(&encode::Config {
        method: encode::Method::Single,
        chapters_dir: &tmp_dir_wrapper,
        output: Some(&output),
        create_output_dir: false,
        overwrite: c.overwrite,
        dirs_prefix: None,
        start_chapter: None,
        end_chapter: None,
        extended_image_formats: c.extended_image_formats,
        disable_nat_sort: c.disable_nat_sort,
        show_chapters_path: false,
        display_full_names: false,
        compress_losslessly: c.compress_losslessly
    }).map_err(RebuildingError::EncodingError)?;

    assert_eq!(path.len(), 1, "Internal error: encoding during rebuild did not create exactly 1 file");

    debug!("Removing temporary directory...");

    // Remove the (now useless) temporary directory
    if let Err(_) = fs::remove_dir_all(tmp_dir_wrapper) {
        // Don't fail the whole operation is removing failed, as the comic was built successfully nonetheless
        error!("Failed to remove temporary directory!");
    }

    Ok(path[0].clone())
}

/// Rebuild a comic using the provided command-line arguments
pub fn from_args(args: &ArgMatches) -> Result<PathBuf, RebuildingError> {
    rebuild(&Config {
        input: Path::new(args.value_of("input").unwrap()),
        output: args.value_of("output").map(Path::new),
        overwrite: args.is_present("overwrite"),
        temporary_dir: args.value_of("temporary_dir").map(Path::new),
        only_extract_images: args.is_present("only-extract-images"),
        extended_image_formats: args.is_present("extended-image-formats"),
        disable_nat_sort: args.is_present("disable-natural-sorting"),
        compress_losslessly: args.is_present("compress-losslessly")
    })
}
