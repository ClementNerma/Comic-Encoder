use crate::cli::opts::{EncodeSingle, EncodingOptions};
use crate::lib::build_vol::{build_volume, BuildMethod};
use crate::{cli::error::EncodingError, lib::build_vol::BuildVolumeArgs};
use std::path::PathBuf;

/// Compile a single directory to a single volume file
pub fn encode_one(
    opts: &EncodeSingle,
    enc_opts: &EncodingOptions,
) -> Result<PathBuf, EncodingError> {
    let input = enc_opts.input.clone();

    let output = match &enc_opts.output {
        Some(output) => output.clone(),
        None => {
            let filename = input
                .file_name()
                .ok_or(EncodingError::SingleInputDirectorHasNoName)?;
            input.join(filename).with_extension("cbz")
        }
    };

    if !input.exists() {
        return Err(EncodingError::SingleInputDirectoryNotFound);
    } else if !input.is_dir() {
        return Err(EncodingError::SingleInputDirectoryIsNotADirectory);
    }

    if output.is_dir() {
        return Err(EncodingError::OutputVolumeFileAlreadyExists(1, input));
    }

    let out_filename = output
        .file_name()
        .ok_or(EncodingError::SingleOutputFileHasNoName)?;

    build_volume(&BuildVolumeArgs {
        method: &BuildMethod::Single(opts),
        enc_opts,
        output: &output,
        volume: 1,
        volumes: 1,
        vol_num_len: 1,
        chapter_num_len: 1,
        start_chapter: 1,
        chapters: &vec![(1, input, out_filename.to_string_lossy().to_string())],
    })
}
