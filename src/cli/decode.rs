use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io;
use std::time::Instant;
use clap::ArgMatches;
use zip::ZipArchive;
use pdf::file::File as PDFFile;
use pdf::object::XObject;
use crate::lib;
use super::error::DecodingError;

pub struct Config<'a> {
    input: &'a Path,
    output: &'a Path,
    create_output_dir: bool,
    only_extract_images: bool,
    extended_image_formats: bool,
    disable_nat_sort: bool
}

pub fn decode(c: &Config) -> Result<Vec<PathBuf>, DecodingError> {
    if !c.input.exists() {
        Err(DecodingError::InputFileNotFound)?
    } else if !c.input.is_file() {
        Err(DecodingError::InputFileIsADirectory)?
    }

    if !c.output.exists() {
        if c.create_output_dir {
            fs::create_dir_all(c.output).map_err(DecodingError::FailedToCreateOutputDirectory)?
        } else {
            Err(DecodingError::OutputDirectoryNotFound)?
        }
    } else if !c.output.is_dir() {
        Err(DecodingError::OutputDirectoryIsAFile)?
    }

    let ext = c.input.extension().ok_or(DecodingError::UnsupportedFormat(String::new()))?;
    let ext = ext.to_str().ok_or(DecodingError::InputFileHasInvalidUTF8FileExtension(ext.to_os_string()))?;

    let extraction_started = Instant::now();

    let result = match ext {
        "zip" | "cbz" => {
            debug!("Matched input format: ZIP / CBZ");

            trace!("Opening input file...");

            let file = File::open(c.input).map_err(DecodingError::FailedToOpenZipFile)?;

            trace!("Creating ZIP archive...");

            let mut zip = ZipArchive::new(file).map_err(DecodingError::InvalidZipArchive)?;

            let zip_files = zip.len();

            struct ExtractedFile {
                path_in_zip: PathBuf,
                extracted_path: PathBuf,
                extension: Option<String>
            }

            let mut pages: Vec<ExtractedFile> = vec![];
            let mut counter = 0;

            for i in 0..zip.len() {
                trace!("Retrieving ZIP file with ID {}...", i);

                let mut file = zip.by_index(i).map_err(DecodingError::ZipError)?;

                if file.is_file() {
                    let file_name = file.sanitized_name();

                    if c.only_extract_images && !lib::has_image_ext(&file_name, c.extended_image_formats) {
                        trace!("Ignoring file {}/{} based on extension", i + 1, zip_files);
                        continue ;
                    }

                    let ext = file_name.extension()
                        .map(|ext| ext.to_str().ok_or(DecodingError::ZipFileHasInvalidUTF8FileExtension(file_name.clone())))
                        .transpose()?;

                    let outpath = c.output.join(Path::new(&format!("___tmp_pic_{}", counter)));

                    trace!("File is a page. Creating an output file for it...");
                    let mut outfile = File::create(&outpath).map_err(|err| DecodingError::FailedToCreateOutputFile(err, outpath.clone()))?;

                    debug!("Extracting file {} out of {}...", i + 1, zip_files);
                    io::copy(&mut file, &mut outfile).map_err(|err| DecodingError::FailedToExtractZipFile {
                        path_in_zip: file_name.clone(), extract_to: outpath.clone(), err
                    })?;

                    pages.push(ExtractedFile {
                        extension: ext.map(|ext| ext.to_owned()),
                        path_in_zip: file_name,
                        extracted_path: outpath
                    });

                    counter += 1;
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
            let page_num_len = pages.len().to_string().len();

            debug!("Renaming pictures...");

            for (i, page) in pages.into_iter().enumerate() {
                let target = c.output.join(&match page.extension {
                    None => format!("{:0page_num_len$}", i + 1, page_num_len=page_num_len),
                    Some(ref ext) => format!("{:0page_num_len$}.{}", i + 1, ext, page_num_len=page_num_len)
                });

                trace!("Renaming picture {}/{}...", i + 1, total_pages);

                fs::rename(&page.extracted_path, &target).map_err(|err| DecodingError::FailedToRenameTemporaryFile {
                    from: page.extracted_path, to: target.to_path_buf(), err
                })?;

                extracted.push(target);
            }

            Ok(extracted)
        },

        "pdf" => {
            debug!("Matched input format: PDF");
            trace!("Opening input file...");

            let pdf = PDFFile::open(c.input).map_err(DecodingError::FailedToOpenPdfFile)?;

            let mut images = vec![];

            info!("Looking for images in the provided PDF...");

            for (i, page) in pdf.pages().enumerate() {
                trace!("Counting images from page {}...", i);

                let page = page.map_err(|err| DecodingError::FailedToGetPdfPage(i + 1, err))?;
                let resources = page.resources(&pdf).map_err(|err| DecodingError::FailedToGetPdfPageResources(i + 1, err))?;
                images.extend(resources.xobjects.iter().filter_map(|(_, o)| match o {
                    XObject::Image(im) => Some(im.clone()),
                    _ => None
                }));
            }

            info!("Extracting {} images from PDF...", images.len());

            let mut extracted = vec![];
            let page_num_len = images.len().to_string().len();

            for (i, image) in images.iter().enumerate() {
                let outpath = c.output.join(Path::new(&format!("{:0page_num_len$}.jpg", i + 1, page_num_len=page_num_len)));

                debug!("Extracting page {}/{}...", i + 1, images.len());

                fs::write(&outpath, image.as_jpeg().unwrap()).map_err(|err| DecodingError::FailedToExtractPdfImage(i + 1, outpath.clone(), err))?;

                extracted.push(outpath);
            }

            Ok(extracted)
        },

        _ => Err(DecodingError::UnsupportedFormat(ext.to_owned()))
    };

    if let Ok(pages) = &result {
        let elapsed = extraction_started.elapsed();
        info!("Successfully extracted {} pages in {}.{:03} s!", pages.len(), elapsed.as_secs(), elapsed.subsec_millis());
    }

    result
}

pub fn from_args(args: &ArgMatches) -> Result<Vec<PathBuf>, DecodingError> {
    decode(&Config {
        input: Path::new(args.value_of("input").unwrap()),
        output: Path::new(args.value_of("output").unwrap()),
        create_output_dir: args.is_present("create-output-dir"),
        only_extract_images: args.is_present("only-extract-images"),
        extended_image_formats: args.is_present("extended-image-formats"),
        disable_nat_sort: args.is_present("disable-natural-sorting")
    })
}
