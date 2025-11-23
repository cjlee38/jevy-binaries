use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Result;
use flate2::read::GzDecoder;
use serde::Deserialize;
use tar::Archive;

fn main() -> Result<()> {
    println!("🏭 [jevy-binaries] Starting FFmpeg Build Factory...");

    let (os, arch) = detect_os_arch();
    let triplet = determine_triplet(&os, &arch);

    println!("🎯 Target System: {} ({})", os, arch);
    println!("🔧 Vcpkg Triplet: {}", triplet);

    let vcpkg_root = prepare_vcpkg()?;
    create_custom_triplet(&vcpkg_root, &triplet, &os)?;
    run_vcpkg_install(&vcpkg_root, &triplet)?;
    download_and_extract_pdfium(&vcpkg_root, &os, &arch)?;
    harvest_artifacts(&vcpkg_root, &triplet, &os, &arch)?;

    println!("✅ Build & Harvest Complete!");
    Ok(())
}

fn detect_os_arch() -> (String, String) {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string(); // x86_64 or aarch64
    (os, arch)
}

fn determine_triplet(os: &str, arch: &str) -> String {
    match os {
        "macos" => match arch {
            "aarch64" => "arm64-osx-static".to_string(),
            "x86_64" => "x64-osx-static".to_string(),
            _ => panic!("Unsupported macOS architecture: {}", arch),
        },
        "windows" => match arch {
            "aarch64" => "arm64-windows-static-md".to_string(),
            "x86_64" => "x64-windows-static-md".to_string(),
            _ => panic!("Unsupported Windows architecture: {}", arch),
        },
        "linux" => match arch {
            "x86_64" => "x64-linux-static".to_string(),
            _ => panic!("Unsupported Linux architecture: {}", arch),
        },
        _ => panic!("Unsupported OS: {}", os),
    }
}

fn prepare_vcpkg() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    let vcpkg_dir = current_dir.join("vcpkg");

    if !vcpkg_dir.exists() {
        println!("📥 Cloning vcpkg...");
        let status = Command::new("git")
            .args(&["clone", "https://github.com/microsoft/vcpkg.git"])
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to clone vcpkg");
        }
    }

    let vcpkg_exe_name = if cfg!(target_os = "windows") { "vcpkg.exe" } else { "vcpkg" };
    if !vcpkg_dir.join(vcpkg_exe_name).exists() {
        println!("🚀 Bootstrapping vcpkg...");
        let script_name = if cfg!(target_os = "windows") { "bootstrap-vcpkg.bat" } else { "./bootstrap-vcpkg.sh" };
        
        let status = Command::new(script_name)
            .current_dir(&vcpkg_dir)
            .status()?;
        
        if !status.success() {
            anyhow::bail!("Failed to bootstrap vcpkg");
        }
    }

    Ok(vcpkg_dir)
}

fn create_custom_triplet(vcpkg_root: &Path, triplet: &str, os: &str) -> Result<()> {
    if os == "windows" {
        return Ok(());
    }

    let triplets_dir = vcpkg_root.join("triplets");
    if !triplets_dir.exists() {
        fs::create_dir_all(&triplets_dir)?;
    }

    let triplet_file = triplets_dir.join(format!("{}.cmake", triplet));
    
    let content = if os == "macos" {
        let arch = if triplet.starts_with("arm64") { "arm64" } else { "x86_64" };
        format!(
            "set(VCPKG_TARGET_ARCHITECTURE {})\n\
             set(VCPKG_CRT_LINKAGE dynamic)\n\
             set(VCPKG_LIBRARY_LINKAGE static)\n\
             set(VCPKG_CMAKE_SYSTEM_NAME Darwin)\n",
            arch
        )
    } else if os == "linux" {
        "set(VCPKG_TARGET_ARCHITECTURE x64)\n\
         set(VCPKG_CRT_LINKAGE dynamic)\n\
         set(VCPKG_LIBRARY_LINKAGE static)\n\
         set(VCPKG_CMAKE_SYSTEM_NAME Linux)\n".to_string()
    } else {
        String::new()
    };

    if !content.is_empty() {
        fs::write(triplet_file, content)?;
        println!("✨ Created custom triplet: {}", triplet);
    }

    Ok(())
}

fn run_vcpkg_install(vcpkg_root: &Path, triplet: &str) -> Result<()> {
    let vcpkg_exe = if cfg!(target_os = "windows") { "vcpkg.exe" } else { "./vcpkg" };
    
    let package = format!("ffmpeg[core,avcodec,avformat,avfilter,swresample,swscale,dav1d]:{}", triplet);

    println!("📦 Installing packages: {}", package);

    let status = Command::new(vcpkg_exe)
        .current_dir(vcpkg_root)
        .arg("install")
        .arg(&package)
        .arg("--recurse") 
        .status()?;

    if !status.success() {
        anyhow::bail!("vcpkg install failed");
    }

    Ok(())
}

