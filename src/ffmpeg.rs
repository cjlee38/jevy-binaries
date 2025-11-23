use std::process::Command;
use anyhow::Result;
use std::{env, path::{Path, PathBuf}};

const OS: &str = env::consts::OS;
const ARCH: &str = env::consts::ARCH;

pub fn install_ffmpeg(vcpkg_root: &Path, triplet: &str) -> Result<(String, PathBuf)> {
    let vcpkg_exe = match OS {
        "windows" => "vcpkg.exe",
        _ => "./vcpkg",
    };

    let package = format!(
        "ffmpeg[core,avcodec,avformat,avfilter,swresample,swscale,dav1d]:{}",
        triplet
    );

    println!(">>> Installing packages: {}", package);

    let status = Command::new(vcpkg_exe)
        .current_dir(vcpkg_root)
        .arg("install")
        .arg(&package)
        .arg("--recurse")
        .status()?;

    if !status.success() {
        anyhow::bail!("vcpkg install failed");
    }

    let target_name = format!(
        "ffmpeg-{}-{}",
        match OS {
            "macos" => "mac",
            "windows" => "win",
            _ => "linux",
        },
        match ARCH {
            "x86_64" => "x64",
            _ => "arm64",
        }
    );

    let installed_path = vcpkg_root.join("installed").join(triplet);

    Ok((target_name, installed_path))
}
