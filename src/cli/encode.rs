use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::env;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::time::Instant;
use clap::ArgMatches;
use zip::{ZipWriter, CompressionMethod};
use zip::write::FileOptions;
use super::error::EncodingError;
use crate::lib;

/// Encoding method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Compile multiple chapters in single volumes
    Compile(u16),
    /// Put each chapter in a single volume
    Individual,
    /// Compile all chapters in a single volume
    Single
}

/// Encoding configuration
#[derive(Debug, PartialEq, Eq)]
pub struct Config<'a> {
    /// The encoding method to use
    pub method: Method,
    /// Path to the directory containing all chapters
    pub chapters_dir: &'a Path,
    /// Path to either the output directory or the output file, depending on the method
    pub output: Option<&'a Path>,
    // Add the start and end chapter at the end of each volume's filename
    pub chapters_suffix: bool,
    /// Create the output directory if it does not exist (or the output file's parent directory)
    pub create_output_dir: bool,
    /// Overwrite existing files instead of failing
    pub overwrite: bool,
    /// Ignore output files that already exist
    pub skip_existing: bool,
    /// If provided, ignore every chapter directory that does not start by the provided prefix
    pub dirs_prefix: Option<&'a str>,
    /// Start at a specific chapter number (chap. numbers start at 1)
    pub start_chapter: Option<usize>,
    /// Ends at a specific chapter number (chap. numbers start at 1)
    pub end_chapter: Option<usize>,
    /// Consider input path as a single chapter directory (don't look at sub-directory chapters)
    pub root_chapter: bool,
    /// Allows to use extended image formats that may not be supported by comic readers
    pub extended_image_formats: bool,
    /// Disables natural sort and rely on native UTF-8 sort instead, which gives an intuitive order of items (e.g. `folder 10` will be _before_ `folder 2`)
    pub disable_nat_sort: bool,
    /// Displays the path of each chapter before it is put in a volume
    pub show_chapters_path: bool,
    /// Display full output file names (by default they are truncated above 50 characters)
    pub display_full_names: bool,
    /// Compresses losslessly all images, which is a lot slower but usually saves around 5% of space
    pub compress_losslessly: bool
}

