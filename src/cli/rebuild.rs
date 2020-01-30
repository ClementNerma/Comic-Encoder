use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use clap::ArgMatches;
use crate::lib;
use super::decode;
use super::encode;
use super::error::RebuildingError;

/// Rebuild configuration
pub struct Config<'a> {
    input: &'a Path,
    output: Option<&'a Path>,
    dir: bool,
    create_output_dir: bool,
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
pub fn rebuild(c: &Config) -> Result<Vec<PathBuf>, RebuildingError> {
    // To get a better log output, pages are not directly put into a temporary directory
    // Else, user would see messages from decoding and encoding sub-commands like "encoding chapter '__tmp_comic_encoder_extract'..."
    //  which is not very nice.
    // Instead, we create put the images inside another folder in the temporary directory, with the same name than the input file
    // Only the extension is removed to get a proper name
    // This means that with a file named "MyComic.cbz", the temporary directory will be "__tmp_comic_encoder_extract/MyComic"

    // Get absolute path to the input for path manipulation
    let input = env::current_dir().map_err(RebuildingError::FailedToGetCWD)?.join(c.input);

    if c.dir {
        return rebuild_dir(c, &input);
    }

    // Get the temporary directory's wrapper (the one with the ugly name)
    let tmp_dir_wrapper = match c.temporary_dir {
        Some(path) => path.to_path_buf(),
        None => input.parent().ok_or(RebuildingError::InputFileIsRootDirectory)?.join("__tmp_comic_encoder_extract")
    };

    // Get the directory inside the temporary directory when we will put all extracted pages
    let tmp_dir_pages = tmp_dir_wrapper.join(input.with_extension("").file_name().ok_or(RebuildingError::InputFileIsRootDirectory)?);

    // If the temporary directory already exists (if for instance previous rebuilding operation was interrupted), remove it
    // This is important because existing images from another comic may be remaining in this directory, which could result in
    //  additional, unrelated pages being added to the rebuilt comic
    if tmp_dir_pages.exists() {
        debug!("Removing existing temporary directory...");
        fs::remove_dir_all(&tmp_dir_pages).map_err(RebuildingError::FailedToRemoveExistingTemporaryDirectory)?;
    }

    // Get the path to the output directory
    let output = match c.output {
        Some(path) => path.to_path_buf(),
        None => input.with_extension("cbz")
    };

    info!("==> Extracting images...");

    // Extract all images from the input comic
    decode::decode(&decode::Config {
        input: &input,
        output: Some(&tmp_dir_pages),
        create_output_dir: true,
        only_extract_images: c.only_extract_images,
        extended_image_formats: c.extended_image_formats,
        disable_nat_sort: c.disable_nat_sort
    }, true).map_err(RebuildingError::DecodingError)?;

    info!("==> Encoding images in a book...");

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
    }, true).map_err(RebuildingError::EncodingError)?;

    assert_eq!(path.len(), 1, "Internal error: encoding during rebuild did not create exactly 1 file");

    debug!("Removing temporary directory...");

    // Remove the (now useless) temporary directory
    if let Err(_) = fs::remove_dir_all(tmp_dir_wrapper) {
        // Don't fail the whole operation is removing failed, as the comic was built successfully nonetheless
        error!("Failed to remove temporary directory!");
    }

    Ok(vec![ path[0].clone() ])
}

/// Rebuild all comics in a directory (internal function)
fn rebuild_dir(c: &Config, input: &Path) -> Result<Vec<PathBuf>, RebuildingError> {
    trace!("Task: rebuild all files from input directory");

    if !input.exists() {
        Err(RebuildingError::InputDirectoryNotFound)?;
    }

    let output = c.output.unwrap_or(&input);

    // Ensure output directory exists, or create it if asked to
    if !output.exists() {
        if c.create_output_dir {
            trace!("Creating output directory...");

            fs::create_dir_all(output).map_err(RebuildingError::FailedToCreateOutputDirectory)?;
        } else {
            Err(RebuildingError::OutputDirectoryNotFound)?;
        }
    } else if output.is_file() {
        Err(RebuildingError::OutputDirectoryIsAFile)?;
    }

    // The list of files to rebuild
    let mut files = vec![];

    debug!("Checking all files to rebuild...");

    // Check all comic files in the input directory
    for item in fs::read_dir(&input).map_err(RebuildingError::FailedToReadInputDirectory)? {
        let item = item.map_err(RebuildingError::FailedToReadInputDirectory)?.path();

        if item.is_file() {
            if let Some(ext) = item.extension() {
                match ext.to_str() {
                    None => Err(RebuildingError::InputItemHasInvalidUTF8Extension(item))?,
                    Some(ext) => if ext == "zip" || ext == "cbz" || ext == "pdf" {
                        files.push(item)
                    }
                }
            }
        }
    }

    debug!("Found {} files!", files.len());

    trace!("Sorting files...");
    files.sort_by(|a, b| lib::natural_paths_cmp(a, b));

    let total = files.len();
    let file_name_len = total.to_string().len();   

    // List of built files
    let mut output_files = vec![];

    info!("Going to rebuild {} files.", total);

    // Rebuild all files
    for (i, file) in files.iter().enumerate() {
        let file_name = file.file_name().unwrap();

        info!("=> ({:0file_name_len$}/{}): {}", i + 1, total, file_name.to_string_lossy(), file_name_len=file_name_len);

        output_files.extend_from_slice(&rebuild(&Config {
            input: &input.join(file_name),
            output: Some(&output.join(file.with_extension("cbz").file_name().unwrap())),
            dir: false,
            create_output_dir: false,
            overwrite: c.overwrite,
            temporary_dir: c.temporary_dir,
            only_extract_images: c.only_extract_images,
            extended_image_formats: c.extended_image_formats,
            disable_nat_sort: c.disable_nat_sort,
            compress_losslessly: c.compress_losslessly
        })?);
    }

    Ok(output_files)
}

/// Rebuild a comic using the provided command-line arguments
pub fn from_args(args: &ArgMatches) -> Result<Vec<PathBuf>, RebuildingError> {
    rebuild(&Config {
        input: Path::new(args.value_of("input").unwrap()),
        output: args.value_of("output").map(Path::new),
        dir: args.is_present("dir"),
        create_output_dir: args.is_present("create-output-dir"),
        overwrite: args.is_present("overwrite"),
        temporary_dir: args.value_of("temporary-dir").map(Path::new),
        only_extract_images: args.is_present("only-extract-images"),
        extended_image_formats: args.is_present("extended-image-formats"),
        disable_nat_sort: args.is_present("disable-natural-sorting"),
        compress_losslessly: args.is_present("compress-losslessly")
    })
}
