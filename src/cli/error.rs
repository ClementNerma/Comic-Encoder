use std::ffi::OsString;
use std::io::Error as IOError;
use std::path::PathBuf;
use std::fmt;
use zip::result::ZipError;
use pdf::error::PdfError;

/// Error during in the "encode" action
pub enum EncodingError {
    MissingOutputPath,
    InvalidNumberOfChaptersPerVolume,
    InvalidStartChapter,
    InvalidEndChapter,
    AtLeast1ChapterPerVolume,
    StartChapterCannotBeHigherThanEndChapter,
    FailedToGetCWD(IOError),
    ChaptersDirectoryNotFound,
    OutputDirectoryNotFound,
    OutputFileHasInvalidUTF8Name(OsString),
    SingleInputDirectoryNotFound,
    SingleInputDirectoryIsNotADirectory,
    SingleInputDirectorHasNoName,
    SingleOutputFileHasNoName,
    FailedToCreateOutputDirectory(IOError),
    FailedToReadChaptersDirectory(IOError),
    ItemHasInvalidUTF8Name(OsString),
    FailedToCreateVolumeFile(usize, IOError),
    OutputVolumeFileAlreadyExists(usize, PathBuf),
    OutputVolumeFileIsADirectory(usize, PathBuf),
    FailedToOverwriteOutputVolumeFile(usize, PathBuf, IOError),
    FailedToListChapterDirectoryFiles { volume: usize, chapter: usize, chapter_path: PathBuf, err: IOError },
    FailedToOpenImage { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToCreateChapterDirectoryInZip { volume: usize, chapter: usize, dir_name: String, err: ZipError },
    FailedToCreateImageFileInZip { volume: usize, chapter: usize, file_path: PathBuf, err: ZipError },
    FailedToReadImage { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToWriteImageFileToZip { volume: usize, chapter: usize, chapter_path: PathBuf, image_path: PathBuf, err: IOError },
    FailedToCloseZipArchive(usize, ZipError),
    FailedToRenameCompleteArchive(usize, IOError)
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::MissingOutputPath =>
                "Please provide an output path".to_string(),

            Self::InvalidNumberOfChaptersPerVolume =>
                "Please provide a valid number of chapters per volume (integer, strictly higher than 0)".to_string(),

            Self::InvalidStartChapter =>
                "Please provide a valid start chapter (integer, strictly higher than 0)".to_string(),

            Self::InvalidEndChapter =>
                "Please provide a valid end chapter (integer, strictly higher than 0)".to_string(),

            Self::AtLeast1ChapterPerVolume =>
                "There must be at least 1 chapter per volume".to_string(),

            Self::StartChapterCannotBeHigherThanEndChapter =>
                "Start chapter cannot be higher than the end chapter".to_string(),

            Self::FailedToGetCWD(err) =>
                format!("Failed to get current working directory: {}", err),

            Self::ChaptersDirectoryNotFound =>
                "Chapters directory was not found".to_string(),
            
            Self::OutputDirectoryNotFound =>
                "Output directory was not found".to_string(),

            Self::OutputFileHasInvalidUTF8Name(name) =>
                format!("Output file does not have a valid UTF-8 name ('{}')", name.to_string_lossy()),

            Self::SingleInputDirectoryNotFound =>
                "Input directory was not found".to_string(),

            Self::SingleInputDirectoryIsNotADirectory =>
                "Input directory is not a directory".to_string(),

            Self::SingleInputDirectorHasNoName =>
                "Input directory has no name, so an output name cannot be inferred".to_string(),

            Self::SingleOutputFileHasNoName =>
                "Output file does not have a valid name (e.g. '.' or '/')".to_string(),

            Self::FailedToCreateOutputDirectory(err) =>
                format!("Failed to create output directory: {}", err),
            
            Self::FailedToReadChaptersDirectory(err) =>
                format!("Failed to read the chapters directory: {}", err),
            
            Self::ItemHasInvalidUTF8Name(path) =>
                format!("A file or directory has not a valid UTF-8 name in the input directory: {}", path.to_string_lossy()),
            
            Self::FailedToCreateVolumeFile(volume, err) =>
                format!("Failed to create the file of volume {}: {}", volume, err),
            
            Self::OutputVolumeFileAlreadyExists(volume, path) =>
                format!("Failed to create the file of volume {} because path '{}' already exists (use '--overwrite' to force writing)", volume, path.to_string_lossy()),

            Self::OutputVolumeFileIsADirectory(volume, path) =>
                format!("Failed to create the file of volume {} because path '{}' is a directory", volume, path.to_string_lossy()),

            Self::FailedToOverwriteOutputVolumeFile(volume, path, err) =>
                format!("Failed to overwrite the file of volume {} at path '{}': {}", volume, path.to_string_lossy(), err),

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
                format!("Failed to close archive for volume {}: {}", volume, err),

            Self::FailedToRenameCompleteArchive(volume, err) =>
                format!("Failed to rename complete archive for volume {}: {}", volume, err)
        })
    }
}

