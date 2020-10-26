use clap::{crate_authors, crate_description, crate_version, Clap};
use std::path::PathBuf;

#[derive(Clap, Debug)]
#[clap(
    name = "Comic Encoder", version = crate_version!(), author = crate_authors!(), about = crate_description!()
)]
pub struct Opts {
    /// Do not display any message other than errors
    #[clap(
        global = true,
        long = "silent",
        conflicts_with = "verbose",
        conflicts_with = "debug"
    )]
    pub silent: bool,

    /// Display detailed informations
    #[clap(
        global = true,
        long = "verbose",
        short,
        conflicts_with = "silent",
        conflicts_with = "debug"
    )]
    pub verbose: bool,

    /// Display extremely detailed informations
    #[clap(
        global = true,
        long = "debug",
        conflicts_with = "silent",
        conflicts_with = "verbose"
    )]
    pub debug: bool,

    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Clap, Debug)]
pub enum Action {
    Encode(Encode),
    Decode(Decode),
}

#[derive(Clap, Debug)]
pub struct Encode {
    /// Encoding method
    #[clap(subcommand)]
    pub method: EncodingMethod,

    #[clap(flatten)]
    pub options: EncodingOptions,
}

#[derive(Clap, Debug)]
/// Encode directories to volumes
pub enum EncodingMethod {
    Compile(CompilationOptions),
    Single(EncodeSingle),
}

#[derive(Clap, Debug)]
pub struct EncodingOptions {
    /// Path to the directory containing the chapters or the volumes to encode
    #[clap(parse(from_os_str))]
    pub input: PathBuf,

    /// Path to the directory where the volumes should be put or to the single volume
    #[clap(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Overwrite existing files instead of failing
    #[clap(global = true, long)]
    pub overwrite: bool,

    /// Add the number of pages at the end of each volume's filename
    #[clap(global = true, long)]
    pub append_pages_count: bool,

    /// Allow additional image formats that may not be supported by all readers (e.g. TIF / RAW / CR2 / ... files)
    #[clap(global = true, short, long)]
    pub accept_extended_image_formats: bool,

    /// Disable natural sorting for pictures (use default UTF-8 sorting, a bit faster but unintuitive)
    #[clap(global = true, short, long)]
    pub simple_sorting: bool,

    /// Compress losslessly (a lot slower, save up about 5% of the final volumes' size)
    #[clap(global = true, long)]
    pub compress_losslessly: bool,
}

#[derive(Clap, Debug, Clone)]
/// Compile chapter directories into volumes
pub struct CompilationOptions {
    #[clap(subcommand)]
    pub method: CompilationMethod,

    /// Creates output directory if it does not exist yet
    #[clap(global = true, long)]
    pub create_output_dir: bool,

    /// Prefix in the name of the chapter directories
    #[clap(global = true, short, long)]
    pub dirs_prefix: Option<String>,

    /// Start at a specific chapter/volume (ignore every chapter before this one)
    #[clap(global = true, long)]
    pub start_chapter: Option<usize>,

    /// End at a specific chapter/volume (ignore every chapter after this one)
    #[clap(global = true, long)]
    pub end_chapter: Option<usize>,
}

#[derive(Clap, Debug, Clone, Copy)]
pub enum CompilationMethod {
    Ranges(CompileRanges),
    Each(CompileEach),
}

#[derive(Clap, Debug, Clone, Copy)]
/// Compile multiple chapters in single volumes (e.g. compile 10 to compile 10 chapters per volume)
pub struct CompileRanges {
    #[clap(global = true, about = "Number of chapters per volume")]
    pub chapters_per_volume: u16,

    /// Add the start and end chapter at the end of each volume's filename
    #[clap(global = true, long)]
    pub append_chapters_range: bool,

    /// Show path for each chapter put in a volume
    #[clap(global = true, long)]
    pub debug_chapters_path: bool,
}

#[derive(Clap, Debug, Clone, Copy)]
/// Compile directories to individual volumes
pub struct CompileEach {
    /// Skip output chapter files that already exist
    #[clap(global = true, long, conflicts_with = "append_pages_count")]
    pub skip_existing: bool,

    /// Display full file names (by default names are truncated above 50 characters)
    #[clap(global = true, long)]
    pub display_full_names: bool,
}

#[derive(Clap, Debug, Clone, Copy)]
/// Encode a single directory as a single volume
pub struct EncodeSingle {}

#[derive(Clap, Debug, Clone)]
/// Extract images from an existing comic book
pub struct Decode {
    /// The comic book to decode
    #[clap(parse(from_os_str))]
    pub input: PathBuf,

    /// Directory where images will be written
    #[clap(global = true, short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Creates output directory if it does not exist yet
    #[clap(global = true, long)]
    pub create_output_dir: bool,

    /// Only extract supported image formats
    #[clap(global = true, short, long)]
    pub extract_images_only: bool,

    /// When using '--extract-images-only', extract additional image formats that may not be supported by all readers (e.g. TIF / RAW / CR2 / ... files)
    #[clap(global = true, short, long, requires = "extract-images-only")]
    pub accept_extended_image_formats: bool,

    /// Disable natural sorting (use default UTF-8 sorting, a bit faster but unintuitive)
    #[clap(global = true, short, long)]
    pub simple_sorting: bool,

    /// Continue extraction even if some pages cannot be extracted from the input PDF (only if input file is PDF)
    #[clap(global = true, long)]
    pub skip_bad_pdf_pages: bool,
}
