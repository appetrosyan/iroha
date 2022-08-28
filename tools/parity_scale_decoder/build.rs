//! Build script that auto-updates sample binaries from sources.

#![allow(clippy::restriction)]

use std::{fs, io::Result, path::PathBuf};

use eyre::WrapErr as _;
use iroha_data_model::prelude::*;
use parity_scale_codec::Encode;
use serde::de::DeserializeOwned;

fn main() -> eyre::Result<()> {
    sample_into_binary_file::<Account>("account")
        .wrap_err("Failed to decode sample `account.json`")?;

    sample_into_binary_file::<Domain>("domain")
        .wrap_err("Failed to decode sample `domain.json`")?;

    sample_into_binary_file::<Trigger<FilterBox>>("trigger")
        .wrap_err("Failed to decode sample `trigger`")?;
    Ok(())
}

fn sample_into_binary_file<T>(filename: &str) -> Result<()>
where
    T: Encode + DeserializeOwned,
{
    let mut path_to = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_to.push("samples/");
    path_to.push(filename);

    let mut path_to_json_sample = path_to.clone();
    path_to_json_sample.set_extension("json");

    let mut path_to_binary = path_to.clone();
    path_to_binary.set_extension("bin");

    let buf = fs::read_to_string(path_to_json_sample)?;

    let sample = serde_json::from_str::<T>(buf.as_str())?;

    let buf = sample.encode();

    fs::write(path_to_binary, buf)?;

    Ok(())
}
