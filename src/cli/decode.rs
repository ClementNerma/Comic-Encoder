use std::path::{Path, PathBuf};
use std::env;
use std::fs::{self, File};
use std::io;
use std::time::Instant;
use clap::ArgMatches;
use zip::ZipArchive;
use pdf::file::File as PDFFile;
use pdf::object::XObject;
use crate::lib;
use super::error::DecodingError;

/// Decoding configuration
pub struct Config<'a> {
    /// Path to the comic
    pub input: &'a Path,
    /// Path to the output directory
    pub output: Option<&'a Path>,
    /// Create the output directory if it does not exist
    pub create_output_dir: bool,
    /// Only extract supported image formats (other files will be ignored)
    pub only_extract_images: bool,
    /// When only extracting images, allow extended image formats that may not be supported by comic readers
    pub extended_image_formats: bool,
    /// Disables natural sort and rely on native UTF-8 sort instead, which gives an intuitive order of items (e.g. `folder 10` will be _before_ `folder 2`)
    pub disable_nat_sort: bool
}

/// Perform a decoding using the provided configuration object
/// `is_rebuilding` is used to indicate encoding is performed as part of the 'rebuild' method
pub fn decode(c: &Config, is_rebuilding: bool) -> Result<Vec<PathBuf>, DecodingError> {
    // Get absolute path to the input for path manipulation
    let input = env::current_dir().map_err(DecodingError::FailedToGetCWD)?.join(c.input);

    // Check if the input file exists
    if !input.exists() {
        Err(DecodingError::InputFileNotFound)?
    } else if !input.is_file() {
        Err(DecodingError::InputFileIsADirectory)?
    }

    // Determine a prefix to show when rebuilding
    let rebuild_prefix = if is_rebuilding { "===> " } else { "" };

    // Create the output directory if needed, and get the output path
    let output = match c.output {
        Some(output) => {
            if !output.exists() {
                if c.create_output_dir {
                    fs::create_dir_all(output).map_err(DecodingError::FailedToCreateOutputDirectory)?
                } else {
                    Err(DecodingError::OutputDirectoryNotFound)?
                }
            } else if !output.is_dir() {
                Err(DecodingError::OutputDirectoryIsAFile)?
            }

            output.to_owned()
        },

        None => {
            let path = input.with_extension("").to_owned();
            fs::create_dir_all(&path).map_err(DecodingError::FailedToCreateOutputDirectory)?;
            path
        }
    };

    // Get the input file's extension to determine its format
    let ext = input.extension().ok_or(DecodingError::UnsupportedFormat(String::new()))?;
    let ext = ext.to_str().ok_or(DecodingError::InputFileHasInvalidUTF8FileExtension(input.file_name().unwrap().to_os_string()))?;

    // Get timestamp to measure decoding time
    let extraction_started = Instant::now();

    // Decode
    let result = match ext {
        "zip" | "cbz" => {
            debug!("Matched input format: ZIP / CBZ");
            trace!("Opening input file...");

            let file = File::open(input).map_err(DecodingError::FailedToOpenZipFile)?;

            trace!("Opening ZIP archive...");

            let mut zip = ZipArchive::new(file).map_err(DecodingError::InvalidZipArchive)?;

            let zip_files = zip.len();

            /// Represent a page that has been extracted from the comic archive
            struct ExtractedFile {
                path_in_zip: PathBuf,
                extracted_path: PathBuf,
                extension: Option<String>
            }

            // List of extracted pages
            let mut pages: Vec<ExtractedFile> = vec![];

            for i in 0..zip.len() {
                trace!("Retrieving ZIP file with ID {}...", i);

                // Get a file from the ZIP
                let mut file = zip.by_index(i).map_err(DecodingError::ZipError)?;

                // Ignore folders
                if file.is_file() {
                    let file_name = file.sanitized_name();

                    // Ensure the file is an image if only images have to be extracted
                    if c.only_extract_images && !lib::has_image_ext(&file_name, c.extended_image_formats) {
                        trace!("Ignoring file {}/{} based on extension", i + 1, zip_files);
                        continue ;
                    }

                    // Get the file's extension to determine output file's name
                    let ext = file_name.extension()
                        .map(|ext| ext.to_str().ok_or(DecodingError::ZipFileHasInvalidUTF8FileExtension(file_name.clone())))
                        .transpose()?;

                    let outpath = output.join(Path::new(&format!("___tmp_pic_{}", pages.len())));

                    // Create output file
                    trace!("File is a page. Creating an output file for it...");
                    let mut outfile = File::create(&outpath).map_err(|err| DecodingError::FailedToCreateOutputFile(err, outpath.clone()))?;

                    // Extract the page
                    debug!("Extracting file {} out of {}...", i + 1, zip_files);
                    io::copy(&mut file, &mut outfile).map_err(|err| DecodingError::FailedToExtractZipFile {
                        path_in_zip: file_name.clone(), extract_to: outpath.clone(), err
                    })?;

                    pages.push(ExtractedFile {
                        extension: ext.map(|ext| ext.to_owned()),
                        path_in_zip: file_name,
                        extracted_path: outpath
                    });
                }
            }

            trace!("Sorting pages...");

            if c.disable_nat_sort {
                pages.sort_by(|a, b| a.path_in_zip.cmp(&b.path_in_zip));
            } else {
                pages.sort_by(|a, b| lib::natural_paths_cmp(&a.path_in_zip, &b.path_in_zip));
            }

            let total_pages = pages.len();

            let mut extracted = vec![];

            // Get the number of characters the last page takes to display
            let page_num_len = pages.len().to_string().len();

            debug!("Renaming pictures...");

            for (i, page) in pages.into_iter().enumerate() {
                let target = output.join(&match page.extension {
                    None => format!("{:0page_num_len$}", i + 1, page_num_len=page_num_len),
                    Some(ref ext) => format!("{:0page_num_len$}.{}", i + 1, ext, page_num_len=page_num_len)
                });

                trace!("Renaming picture {}/{}...", i + 1, total_pages);

                fs::rename(&page.extracted_path, &target).map_err(|err| DecodingError::FailedToRenameTemporaryFile {
                    from: page.extracted_path, to: target.to_owned(), err
                })?;

                extracted.push(target);
            }

            Ok(extracted)
        },

        "pdf" => {
            debug!("Matched input format: PDF");
            trace!("Opening input file...");

            let pdf = PDFFile::open(input).map_err(DecodingError::FailedToOpenPdfFile)?;

            let mut images = vec![];

            debug!("Looking for images in the provided PDF...");

            // List all images in the PDF
            for (i, page) in pdf.pages().enumerate() {
                trace!("Counting images from page {}...", i);

                let page = page.map_err(|err| DecodingError::FailedToGetPdfPage(i + 1, err))?;
                let resources = page.resources(&pdf).map_err(|err| DecodingError::FailedToGetPdfPageResources(i + 1, err))?;
                images.extend(resources.xobjects.iter().filter_map(|(_, o)| match o {
                    XObject::Image(im) => Some(im.clone()),
                    _ => None
                }));
            }

            info!("{}Extracting {} images from PDF...", rebuild_prefix, images.len());

            let mut extracted = vec![];
            let page_num_len = images.len().to_string().len();

            // Extract all images from the PDF
            for (i, image) in images.iter().enumerate() {
                let outpath = output.join(Path::new(&format!("{:0page_num_len$}.jpg", i + 1, page_num_len=page_num_len)));

                debug!("Extracting page {}/{}...", i + 1, images.len());

                fs::write(&outpath, image.as_jpeg().unwrap()).map_err(|err| DecodingError::FailedToExtractPdfImage(i + 1, outpath.clone(), err))?;

                extracted.push(outpath);
            }

            Ok(extracted)
        },

        _ => {
            if lib::is_supported_for_decoding(ext) {
                warn!("Internal error: format '{}' cannot be handled but is marked as supported nonetheless", ext);
            }

            Err(DecodingError::UnsupportedFormat(ext.to_owned()))
        }
    };

    if let Ok(pages) = &result {
        let elapsed = extraction_started.elapsed();
        info!("{}Successfully extracted {} pages in {}.{:03} s!", rebuild_prefix, pages.len(), elapsed.as_secs(), elapsed.subsec_millis());
    }

    result
}

/// Perform a decoding using the provided command-line arguments
pub fn from_args(args: &ArgMatches) -> Result<Vec<PathBuf>, DecodingError> {
    decode(&Config {
        input: Path::new(args.value_of("input").unwrap()),
        output: args.value_of("output").map(|out| Path::new(out)),
        create_output_dir: args.is_present("create-output-dir"),
        only_extract_images: args.is_present("only-extract-images"),
        extended_image_formats: args.is_present("extended-image-formats"),
        disable_nat_sort: args.is_present("disable-natural-sorting")
    }, false)
}