/// Error during in the "decode" action
pub enum DecodingError {
    InputFileNotFound,
    InputFileIsADirectory,
    OutputDirectoryNotFound,
    FailedToGetCWD(IOError),
    FailedToCreateOutputDirectory(IOError),
    OutputDirectoryIsAFile,
    InputFileHasInvalidUTF8FileExtension(OsString),
    UnsupportedFormat(String),
    FailedToOpenZipFile(IOError),
    InvalidZipArchive(ZipError),
    ZipError(ZipError),
    ZipFileHasInvalidUTF8FileExtension(PathBuf),
    FailedToCreateOutputFile(IOError, PathBuf),
    FailedToExtractZipFile { path_in_zip: PathBuf, extract_to: PathBuf, err: IOError },
    FailedToRenameTemporaryFile { from: PathBuf, to: PathBuf, err: IOError },
    FailedToOpenPdfFile(PdfError),
    FailedToGetPdfPage(usize, PdfError),
    FailedToGetPdfPageResources(usize, PdfError),
    FailedToExtractPdfImage(usize, PathBuf, IOError)
}

impl fmt::Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::InputFileNotFound =>
                "Input file was not found".to_string(),

            Self::InputFileIsADirectory =>
                "Input file is a directory".to_string(),

            Self::OutputDirectoryNotFound =>
                "Output directory was not found".to_string(),

            Self::FailedToGetCWD(err) =>
                format!("Failed to get current working directory: {}", err),

            Self::FailedToCreateOutputDirectory(err) =>
                format!("Failed to create output directory: {}", err),

            Self::OutputDirectoryIsAFile =>
                "Output directory is a file".to_string(),

            Self::InputFileHasInvalidUTF8FileExtension(path) =>
                format!("Input file has invalid UTF-8 file extension ('{}')", path.to_string_lossy()),

            Self::UnsupportedFormat(ext) =>
                format!("Unsupported image format (based on file extension) '{}'", ext),

            Self::FailedToOpenZipFile(err) =>
                format!("Failed to open input ZIP file: {}", err),

            Self::InvalidZipArchive(err) =>
                format!("Invalid ZIP archive: {}", err),

            Self::ZipError(err) =>
                format!("Error while reading ZIP archive: {}", err),

            Self::ZipFileHasInvalidUTF8FileExtension(path) =>
                format!("A ZIP file has an invalid UTF-8 file extension ('{}')", path.to_string_lossy()),

            Self::FailedToCreateOutputFile(err, path) =>
                format!("Failed to create output file '{}': {}", path.to_string_lossy(), err),

            Self::FailedToExtractZipFile { path_in_zip, extract_to, err } =>
                format!("Failed to extract ZIP file '{}' to '{}': {}", path_in_zip.to_string_lossy(), extract_to.to_string_lossy(), err),

            Self::FailedToRenameTemporaryFile { from, to, err } =>
                format!("Failed to rename temporary file '{}' to '{}': {}", from.to_string_lossy(), to.to_string_lossy(), err),

            Self::FailedToOpenPdfFile(err) =>
                format!("Failed to open PDF file: {}", err),

            Self::FailedToGetPdfPage(page, err) =>
                format!("Failed to get PDF page n°{}: {}", page, err),
            
            Self::FailedToGetPdfPageResources(page, err) =>
                format!("Failed to get resources from PDF page n°{}: {}", page, err),

            Self::FailedToExtractPdfImage(page, path, err) =>
                format!("Failed extract PDF image from page n°{} to path '{}': {}", page, path.to_string_lossy(), err)
        })
    }
}