fn harvest_artifacts(vcpkg_root: &Path, triplet: &str, os: &str, arch: &str) -> Result<()> {
    let dist_root = Path::new("dist");
    let target_name = format!("{}-{}", match os { "macos" => "mac", "windows" => "win", _ => "linux" }, match arch { "x86_64" => "x64", _ => "arm64" });

    let lib_ext = if os == "windows" { "lib" } else { "a" };
    let options = fs_extra::dir::CopyOptions::new().overwrite(true).content_only(true);

    // Harvest FFmpeg from vcpkg
    let ffmpeg_output_dir = dist_root.join(format!("ffmpeg-{}", target_name));
    if ffmpeg_output_dir.exists() {
        fs::remove_dir_all(&ffmpeg_output_dir)?;
    }
    fs::create_dir_all(ffmpeg_output_dir.join("lib"))?;
    fs::create_dir_all(ffmpeg_output_dir.join("include"))?;

    let installed_dir = vcpkg_root.join("installed").join(triplet);
    println!("🚜 Harvesting FFmpeg artifacts from {:?} to {:?}", installed_dir, ffmpeg_output_dir);

    let lib_src = installed_dir.join("lib");
    let lib_dst = ffmpeg_output_dir.join("lib");

    for entry in fs::read_dir(lib_src)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == lib_ext {
                let file_name = path.file_name().unwrap();
                fs::copy(&path, lib_dst.join(file_name))?;
            }
        }
    }

    let include_src = installed_dir.join("include");
    let include_dst = ffmpeg_output_dir.join("include");
    fs_extra::dir::copy(&include_src, &include_dst, &options)?;

    // Harvest PDFium
    let pdfium_dir = vcpkg_root.join("pdfium");
    if pdfium_dir.exists() {
        let pdfium_output_dir = dist_root.join(format!("pdfium-{}", target_name));
        if pdfium_output_dir.exists() {
            fs::remove_dir_all(&pdfium_output_dir)?;
        }
        fs::create_dir_all(pdfium_output_dir.join("lib"))?;
        fs::create_dir_all(pdfium_output_dir.join("include"))?;

        println!("🚜 Harvesting PDFium artifacts from {:?} to {:?}", pdfium_dir, pdfium_output_dir);

        let pdfium_lib = pdfium_dir.join("lib");
        if pdfium_lib.exists() {
            let pdfium_lib_dst = pdfium_output_dir.join("lib");
            for entry in fs::read_dir(pdfium_lib)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == lib_ext {
                        let file_name = path.file_name().unwrap();
                        fs::copy(&path, pdfium_lib_dst.join(file_name))?;
                    }
                }
            }
        }

        let pdfium_include = pdfium_dir.join("include");
        if pdfium_include.exists() {
            let pdfium_include_dst = pdfium_output_dir.join("include");
            fs_extra::dir::copy(&pdfium_include, &pdfium_include_dst, &options)?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn determine_pdfium_platform(os: &str, arch: &str) -> String {
    let platform = match os {
        "macos" => "mac",
        "windows" => "win",
        "linux" => "linux",
        _ => panic!("Unsupported OS: {}", os),
    };

    let arch_str = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => panic!("Unsupported architecture: {}", arch),
    };

    format!("pdfium-{}-{}.tgz", platform, arch_str)
}

fn download_and_extract_pdfium(vcpkg_root: &Path, os: &str, arch: &str) -> Result<()> {
    let pdfium_dir = vcpkg_root.join("pdfium");

    if pdfium_dir.exists() {
        println!("📦 PDFium already exists, skipping download");
        return Ok(());
    }

    let asset_name = determine_pdfium_platform(os, arch);
    println!("📥 Downloading PDFium: {}", asset_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("jevy-binaries")
        .build()?;

    let release: GithubRelease = client
        .get("https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest")
        .send()?
        .json()?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!("Asset {} not found in latest release", asset_name))?;

    println!("🔗 Download URL: {}", asset.browser_download_url);

    let response = client.get(&asset.browser_download_url).send()?;
    let bytes = response.bytes()?;

    println!("📦 Downloaded {} bytes, extracting...", bytes.len());

    let cursor = Cursor::new(bytes);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    fs::create_dir_all(&pdfium_dir)?;
    archive.unpack(&pdfium_dir)?;

    println!("✅ PDFium extracted successfully");

    Ok(())
}