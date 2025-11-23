mod ffmpeg;
mod pdfium;

use anyhow::Result;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::ffmpeg::install_ffmpeg;
use crate::pdfium::install_pdfium;

const OS: &str = env::consts::OS;
const ARCH: &str = env::consts::ARCH;

fn main() -> Result<()> {
    println!(">>> [jevy-binaries] Starting FFmpeg Build Factory...");
    println!(">>> Target System: {} ({})", OS, ARCH);

    let triplet = determine_triplet();
    println!(">>> Vcpkg Triplet: {}", triplet);
    let vcpkg_root = prepare_vcpkg(&triplet)?;

    let ffmpeg_artifact = install_ffmpeg(&vcpkg_root, &triplet)?;
    let pdfium_artifact = install_pdfium(&vcpkg_root)?;

    harvest_artifacts(vec![
        ffmpeg_artifact,
        pdfium_artifact,
    ])?;

    println!(">>> Complete!");
    Ok(())
}

fn determine_triplet() -> String {
    match OS {
        "macos" => match ARCH {
            "aarch64" => "arm64-osx-static".to_string(),
            "x86_64" => "x64-osx-static".to_string(),
            _ => panic!("Unsupported macOS architecture: {}", ARCH),
        },
        "windows" => match ARCH {
            "aarch64" => "arm64-windows-static-md".to_string(),
            "x86_64" => "x64-windows-static-md".to_string(),
            _ => panic!("Unsupported Windows architecture: {}", ARCH),
        },
        "linux" => match ARCH {
            "x86_64" => "x64-linux-static".to_string(),
            _ => panic!("Unsupported Linux architecture: {}", ARCH),
        },
        _ => panic!("Unsupported OS: {}", OS),
    }
}

fn prepare_vcpkg(triplet: &str) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let vcpkg_dir = current_dir.join("vcpkg");

    if !vcpkg_dir.exists() {
        println!(">>> Cloning vcpkg...");
        let status = Command::new("git")
            .args(&["clone", "https://github.com/microsoft/vcpkg.git"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to clone vcpkg");
        }
    }

    let script = match OS {
        "windows" => "bootstrap-vcpkg.bat",
        _ => "./bootstrap-vcpkg.sh",
    };
    println!(">>> Bootstrapping vcpkg...");
    let status = Command::new(script).current_dir(&vcpkg_dir).status()?;
    if !status.success() {
        anyhow::bail!("Failed to bootstrap vcpkg");
    }

    if OS == "windows" {
        return Ok(vcpkg_dir);
    }
    let triplets_dir = vcpkg_dir.join("triplets");
    if !triplets_dir.exists() {
        fs::create_dir_all(&triplets_dir)?;
    }

    let triplet_file = triplets_dir.join(format!("{}.cmake", triplet));

    let vcpkg_arch = match ARCH {
        "aarch64" | "arm64" => "arm64",
        "x86_64" | "x64" => "x86_64",
        _ => panic!("Unsupported architecture: {}", ARCH),
    };
    let vcpkg_system = match OS {
        "macos" => "Darwin",
        "linux" => "Linux",
        _ => panic!("Unsupported OS for triplet creation: {}", OS),
    };
    let content = format!(
        "set(VCPKG_TARGET_ARCHITECTURE {})\n\
         set(VCPKG_CRT_LINKAGE dynamic)\n\
         set(VCPKG_LIBRARY_LINKAGE static)\n\
         set(VCPKG_CMAKE_SYSTEM_NAME {})\n",
        vcpkg_arch, vcpkg_system
    );
    fs::write(triplet_file, content)?;
    println!(">>> Created custom triplet: {}", triplet);

    Ok(vcpkg_dir)
}

// fn create_custom_triplet(vcpkg_root: &Path, triplet: &str) -> Result<()> {
//     if OS == "windows" {
//         return Ok(());
//     }

//     let triplets_dir = vcpkg_root.join("triplets");
//     if !triplets_dir.exists() {
//         fs::create_dir_all(&triplets_dir)?;
//     }

//     let triplet_file = triplets_dir.join(format!("{}.cmake", triplet));

//     // let content = if OS == "macos" {
//     //     let arch = if triplet.starts_with("arm64") { "arm64" } else { "x86_64" };
//     //     format!(
//     //         "set(VCPKG_TARGET_ARCHITECTURE {})\n\
//     //          set(VCPKG_CRT_LINKAGE dynamic)\n\
//     //          set(VCPKG_LIBRARY_LINKAGE static)\n\
//     //          set(VCPKG_CMAKE_SYSTEM_NAME Darwin)\n",
//     //         arch
//     //     )
//     // } else if OS == "linux" {
//     //     "set(VCPKG_TARGET_ARCHITECTURE x64)\n\
//     //      set(VCPKG_CRT_LINKAGE dynamic)\n\
//     //      set(VCPKG_LIBRARY_LINKAGE static)\n\
//     //      set(VCPKG_CMAKE_SYSTEM_NAME Linux)\n".to_string()
//     // } else {
//     //     String::new()
//     // };

//     // if !content.is_empty() {
//     //     fs::write(triplet_file, content)?;
//     //     println!(">>> Created custom triplet: {}", triplet);
//     // }
//     let content = format!(
//         "set(VCPKG_TARGET_ARCHITECTURE {})\n\
//          set(VCPKG_CRT_LINKAGE dynamic)\n\
//          set(VCPKG_LIBRARY_LINKAGE static)\n\
//          set(VCPKG_CMAKE_SYSTEM_NAME Darwin)\n",
//         ARCH
//     );
//     fs::write(triplet_file, content)?;
//     println!(">>> Created custom triplet: {}", triplet);

//     Ok(())
// }


fn harvest_artifacts(artifacts: Vec<(String, PathBuf)>) -> Result<()> {
    let dist_root = Path::new("dist");
    let options = fs_extra::dir::CopyOptions::new()
        .overwrite(true)
        .content_only(true);

    for (name, src_path) in artifacts {
        let output_dir = dist_root.join(&name);

        if output_dir.exists() {
            fs::remove_dir_all(&output_dir)?;
        }
        fs::create_dir_all(&output_dir)?;

        println!(
            ">>> Harvesting {} artifacts from {:?} to {:?}",
            name, src_path, output_dir
        );

        fs_extra::dir::copy(&src_path, &output_dir, &options)?;
    }

    Ok(())
}

