use std::ffi::OsString;
use std::io::Error as IOError;
use std::path::PathBuf;
use std::fmt;
use zip::result::ZipError;

/// Global CLI error
pub enum GlobalError {
    ActionNameIsMissing,
    UnknownAction(String)
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::ActionNameIsMissing => "Please provide an action to perform. Use '--help' to see the list of available commands.".to_owned(),
            Self::UnknownAction(name) => format!("Unknown action '{}'. Use '--help' to see the list of available commands.", name)
        })
    }
}

/// Error during in the "volumify" action
pub enum VolumifyError {
    MissingOutputPath,
    InvalidNumberOfChaptersPerVolume,
    InvalidStartChapter,
    InvalidEndChapter,
    AtLeast1ChapterPerVolume,
    StartChapterCannotBeHigherThanEndChapter,
    ChaptersDirectoryNotFound,
    OutputDirectoryNotFound,
    OutputFileHasInvalidUTF8Name(OsString),
    OutputFileIsADirectory,
    FailedToCreateOutputDirectory(IOError),
    FailedToReadChaptersDirectory(IOError),
    ItemHasInvalidUTF8Name(OsString),
    FailedToCreateVolumeFile(usize, IOError),
    FailedToListChapterDirectoryFiles { volume: usize, chapter: usize, chapter_path: PathBuf, err: IOError },
    FailedToOpenImage { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToCreateChapterDirectoryInZip { volume: usize, chapter: usize, dir_name: String, err: ZipError },
    FailedToCreateImageFileInZip { volume: usize, chapter: usize, file_path: PathBuf, err: ZipError },
    FailedToReadImage { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToWriteImageFileToZip { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToCloseZipArchive(usize, ZipError)
}

impl fmt::Display for VolumifyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::MissingOutputPath =>
                "Please provide an output path".to_owned(),

            Self::InvalidNumberOfChaptersPerVolume =>
                "Please provide a valid number of chapters per volume (integer, strictly higher than 0)".to_owned(),

            Self::InvalidStartChapter =>
                "Please provide a valid start chapter (integer, strictly higher than 0)".to_owned(),

            Self::InvalidEndChapter =>
                "Please provide a valid end chapter (integer, strictly higher than 0)".to_owned(),

            Self::StartChapterCannotBeHigherThanEndChapter =>
                "Start chapter cannot be higher than the end chapter".to_owned(),

            Self::AtLeast1ChapterPerVolume =>
                "There must be at least 1 chapter per volume".to_owned(),

            Self::ChaptersDirectoryNotFound =>
                "Chapters directory was not found".to_owned(),
            
            Self::OutputDirectoryNotFound =>
                "Output directory was not found".to_owned(),

            Self::OutputFileHasInvalidUTF8Name(name) =>
                format!("Output file has not a valid UTF-8 name ('{}')", name.to_string_lossy()),

            Self::OutputFileIsADirectory =>
                "Output file is a directory".to_owned(),

            Self::FailedToCreateOutputDirectory(err) =>
                format!("Failed to create output directory: {}", err),
            
            Self::FailedToReadChaptersDirectory(err) =>
                format!("Failed to read the chapters directory: {}", err),
            
            Self::ItemHasInvalidUTF8Name(path) =>
                format!("A file or directory has not a valid UTF-8 name in the input directory: {}", path.to_string_lossy()),
            
            Self::FailedToCreateVolumeFile(volume, err) =>
                format!("Failed to create the file of volume {}: {}", volume, err),
            
            Self::FailedToListChapterDirectoryFiles { volume, chapter, chapter_path, err } =>
                format!(
                    "Failed to list files for chapter {} in volume {} at '{}': {}",
                    chapter,
                    volume,
                    chapter_path.to_string_lossy(),
                    err
                ),
            
            Self::FailedToOpenImage { volume, chapter, chapter_path: _, image_path, err } =>
                format!(
                    "Failed to open image file '{}' from chapter {} in volume {}: {}",
                    image_path.to_string_lossy(),
                    chapter,
                    volume,
                    err
                ),
            
            Self::FailedToCreateChapterDirectoryInZip { volume, chapter, dir_name: _, err } =>
                format!("Failed to create directory for chapter {} in volume {}: {}", chapter, volume, err),

            Self::FailedToCreateImageFileInZip { volume, chapter, file_path: _, err } =>
                format!("Failed to create image file for chapter {} in volume {}: {}", chapter, volume, err),

            Self::FailedToReadImage { volume, chapter, chapter_path: _, image_path, err } =>
                format!(
                    "Failed to read image file '{}' from chapter {} in volume {}: {}",
                    image_path.to_string_lossy(),
                    chapter,
                    volume,
                    err
                ),

            Self::FailedToWriteImageFileToZip { volume, chapter, chapter_path: _, image_path, err } =>
                format!(
                    "Failed to write image file '{}' from chapter {} in volume {}: {}",
                    image_path.to_string_lossy(),
                    chapter,
                    volume,
                    err
                ),

            Self::FailedToCloseZipArchive(volume, err) =>
                format!("Failed to close archive for volume {}: {}", volume, err)
        })
    }
}
