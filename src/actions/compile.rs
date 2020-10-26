use crate::cli::error::EncodingError;
use crate::cli::opts::{CompilationMethod, CompilationOptions, EncodingOptions};
use crate::lib::build_vol::*;
use crate::lib::deter;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Compile directories to volumes
pub fn compile(
    opts: &CompilationOptions,
    enc_opts: &EncodingOptions,
) -> Result<Vec<PathBuf>, EncodingError> {
    // Get the number of chapters to put in each volume
    let chap_per_vol = match &opts.method {
        CompilationMethod::Ranges(opts) => opts.chapters_per_volume,
        CompilationMethod::Each(_) => 1,
    };

    if chap_per_vol == 0 {
        return Err(EncodingError::AtLeast1ChapterPerVolume);
    }

    if let Some(start_chapter) = opts.start_chapter {
        if start_chapter == 0 {
            return Err(EncodingError::InvalidStartChapter);
        }
    }

    if let Some(end_chapter) = opts.end_chapter {
        if end_chapter == 0 {
            return Err(EncodingError::InvalidEndChapter);
        }
    }

    if let (Some(start_chapter), Some(end_chapter)) = (opts.start_chapter, opts.end_chapter) {
        if end_chapter < start_chapter {
            return Err(EncodingError::StartChapterCannotBeHigherThanEndChapter);
        }
    }

    // Get current directory
    let cwd = env::current_dir().map_err(EncodingError::FailedToGetCWD)?;

    let input_dir = cwd.join(&enc_opts.input);

    if !input_dir.is_dir() {
        return Err(EncodingError::ChaptersDirectoryNotFound);
    }

    // Create the output directory if needed, and get the output path
    let output = match &enc_opts.output {
        Some(output) => {
            let output = cwd.join(output);

            if !output.is_dir() {
                if opts.create_output_dir {
                    fs::create_dir_all(&output)
                        .map_err(EncodingError::FailedToCreateOutputDirectory)?
                } else {
                    return Err(EncodingError::OutputDirectoryNotFound);
                }
            }

            output
        }

        // Output directory = input directory
        None => input_dir.clone(),
    };

    // List of chapter directories
    let mut chapter_dirs: Vec<(PathBuf, String)> = vec![];

    trace!("Reading chapter directories...");

    // Iterate over all items in the input directory
    for entry in fs::read_dir(input_dir).map_err(EncodingError::FailedToReadChaptersDirectory)? {
        let entry = entry.map_err(EncodingError::FailedToReadChaptersDirectory)?;
        let path = entry.path();

        // Ignore files
        if path.is_dir() {
            let entry_name = entry
                .file_name()
                .into_string()
                .map_err(|_| EncodingError::ItemHasInvalidUTF8Name(entry.file_name()))?;

            // Ignore directories not starting by the provided prefix
            if opts
                .dirs_prefix
                .as_ref()
                .map(|prefix| entry_name.starts_with(prefix))
                .unwrap_or(true)
            {
                chapter_dirs.push((path, entry_name));
            }
        }
    }

    trace!("Sorting chapter directories by name...");

    if enc_opts.simple_sorting {
        chapter_dirs.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        chapter_dirs.sort_by(|a, b| deter::natural_paths_cmp(&a.0, &b.0));
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
    let untrimmed_volumes = deter::ceil_div(chapter_dirs.len(), chap_per_vol.into());
    let vol_num_len = untrimmed_volumes.to_string().len();

    // Determine the number of digits for chapters
    let chapter_num_len = chapter_dirs.len().to_string().len();

    let start_chapter = opts.start_chapter.unwrap_or(1) - 1;

    let end_chapter = opts.end_chapter.unwrap_or(chapter_dirs.len());

    // End chapter cannot exceed the number of existing chapter directories minus the start chapter
    let end_chapter = std::cmp::min(end_chapter, chapter_dirs.len() - start_chapter);

    if end_chapter == 0 {
        warn!("No chapter found. Nothing to do.");
        return Ok(vec![]);
    }

    // Determine the real number of chapters to encode
    let chapter_len = end_chapter - start_chapter;

    // Determine the real number of volumes to create
    let volumes = deter::ceil_div(chapter_len, chap_per_vol.into());

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

    trace!("Building chapters list for all volumes...");

    // Generate the build method
    let build_method = match &opts.method {
        CompilationMethod::Ranges(sub_opts) => BuildMethod::Ranges(sub_opts, opts),
        CompilationMethod::Each(sub_opts) => BuildMethod::Each(sub_opts, opts),
    };

    // The list of all created volume files
    let mut output_files = vec![];

    // Iterate over chapters
    for (chapter, (path, chapter_name)) in chapter_dirs
        .into_iter()
        .skip(start_chapter)
        .take(chapter_len)
        .enumerate()
    {
        // Add this chapter to the current volume
        volume_chapters.push((chapter + 1, path, chapter_name));

        // If this volume contains enough chapters, build it
        if volume_chapters.len() == chap_per_vol.into() {
            output_files.push(build_volume(&BuildVolumeArgs {
                method: &build_method,
                enc_opts,
                output: &output,
                volume,
                volumes,
                vol_num_len,
                chapter_num_len,
                start_chapter: volume_start_chapter,
                chapters: &volume_chapters,
            })?);
            volume_start_chapter += volume_chapters.len();
            volume_chapters = vec![];
            volume += 1;
        }
    }

    // If there are remaining chapters, build a last volume with them
    if volume_chapters.is_empty() {
        output_files.push(build_volume(&BuildVolumeArgs {
            method: &build_method,
            enc_opts,
            output: &output,
            volume,
            volumes,
            vol_num_len,
            chapter_num_len,
            start_chapter: volume_start_chapter,
            chapters: &volume_chapters,
        })?);
    }

    info!(
        "Successfully built {} volume{}.",
        output_files.len(),
        if output_files.len() > 1 { "s" } else { "" }
    );

    Ok(output_files)
}
