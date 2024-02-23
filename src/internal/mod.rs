pub mod util;

use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use anyhow::{anyhow, bail, Result};
use lapce_plugin::{Http, VoltEnvironment};
use zip::ZipArchive;

use crate::{error, string};

const DENO_VERSION: &str = "1.41.0";

pub(crate) fn download() -> Result<&'static str> {
    let filename = match VoltEnvironment::operating_system().as_deref() {
        Ok("windows") => "deno.exe",
        _ => "deno",
    };

    if PathBuf::from(filename).exists() {
        return Ok(filename);
    }

    // let mut response = Http::get("https://api.github.com/repos/denoland/deno/releases/latest")?;
    // if response.status_code.is_success() {
    //     let body = response.body_read_all()?;
    //     let release = serde_json::from_slice(&body)?;
    //     release
    // }

    let filename = match (
        VoltEnvironment::operating_system().as_deref(),
        VoltEnvironment::architecture().as_deref(),
        VoltEnvironment::libc().as_deref(),
    ) {
        (Ok("macos"), Ok("x86_64"), _) => "x86_64-apple-darwin",
        (Ok("macos"), Ok("aarch64"), _) => "aarch64-apple-darwin",
        (Ok("linux"), Ok("x86_64"), Ok("glibc")) => "x86_64-unknown-linux-gnu",
        (Ok("windows"), Ok("x86_64"), _) => "x86_64-pc-windows-msvc",
        _ => bail!("Unsupported OS/Arch/Libc"),
    };

    let zip_file = format!("deno-{filename}.zip");

    // Download URL
    let download_url =
        format!("https://github.com/denoland/deno/releases/download/v{DENO_VERSION}/{zip_file}");

    let Ok(mut resp) = Http::get(&download_url) else {
        bail!("Failed to request download artifact!");
    };
    let Ok(body) = resp.body_read_all() else {
        bail!("Failed to read download artifact");
    };
    let Ok(_) = fs::write(&zip_file, body) else {
        error!(string!("Failed to write artifact to filesystem!"));
        bail!("Failed to write artifact to filesystem!");
    };

    let f = match File::open(&zip_file) {
        Ok(f) => f,
        Err(e) => {
            error!(format!(
                "Failed to open clangd download artifact! L: {} C: {} e: {e}",
                line!(),
                column!()
            ));
            bail!("Failed to open clangd download artifact!");
        }
    };

    let mut zip = match ZipArchive::new(f) {
        Ok(v) => v,
        Err(e) => {
            error!(format!(
                "Failed to create zip overlay! L: {} C: {} e: {e}",
                line!(),
                column!()
            ));
            bail!("Failed to create zip overlay!");
        }
    };

    for i in 0..zip.len() {
        let mut file = match zip.by_index(i) {
            Ok(v) => v,
            Err(e) => {
                error!(format!(
                    "Failed to get file in zip '{i}'! L: {} C: {} e: {e}",
                    line!(),
                    column!()
                ));
                continue;
            }
        };
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                error!(format!(
                    "Failed to create directory path '{}'! L: {} C: {} e: {e}",
                    outpath.display(),
                    line!(),
                    column!()
                ));
            };
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p) {
                        error!(format!(
                            "Failed to create directory path '{}'! L: {} C: {} e: {e}",
                            p.display(),
                            line!(),
                            column!()
                        ));
                    };
                }
            }

            let outfile = match File::create(&outpath) {
                Ok(v) => Some(v),
                Err(e) => {
                    error!(format!(
                        "Failed to open file '{}' to write! L: {} C: {} e: {e}",
                        outpath.display(),
                        line!(),
                        column!()
                    ));
                    None
                }
            };

            if let Some(mut outfile) = outfile {
                if let Err(e) = io::copy(&mut file, &mut outfile) {
                    error!(format!(
                        "Failed to write file '{}'! L: {} C: {} e: {e}",
                        outpath.display(),
                        line!(),
                        column!()
                    ));
                    return Err(anyhow!("Failed to create zip overlay!"));
                };
            }
        }
    }

    if let Err(e) = fs::remove_file(&zip_file) {
        error!(format!(
            "Failed to remove download artifact! L: {} C: {} e: {e}",
            line!(),
            column!()
        ));
    };

    Ok(filename)
}
