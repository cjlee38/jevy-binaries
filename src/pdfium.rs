use anyhow::Result;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tar::Archive;

const OS: &str = env::consts::OS;
const ARCH: &str = env::consts::ARCH;

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn determine_pdfium_platform() -> String {
    let platform = match OS {
        "macos" => "mac",
        "windows" => "win",
        "linux" => "linux",
        _ => panic!("Unsupported OS: {}", OS),
    };

    let arch_str = match ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        _ => panic!("Unsupported architecture: {}", ARCH),
    };

    format!("pdfium-{}-{}.tgz", platform, arch_str)
}

pub fn install_pdfium(vcpkg_root: &Path) -> Result<(String, PathBuf)> {
    let pdfium_dir = vcpkg_root.join("pdfium");

    let target_name = format!(
        "pdfium-{}-{}",
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

    if pdfium_dir.exists() {
        println!(">>> PDFium already exists, skipping download");
        return Ok((target_name, pdfium_dir));
    }

    let asset_name = determine_pdfium_platform();
    println!(">>> Downloading PDFium: {}", asset_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("jevy-binaries/1.0")
        .build()?;

    println!(">>> Fetching latest release from GitHub API...");
    let response = client
        .get("https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest")
        .send()?;

    let status = response.status();
    println!(">>> GitHub API response status: {}", status);

    if !status.is_success() {
        let body = response.text().unwrap_or_else(|_| "<failed to read body>".to_string());
        anyhow::bail!("GitHub API request failed with status {}: {}", status, body);
    }

    let body_text = response.text()?;
    let release: GithubRelease = serde_json::from_str(&body_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse GitHub API response: {}. Body preview: {}", e, &body_text[..body_text.len().min(500)]))?;

    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!("Asset {} not found in latest release", asset_name))?;

    println!(">>> Download URL: {}", asset.browser_download_url);

    let response = client.get(&asset.browser_download_url).send()?;
    let bytes = response.bytes()?;

    println!(">>> Downloaded {} bytes, extracting...", bytes.len());

    let cursor = Cursor::new(bytes);
    let tar = GzDecoder::new(cursor);
    let mut archive = Archive::new(tar);

    fs::create_dir_all(&pdfium_dir)?;
    archive.unpack(&pdfium_dir)?;

    println!(">>> PDFium extracted successfully");

    Ok((target_name, pdfium_dir))
}
