use anyhow::Result;
use std::process::Command;
use std::{
    env,
    path::{Path, PathBuf},
};

const OS: &str = env::consts::OS;
const ARCH: &str = env::consts::ARCH;

pub fn install_ffmpeg(vcpkg_root: &Path, triplet: &str) -> Result<PathBuf> {
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

    let installed_path = vcpkg_root.join("installed").join(triplet);
    Ok(installed_path)
}
