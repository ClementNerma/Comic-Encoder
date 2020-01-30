use std::path::{Path, PathBuf};
use std::fs;
use clap::ArgMatches;
use super::decode;
use super::encode;
use super::error::RebuildingError;

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

pub fn rebuild(c: &Config) -> Result<PathBuf, RebuildingError> {
    let tmp_dir_wrapper = match c.temporary_dir {
        Some(path) => path.to_path_buf(),
        None => c.input.parent().ok_or(RebuildingError::InputFileIsRootDirectory)?.join("__tmp_comic_encoder_extract")
    };

    let tmp_dir = tmp_dir_wrapper.join(c.input.with_extension("").file_name().ok_or(RebuildingError::InputFileIsRootDirectory)?);

    let output = match c.output {
        Some(path) => path.to_path_buf(),
        None => c.input.with_extension("cbz")
    };

    info!("==> (1/2) Extracting images...");

    decode::decode(&decode::Config {
        input: c.input,
        output: Some(&tmp_dir),
        create_output_dir: true,
        only_extract_images: c.only_extract_images,
        extended_image_formats: c.extended_image_formats,
        disable_nat_sort: c.disable_nat_sort
    }).map_err(RebuildingError::DecodingError)?;

    info!("==> (2/2) Encoding images in a book...");

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

    if let Err(_) = fs::remove_dir_all(tmp_dir_wrapper) {
        error!("Failed to remove temporary directory!");
    }

    Ok(path[0].clone())
}

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
