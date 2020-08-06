use std::path::PathBuf;
use crate::cli::error::EncodingError;
use crate::cli::opts::{EncodeSingle, EncodingOptions};
use crate::lib::build_vol::{BuildMethod, build_volume};

/// Compile a single directory to a single volume file
pub fn encode_one(opts: &EncodeSingle, enc_opts: &EncodingOptions) -> Result<PathBuf, EncodingError> {
    let input = enc_opts.input.clone();
    
    let output = match &enc_opts.output {
        Some(output) => output.clone(),
        None => {
            let filename = input.file_name().ok_or(EncodingError::SingleInputDirectorHasNoName)?;
            input.join(filename).with_extension("cbz")
        }
    };

    if !input.exists() {
        return Err(EncodingError::SingleInputDirectoryNotFound);
    } else if !input.is_dir() {
        return Err(EncodingError::SingleInputDirectoryIsNotADirectory);
    }

    if output.exists() {
        if output.is_dir() {
            return Err(EncodingError::OutputVolumeFileAlreadyExists(1, input));
        }
    }

    let out_filename = output.file_name().ok_or(EncodingError::SingleOutputFileHasNoName)?;

    build_volume(
        &BuildMethod::Single(opts),
        enc_opts,
        &output,
        1,
        1,
        1,
        1,
        1,
        vec![ (1, input, out_filename.to_string_lossy().to_string()) ]
    )
}
