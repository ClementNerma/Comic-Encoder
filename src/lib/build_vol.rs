use std::time::Instant;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::fs::{self, File};
use zip::CompressionMethod;
use zip::write::{ZipWriter, FileOptions};
use crate::lib::deter;
use crate::cli::error::EncodingError;
use crate::cli::opts::*;

#[derive(Copy, Clone)]
pub enum BuildMethod<'a> {
    Ranges(&'a CompileRanges, &'a CompilationOptions),
    Each(&'a CompileEach, &'a CompilationOptions),
    Single(&'a EncodeSingle)
}

/// Build a volume
/// `output` is the actual output path
/// `volume` is the current volume number, starting at 1
/// `volumes` is the total number of volumes
/// `vol_num_len` is the maximum string length of the volume number (e.g. 1520 volumes => `vol_num_len == 4`)
/// `chapter_num_len` is like `vol_num_len` but for chapters
/// `start_chapter` is the number of the first chapter in this volume
/// `chapters` is a list of the chapters this volume contains. It's a vector of tuples containing: (chapter number, path to the chapter's directory, chapter's directory's file name)
pub fn build_volume(method: &'_ BuildMethod, enc_opts: &'_ EncodingOptions, output: &'_ Path, volume: usize, volumes: usize, vol_num_len: usize, chapter_num_len: usize, start_chapter: usize, chapters: impl AsRef<[(usize, PathBuf, String)]>) -> Result<PathBuf, EncodingError> {
    // Get timestamp to measure performance
    let build_started = Instant::now();

    let chapters = chapters.as_ref();

    // Get the file name for this volume
    let output_path_without_ext = match method {
        BuildMethod::Ranges(opts, _) => if !opts.append_chapters_range || chapters.len() == 0 {
            output.join(format!("Volume-{:0vol_num_len$}", volume, vol_num_len=vol_num_len))
        } else {
            output.join(format!(
                "Volume-{:0vol_num_len$} (c{:0chapter_num_len$}-c{:0chapter_num_len$})",
                volume,
                start_chapter,
                start_chapter + chapters.len() - 1,
                vol_num_len = vol_num_len,
                chapter_num_len = chapter_num_len
            ))
        },

        BuildMethod::Each(_, _) => {
            assert_eq!(chapters.len(), 1, "Internal error: individual chapter's volume does contain exactly 1 chapter!");
            output.join(format!("{}", chapters[0].2))
        },

        BuildMethod::Single(_) => output.with_extension("")
    };

    // If the number of pages won't be happened to the final name, we can predict the final name of the file
    // Else we cannot as we don't know the number of pages in this volume, yet.
    if let BuildMethod::Each(opts, _) = method {
        // And if 'skip_existing' is set, that means we don't have to append the number of pages as this argument
        // conflicts with the 'append_pages_count'.
        if opts.skip_existing {
            let complete_path = output_path_without_ext.with_extension("cbz");

            if complete_path.exists() {
                warn!("Warning: skipping volume {} containing chapters {} to {} as its output file '{}' already exists (--skip-existing provided)", volume, start_chapter, start_chapter + chapters.len() - 1, output.to_string_lossy());
                return Ok(complete_path);
            }
        }
    }

    // Get the path to this volume's (staging) ZIP archive
    let staging_path = output_path_without_ext.with_extension(".comic-enc-partial");

    // Fail if the target file already exists and '--overwrite' has not been specified
    if staging_path.exists() && !enc_opts.overwrite {
        Err(EncodingError::OutputVolumeFileAlreadyExists(volume, staging_path.clone()))?
    }

    // Create a ZIP file to this path
    let zip_file = File::create(staging_path.clone()).map_err(|err| EncodingError::FailedToCreateVolumeFile(volume, err))?;

    let mut zip_writer = ZipWriter::new(zip_file);

    // Consider compression
    let zip_options = FileOptions::default().compression_method(
        if enc_opts.compress_losslessly { CompressionMethod::Deflated } else { CompressionMethod::Stored }
    );

    // Determine the common display name for individual chapters
    let display_name_individual = match method {
        BuildMethod::Each(opts, _) => Some(match opts.display_full_names {
            true => format!("'{}'", chapters[0].2.clone()),
            false => if chapters[0].2.len() <= 50 {
                format!("'{}'", chapters[0].2)
            } else {
                let cut: String = chapters[0].2.chars().take(50).collect();
                format!("'{}...'", cut)
            }
        }),

        _ => None
    };

    // Determine how to display the volume's name in STDOUT
    let volume_display_name = match method {
        BuildMethod::Ranges(_, _) => format!("{:0vol_num_len$}", volume, vol_num_len = vol_num_len),
        BuildMethod::Each(_, _) => format!("'{}'", display_name_individual.as_ref().unwrap()),
        BuildMethod::Single(_) => format!("'{}'", output_path_without_ext.file_name().expect("Internal error: output path without extension has no filename when building").to_string_lossy())
    };

    // Prepare a buffer to store the picture's files
    let mut buffer = Vec::new();
    
    // Count the number of pictures in this volume
    let mut pics_counter = 0;

    // Treat each chapter of the volume
    for (chapter, chapter_path, chapter_name) in chapters.iter() {
        // Determine how to display the chapter's title in STDOUT
        let chapter_display_name = match method {
            BuildMethod::Each(_, _) => format!("'{}'", display_name_individual.as_ref().unwrap()),
            _ => format!("{:0chapter_num_len$}", chapter, chapter_num_len = chapter_num_len)
        };

        trace!("Reading files recursively from chapter {}'s directory '{}'...", chapter, chapter_name);

        // Get the list of all image files in the chapter's directory, recursively
        let mut chapter_pics = deter::readdir_files_recursive(&chapter_path, Some(
                &|path: &PathBuf| deter::has_image_ext(path, enc_opts.accept_extended_image_formats)
            ))
            .map_err(|err| EncodingError::FailedToListChapterDirectoryFiles {
                volume, chapter: *chapter, chapter_path: chapter_path.to_path_buf(), err
            })?;

        trace!("Found '{}' picture files from chapter {}'s directory '{}'. Sorting them...", chapter_pics.len(), chapter, chapter_name);

        match method {
            BuildMethod::Ranges(opts, _) =>
                if opts.debug_chapters_path {
                    info!("Adding chapter {} to volume {} from directory '{}'", chapter_display_name, volume_display_name, chapter_name);
                } else {
                    debug!("Adding chapter {} to volume {}...", chapter_display_name, volume_display_name);
                },

            BuildMethod::Each(_, _) =>
                debug!("Adding directory n°{} to volume {}", chapter_display_name, volume_display_name),

            BuildMethod::Single(_) => {}
        }

        // Sort the image files by name
        if enc_opts.simple_sorting {
            chapter_pics.sort();
        } else {
            chapter_pics.sort_by(deter::natural_paths_cmp);
        };

        // Disable mutability for this variable
        let chapter_path = chapter_path;

        // Determine the name of this chapter's directory in the volume's ZIP
        let zip_dir_name = match method {
            BuildMethod::Each(_, _) => chapters[0].2.clone(),

            _ => format!(
                "Vol_{:0vol_num_len$}_Chapter_{:0chapter_num_len$}",
                volume, chapter, vol_num_len = vol_num_len, chapter_num_len = chapter_num_len
            )
        };

        trace!("Adding directory '{}' to ZIP archive...", zip_dir_name);

        // Create an empty directory for this chapter in the volume's ZIP
        zip_writer.add_directory(&zip_dir_name, zip_options).map_err(|err| EncodingError::FailedToCreateChapterDirectoryInZip {
            volume, chapter: *chapter, dir_name: zip_dir_name.to_owned(), err
        })?;

        // Compute the length of displayable picture number (e.g. 1520 pictures will give 4)
        let pic_num_len = chapter_pics.len().to_string().len();

        // Iterate over each page
        for (page_nb, file) in chapter_pics.iter().enumerate() {
            // Determine the name of the file in the ZIP directory
            let name_in_zip = match method {
                BuildMethod::Each(_, _) => format!(
                    "{}_Pic_{:0pic_num_len$}.{file_ext}",
                    volume_display_name,
                    page_nb,
                    file_ext = file.extension().unwrap().to_str().ok_or_else(
                        || EncodingError::ItemHasInvalidUTF8Name(file.file_name().unwrap().to_os_string())
                    )?,
                    pic_num_len = pic_num_len
                ),

                _ => format!(
                    "Vol_{:0vol_num_len$}_Chapter_{:0chapter_num_len$}_Pic_{:0pic_num_len$}.{file_ext}",
                    volume,
                    chapter,
                    page_nb,
                    file_ext = file.extension().unwrap().to_str().ok_or_else(
                        || EncodingError::ItemHasInvalidUTF8Name(file.file_name().unwrap().to_os_string())
                    )?,
                    vol_num_len = vol_num_len,
                    chapter_num_len = chapter_num_len,
                    pic_num_len = pic_num_len
                ),
            };

            trace!(
                "Adding picture {:0pic_num_len$} at '{}' from chapter {} to volume {} as '{}/{}'...",
                page_nb, file.to_string_lossy(), chapter_display_name, volume_display_name, zip_dir_name, name_in_zip, pic_num_len = pic_num_len
            );

            // Determine the path of the file in the ZIP directory
            let path_in_zip = &Path::new(&zip_dir_name).join(Path::new(&name_in_zip));

            // Create the empty file in the archive
            zip_writer.start_file_from_path(path_in_zip, zip_options)
                .map_err(|err| EncodingError::FailedToCreateImageFileInZip {
                    volume, chapter: *chapter, file_path: path_in_zip.to_path_buf(), err
                })?;

            // Read the real file
            let mut f = File::open(file).map_err(|err| EncodingError::FailedToOpenImage {
                volume, chapter: *chapter, chapter_path: chapter_path.to_path_buf(), image_path: file.to_path_buf(), err
            })?;

            f.read_to_end(&mut buffer).map_err(|err| EncodingError::FailedToReadImage {
                volume, chapter: *chapter, chapter_path: chapter_path.to_path_buf(), image_path: file.to_path_buf(), err
            })?;

            // Write the file to the ZIP archive
            zip_writer.write_all(&buffer).map_err(|err| EncodingError::FailedToWriteImageFileToZip {
                volume, chapter: *chapter, chapter_path: chapter_path.to_path_buf(), image_path: file.to_path_buf(), err
            })?;

            buffer.clear();

            pics_counter += 1;
        }
    }

    trace!("Closing ZIP archive...");

    // Close the archive
    zip_writer.finish().map_err(|err| EncodingError::FailedToCloseZipArchive(volume, err))?;

    // Determine the file's final path with the right (non-partial) extension + number of pages if asked to
    let mut complete_path = output_path_without_ext.with_extension("cbz");

    if enc_opts.append_pages_count {
        let mut filename_with_pages = complete_path
            .with_extension("")
            .file_name()
            .expect("Internal error: output path when building has no filename")
            .to_os_string();
        
        filename_with_pages.push(format!(" ({} pages).cbz", pics_counter));
        
        complete_path = complete_path.with_file_name(filename_with_pages)
    };

    // Check if final path exists
    if complete_path.exists() {
        if complete_path.exists() && !enc_opts.overwrite {
            Err(EncodingError::OutputVolumeFileAlreadyExists(volume, complete_path.clone()))?
        }

        if !complete_path.is_dir() {
            Err(EncodingError::OutputVolumeFileIsADirectory(volume, complete_path.clone()))?
        }

        if let Err(err) = fs::remove_file(&complete_path) {
            Err(EncodingError::FailedToOverwriteOutputVolumeFile(volume, complete_path.clone(), err))?
        }
    }

    // Rename the staging file to its complete name
    if let Err(err) = fs::rename(&staging_path, &complete_path) {
        return Err(EncodingError::FailedToRenameCompleteArchive(volume, err));
    }

    let complete_filename = complete_path.file_name()
        .expect("Internal error: output path when building has no filename")
        .to_string_lossy();

    // Get the eventually truncated file name to display in the success message
    let success_display_file_name = match complete_filename.len() {
        0..=50 => complete_filename.to_string(),
        _ => format!("{}...", complete_filename.chars().take(50).collect::<String>())
    };

    // Compute elapsed time
    let elapsed = build_started.elapsed();

    // Format elapsed time
    let elapsed = format!("{}.{:03} s", elapsed.as_secs(), elapsed.subsec_millis());

    // Padding for after the filename
    let filename_right_padding = if success_display_file_name.len() < 50 { " ".repeat(50 - success_display_file_name.len()) } else { String::new() };

    match method {
        BuildMethod::Each(_, _) => info!(
            "Successfully written volume {:0vol_num_len$} / {} to file '{}{}', containing {} pages in {}.",
            volume,
            volumes,
            success_display_file_name,
            filename_right_padding,
            pics_counter,
            elapsed,
            vol_num_len = vol_num_len
        ),

        _ => info!(
            "Successfully written volume {} / {} (chapters {:0chapter_num_len$} to {:0chapter_num_len$}) in '{}'{}, containing {} pages in {}.",
            volume_display_name,
            volumes,
            start_chapter,
            start_chapter + chapters.len() - 1,
            success_display_file_name,
            filename_right_padding,
            pics_counter,
            elapsed,
            chapter_num_len = chapter_num_len
        )
    }

    Ok(staging_path)
}