/// Build a volume
/// `is_rebuilding` is used to indicate encoding is performed as part of the 'rebuild' method
/// `output` is the actuel output path
/// `volume` is the current volume number, starting at 1
/// `volumes` is the total number of volumes
/// `vol_num_len` is the maximum string length of the volume number (e.g. 1520 volumes => `vol_num_len == 4`)
/// `chapter_num_len` is like `vol_num_len` but for chapters
/// `start_chapter` is the number of the first chapter in this volume
/// `chapters` is a list of the chapters this volume contains. It's a vector of tuples containing: (chapter number, path to the chapter's directory, chapter's directory's file name)
/// `config` is the configuration to use
fn build(c: &Config<'_>, is_rebuilding: bool, output: &'_ Path, volume: usize, volumes: usize, vol_num_len: usize, chapter_num_len: usize, start_chapter: usize, chapters: &Vec<(usize, PathBuf, String)>) -> Result<PathBuf, EncodingError> {
    // Get timestamp to measure performance
    let build_started = Instant::now();

    // Get the file name for this volume
    let file_name = match c.method {
        Method::Compile(_) => if !c.chapters_suffix || chapters.len() == 0 {
            format!("Volume-{:0vol_num_len$}.cbz", volume, vol_num_len=vol_num_len)
        } else {
            format!(
                "Volume-{:0vol_num_len$} (c{:0chapter_num_len$}-c{:0chapter_num_len$}).cbz",
                volume,
                start_chapter,
                start_chapter + chapters.len() - 1,
                vol_num_len = vol_num_len,
                chapter_num_len = chapter_num_len
            )
        },

        Method::Individual => {
            assert_eq!(chapters.len(), 1, "Internal error: individual chapter's volume does contain exactly 1 chapter!");
            format!("{}.cbz", chapters[0].2)
        },

        Method::Single => output.file_name().unwrap().to_str().unwrap().to_owned()
    };

    // Get the path to this volume's future ZIP archive
    let zip_path = match c.method {
        Method::Compile(_) | Method::Individual => output.join(Path::new(&file_name)),
        Method::Single => output.to_path_buf()
    };

    // Fail if the target file already exists and '--overwrite' has not been specified
    if !c.overwrite && zip_path.exists() {
        if c.skip_existing {
            warn!("Warning: skipping volume {} containing chapters {} to {} as its output file '{}' already exists (--skip-existing provided)", volume, start_chapter, start_chapter + chapters.len() - 1, zip_path.to_string_lossy());
            return Ok(zip_path);
        }

        Err(EncodingError::OutputFileAlreadyExists(volume, zip_path.clone()))?
    }

    // Create a ZIP file to this path
    let zip_file = File::create(zip_path.clone()).map_err(|err| EncodingError::FailedToCreateVolumeFile(volume, err))?;

    let mut zip_writer = ZipWriter::new(zip_file);

    // Consider compression
    let zip_options = FileOptions::default().compression_method(if c.compress_losslessly {
        CompressionMethod::Deflated
    } else {
        CompressionMethod::Stored
    });

    // Determine the common display name for individual chapters
    let display_name_individual = match c.method {
        Method::Individual => Some(match c.display_full_names {
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
    let volume_display_name = match c.method {
        Method::Compile(_) => format!("{:0vol_num_len$}", volume, vol_num_len = vol_num_len),
        Method::Individual => format!("'{}'", display_name_individual.as_ref().unwrap()),
        Method::Single => format!("'{}'", file_name)
    };

    // Prepare a buffer to store the picture's files
    let mut buffer = Vec::new();
    
    // Count the number of pictures in this volume
    let mut pics_counter = 0;

    // Treat each chapter of the volume
    for (chapter, chapter_path, chapter_name) in chapters.iter() {
        // Determine how to display the chapter's title in STDOUT
        let chapter_display_name = match c.method == Method::Individual {
            false => format!("{:0chapter_num_len$}", chapter, chapter_num_len = chapter_num_len),
            true => format!("'{}'", display_name_individual.as_ref().unwrap())
        };

        trace!("Reading files recursively from chapter {}'s directory '{}'...", chapter, chapter_name);

        // Get the list of all image files in the chapter's directory, recursively
        let mut chapter_pics = lib::readdir_files_recursive(&chapter_path, Some(&|path: &PathBuf| lib::has_image_ext(path, c.extended_image_formats)))
            .map_err(|err| EncodingError::FailedToListChapterDirectoryFiles {
                volume, chapter: *chapter, chapter_path: chapter_path.to_path_buf(), err
            })?;

        trace!("Found '{}' picture files from chapter {}'s directory '{}'. Sorting them...", chapter_pics.len(), chapter, chapter_name);

        // If asked, show the path to the chapter's directory
        if c.show_chapters_path {
            info!("Adding chapter {} to volume {} from directory '{}'", chapter_display_name, volume_display_name, chapter_name);
        } else if c.method != Method::Individual {
            debug!("Adding chapter {} to volume {}...", chapter_display_name, volume_display_name);
        }

        // Sort the image files by name
        if c.disable_nat_sort {
            chapter_pics.sort();
        } else {
            chapter_pics.sort_by(lib::natural_paths_cmp);
        };

        // Disable mutability for this variable
        let chapter_path = chapter_path;

        // Determine the name of this chapter's directory in the volume's ZIP
        let zip_dir_name = match c.method == Method::Individual {
            false => format!(
                "Vol_{:0vol_num_len$}_Chapter_{:0chapter_num_len$}",
                volume, chapter, vol_num_len = vol_num_len, chapter_num_len = chapter_num_len
            ),

            true => chapters[0].2.clone()
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
            let name_in_zip = match c.method == Method::Individual {
                false => format!(
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

                true => format!(
                    "{}_Pic_{:0pic_num_len$}.{file_ext}",
                    volume_display_name,
                    page_nb,
                    file_ext = file.extension().unwrap().to_str().ok_or_else(
                        || EncodingError::ItemHasInvalidUTF8Name(file.file_name().unwrap().to_os_string())
                    )?,
                    pic_num_len = pic_num_len
                )
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

    // Get the eventually truncated file name to display in the success message
    let success_display_file_name = match file_name.len() {
        0..=50 => file_name.clone(),
        _ => format!("{}...", file_name.chars().take(50).collect::<String>())
    };

    // Compute elapsed time
    let elapsed = build_started.elapsed();

    // Format elapsed time
    let elapsed = format!("{}.{:03} s", elapsed.as_secs(), elapsed.subsec_millis());

    // Padding for after the filename
    let filename_right_padding = if success_display_file_name.len() < 50 { " ".repeat(50 - success_display_file_name.len()) } else { String::new() };

    if c.method == Method::Individual || is_rebuilding {
        info!(
            "{}Successfully written volume {:0vol_num_len$} / {} to file '{}'{}, containing {} pages in {}.",
            if is_rebuilding { "===> " } else { "" },
            volume,
            volumes,
            success_display_file_name,
            filename_right_padding,
            pics_counter,
            elapsed,
            vol_num_len = vol_num_len
        );
    } else {
        info!(
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
        );
    }
    
    Ok(zip_path)
}

/// Perform an encoding using the provided configuration object
/// `is_rebuilding` is used to indicate encoding is performed as part of the 'rebuild' method
pub fn encode(c: &Config, is_rebuilding: bool) -> Result<Vec<PathBuf>, EncodingError> {
    // Get the number of chapters to put in each volume
    let chap_per_vol = match c.method {
        Method::Compile(chap_per_vol) => chap_per_vol,
        Method::Individual => 1,
        Method::Single => std::u16::MAX
    };

    if chap_per_vol == 0 {
        Err(EncodingError::AtLeast1ChapterPerVolume)?
    }

    if let Some(start_chapter) = c.start_chapter {
        if start_chapter == 0 {
            Err(EncodingError::InvalidStartChapter)?
        }
    }

    if let Some(end_chapter) = c.end_chapter {
        if end_chapter == 0 {
            Err(EncodingError::InvalidEndChapter)?
        }
    }

    if let (Some(start_chapter), Some(end_chapter)) = (c.start_chapter, c.end_chapter) {
        if end_chapter < start_chapter {
            Err(EncodingError::StartChapterCannotBeHigherThanEndChapter)?
        }
    }

    // Get absolute path to the chapters dir
    let chapters_dir = env::current_dir().map_err(EncodingError::FailedToGetCWD)?
        .join(if !c.root_chapter { c.chapters_dir } else { c.chapters_dir.parent().ok_or(EncodingError::RootCannotBeUsedAsSingleChapter)? });

    if !chapters_dir.is_dir() {
        Err(EncodingError::ChaptersDirectoryNotFound)?
    }

    // Create the output directory if needed, and get the output path
    let output = match c.output {
        Some(output) => {
            match c.method {
                Method::Compile(_) | Method::Individual => {
                    if !output.is_dir() {
                        if c.create_output_dir {
                            fs::create_dir_all(output).map_err(EncodingError::FailedToCreateOutputDirectory)?
                        } else {
                            Err(EncodingError::OutputDirectoryNotFound)?
                        }
                    }
                },

                Method::Single => {
                    if output.is_dir() {
                        Err(EncodingError::OutputFileIsADirectory)?
                    }

                    let file_name = output.file_name().unwrap();

                    if let None = file_name.to_str() {
                        Err(EncodingError::OutputFileHasInvalidUTF8Name(file_name.to_os_string()))?
                    }

                    let parent = output.parent().expect("Internal error: failed to get parent directory from output file");

                    if !parent.is_dir() {
                        if c.create_output_dir {
                            fs::create_dir_all(parent).map_err(EncodingError::FailedToCreateOutputDirectory)?
                        } else {
                            Err(EncodingError::OutputDirectoryNotFound)?
                        }
                    }
                }
            }

            output.to_path_buf()
        },

        None => match c.method {
            // Output directory = input directory
            Method::Compile(_) | Method::Individual => chapters_dir.to_path_buf(),
            // Output directory = input file with ".cbz" extension
            Method::Single => chapters_dir.join(Path::new(chapters_dir.file_name().unwrap_or(&OsString::from("root"))).with_extension("cbz"))
        }
    };

    // List of chapter directories
    let mut chapter_dirs: Vec<(PathBuf, String)> = vec![];

    if c.root_chapter {
        let chap_name = chapters_dir.file_name().unwrap();
        chapter_dirs.push((
            chapters_dir.to_path_buf(),
            chap_name.to_str().ok_or(EncodingError::ItemHasInvalidUTF8Name(chap_name.to_os_string()))?.to_owned()
        ))
    } else {
        trace!("Reading chapter directories...");

        // Iterate over all items in the input directory
        for entry in fs::read_dir(chapters_dir).map_err(EncodingError::FailedToReadChaptersDirectory)? {
            let entry = entry.map_err(EncodingError::FailedToReadChaptersDirectory)?;
            let path = entry.path();

            // Ignore files
            if path.is_dir() {
                let entry_name = entry.file_name().into_string().map_err(|_| EncodingError::ItemHasInvalidUTF8Name(entry.file_name()))?;

                // Ignore directories not starting by the provided prefix
                if c.dirs_prefix.map(|prefix| entry_name.starts_with(prefix)).unwrap_or(true) {
                    chapter_dirs.push((path, entry_name));
                }
            }
        }

        trace!("Sorting chapter directories by name...");

        if c.disable_nat_sort {
            chapter_dirs.sort_by(|a, b| a.0.cmp(&b.0));
        } else {
            chapter_dirs.sort_by(|a, b| lib::natural_paths_cmp(&a.0, &b.0));
        }
    }

    // Disable mutability for this variable
    let chapter_dirs = chapter_dirs;

    // Current volume
    let mut volume = 1;

    // List of chapter directories of the current volume
    let mut volume_chapters = vec![];

    // First chapter of current volume
    let mut volume_start_chapter = 1;

    // Number of volumes to make, before considering start and end chapter
    // It is used to determine the number of digits volumes should be displayed with
    let untrimmed_volumes = lib::ceil_div(chapter_dirs.len(), chap_per_vol.into());
    let vol_num_len = untrimmed_volumes.to_string().len();

    // Determine the number of digits for chapters
    let chapter_num_len = chapter_dirs.len().to_string().len();

    let start_chapter = c.start_chapter.unwrap_or(1) - 1;

    let end_chapter = match c.method {
        Method::Compile(_) | Method::Individual => c.end_chapter.unwrap_or(chapter_dirs.len()),

        // End chapter cannot exceed the number of chapters per volume
        Method::Single => std::cmp::min(c.end_chapter.unwrap_or(chapter_dirs.len()), start_chapter + usize::from(chap_per_vol))
    };

    // End chapter cannot exceed the number of existing chapter directories minus the start chapter
    let end_chapter = std::cmp::min(end_chapter, chapter_dirs.len() - start_chapter);

    if end_chapter == 0 {
        warn!("No chapter found. Nothing to do.");
        return Ok(vec![]);
    }

    // Determine the real number of chapters to encode
    let chapter_len = end_chapter - start_chapter;

    // Determine the real number of volumes to create
    let volumes = lib::ceil_div(chapter_len, chap_per_vol.into());

    if !is_rebuilding {
        info!(
            "Going to treat chapter{} {} to {} ({} out of {}, {} to ignore) into {} volume{}.",
            if chapter_len > 0 { "s" } else { "" },
            start_chapter + 1,
            end_chapter,
            chapter_len,
            chapter_dirs.len(),
            chapter_dirs.len() - chapter_len,
            volumes,
            if volumes > 1 { "s" } else { "" }
        );
    }

    trace!("Building chapters list for all volumes...");

    // The list of all created volume files
    let mut output_files = vec![];

    // Iterate over chapters
    for (chapter, (path, chapter_name)) in chapter_dirs.into_iter().skip(start_chapter).take(chapter_len).enumerate() {
        // Add this chapter to the current volume
        volume_chapters.push((chapter + 1, path, chapter_name));

        // If this volume contains enough chapters, build it
        if volume_chapters.len() == chap_per_vol.into() {
            output_files.push(build(c, is_rebuilding, &output, volume, volumes, vol_num_len, chapter_num_len, volume_start_chapter, &volume_chapters)?);
            volume_start_chapter += volume_chapters.len();
            volume_chapters = vec![];
            volume += 1;
        }
    }

    // If there are remaining chapters, build a last volume with them
    if volume_chapters.len() != 0 {
        output_files.push(build(c, is_rebuilding, &output, volume, volumes, vol_num_len, chapter_num_len, volume_start_chapter, &volume_chapters)?);
    }

    // Only 1 volume should be created when building single volume
    if c.method == Method::Single {
        assert!(output_files.len() <= 1, "Internal error: more than 1 volume was produced for a single output file.");
    }

    if !is_rebuilding {
        info!("Successfully built {} volume{}.", output_files.len(), if output_files.len() > 1 { "s" } else { "" });
    }

    Ok(output_files)
}

/// Perform an encoding using the provided command-line arguments
pub fn from_args(args: &ArgMatches) -> Result<Vec<PathBuf>, EncodingError> {
    // Determine the encoding method
    let method = if let Some(chapters_per_vol) = args.value_of("compile") {
        Method::Compile(str::parse::<u16>(chapters_per_vol).map_err(|_| EncodingError::InvalidNumberOfChaptersPerVolume)?)
    } else if args.is_present("individual") {
        Method::Individual
    } else if args.is_present("single") {
        Method::Single
    } else {
        unreachable!()
    };

    // Perform the encoding
    encode(&Config {
        method,
        chapters_dir: Path::new(args.value_of("chapters-dir").unwrap()),
        output: args.value_of("output").map(Path::new),
        chapters_suffix: args.is_present("chapters-suffix"),
        create_output_dir: args.is_present("create-output-dir"),
        overwrite: args.is_present("overwrite"),
        skip_existing: args.is_present("skip-existing"),
        dirs_prefix: args.value_of("dirs-prefix"),
        start_chapter: args.value_of("start-chapter").map(str::parse::<usize>).transpose().map_err(|_| EncodingError::InvalidStartChapter)?,
        end_chapter: args.value_of("end-chapter").map(str::parse::<usize>).transpose().map_err(|_| EncodingError::InvalidEndChapter)?,
        root_chapter: args.is_present("root-chapter"),
        extended_image_formats: args.is_present("extended-image-formats"),
        disable_nat_sort: args.is_present("disable-natural-sorting"),
        show_chapters_path: args.is_present("show-chapters-path"),
        display_full_names: args.is_present("display-full-names"),
        compress_losslessly: args.is_present("compress-losslessly")
    }, false)
}
