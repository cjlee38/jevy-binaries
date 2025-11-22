// use anyhow::{Context, Result};
// use flate2::read::GzDecoder;
// use std::env;
// use std::fs::{self, File};
// use std::path::PathBuf;
// use std::process::Command;
// use tar::Archive;

fn main() {
    
}
// /// This build.rs is based on ffmpeg-sys-the-third's build script
// /// Original source: https://github.com/zmwangx/rust-ffmpeg-sys/blob/master/build.rs
// /// We only use the build-from-source parts and adapt it for use with ac-ffmpeg
// #[allow(dead_code)]
// mod build_ffmpeg {
//     use std::env;
//     use std::fs::File;
//     use std::io::{self, BufRead, BufReader};
//     use std::path::{Path, PathBuf};
//     use std::process::Command;

//     pub fn build_ffmpeg() {
//         println!("cargo:warning=Building FFmpeg from source...");

//         let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
//         let ffmpeg_version = "7.1";

//         match build(&out_dir, ffmpeg_version) {
//             Ok(install_dir) => {
//                 println!(
//                     "cargo:warning=FFmpeg built successfully at {:?}",
//                     install_dir
//                 );

//                 // [MODIFIED FOR ac-ffmpeg]
//                 // Tell ac-ffmpeg where to find our built FFmpeg
//                 println!(
//                     "cargo:rustc-env=FFMPEG_INCLUDE_DIR={}",
//                     install_dir.join("include").display()
//                 );
//                 println!(
//                     "cargo:rustc-env=FFMPEG_LIB_DIR={}",
//                     install_dir.join("lib").display()
//                 );
//                 println!("cargo:rustc-env=FFMPEG_STATIC=1");

//                 println!(
//                     "cargo:rustc-link-search=native={}",
//                     install_dir.join("lib").display()
//                 );

//                 // [MODIFIED FOR ac-ffmpeg]
//                 // Original ffmpeg-sys-the-third links FFmpeg libraries here via link_to_libraries()
//                 // but ac-ffmpeg already does that, so we skip it and only link system frameworks
//                 if cfg!(target_os = "macos") {
//                     // let frameworks = vec![
//                     //     "AppKit",
//                     //     "AudioToolbox",
//                     //     "AVFoundation",
//                     //     "CoreFoundation",
//                     //     "CoreGraphics",
//                     //     "CoreMedia",
//                     //     "CoreServices",
//                     //     "CoreVideo",
//                     //     "Foundation",
//                     //     "OpenCL",
//                     //     "OpenGL",
//                     //     "QTKit",
//                     //     "QuartzCore",
//                     //     "Security",
//                     //     "VideoDecodeAcceleration",
//                     //     "VideoToolbox",
//                     // ];

//                     // [MODIFIED FOR ac-ffmpeg] Subset of frameworks needed for our minimal build
//                     for framework in &[
//                         "CoreFoundation",
//                         "CoreMedia",
//                         "CoreVideo",
//                         "CoreServices",
//                         "VideoToolbox",
//                         "AudioToolbox",
//                         "OpenGL",
//                         "Metal",
//                         "CoreImage",
//                         "AppKit",
//                         "Accelerate",
//                     ] {
//                         println!("cargo:rustc-link-lib=framework={}", framework);
//                     }
//                 }
//             }
//             Err(e) => {
//                 panic!("Failed to build FFmpeg: {}", e);
//             }
//         }
//     }

//     fn fetch(source_dir: &Path, ffmpeg_version: &str) -> io::Result<()> {
//         let _ = std::fs::remove_dir_all(source_dir);
//         let status = Command::new("git")
//             .arg("clone")
//             .arg("--depth=1")
//             .arg("-b")
//             .arg(format!("n{ffmpeg_version}"))
//             .arg("https://github.com/FFmpeg/FFmpeg")
//             .arg(source_dir)
//             .status()?;

//         if status.success() {
//             Ok(())
//         } else {
//             Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
//         }
//     }

//     fn build(out_dir: &Path, ffmpeg_version: &str) -> io::Result<PathBuf> {
//         let source_dir = out_dir.join(format!("ffmpeg-{ffmpeg_version}"));
//         let install_dir = out_dir.join("dist");

//         // Skip if already built (same as original line 505-508)
//         if install_dir.join("lib").join("libavutil.a").exists() {
//             rustc_link_extralibs(&source_dir);
//             return Ok(install_dir);
//         }

//         fetch(&source_dir, ffmpeg_version)?;

//         // Command's path is not relative to command's current_dir
//         let configure_path = source_dir.join("configure");
//         assert!(configure_path.exists());
//         let mut configure = Command::new(&configure_path);
//         configure.current_dir(&source_dir);

//         configure.arg(format!("--prefix={}", install_dir.to_string_lossy()));

//         // [REMOVED] Cross-compilation support (we only do native builds)
//         /*
//         if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
//             // Rust targets are subtly different than naming scheme for compiler prefixes.
//             // The cc crate has the messy logic of guessing a working prefix,
//             // and this is a messy way of reusing that logic.
//             let cc = cc::Build::new();
//             let compiler = cc.get_compiler();
//             let compiler = compiler.path().file_stem().unwrap().to_str().unwrap();
//             let suffix_pos = compiler.rfind('-').unwrap(); // cut off "-gcc"
//             let prefix = compiler[0..suffix_pos].trim_end_matches("-wr"); // "wr-c++" compiler

//             configure.arg(format!("--cross-prefix={}-", prefix));
//             configure.arg(format!(
//                 "--arch={}",
//                 env::var("CARGO_CFG_TARGET_ARCH").unwrap()
//             ));
//             configure.arg(format!(
//                 "--target_os={}",
//                 env::var("CARGO_CFG_TARGET_OS").unwrap()
//             ));
//         }
//          */

//         // [REMOVED] Debug/release control (we use release by default)
//         /*
//         // control debug build
//         if env::var("DEBUG").is_ok() {
//             configure.arg("--enable-debug");
//             configure.arg("--disable-stripping");
//         } else {
//             configure.arg("--disable-debug");
//             configure.arg("--enable-stripping");
//         }
//          */

//         // make it static
//         configure.arg("--enable-static");
//         configure.arg("--disable-shared");

//         configure.arg("--enable-pic");

//         // do not build programs since we don't need them
//         configure.arg("--disable-programs");

//         // [REMOVED] We only use LGPL (default) for thumbnail generation
//         // configure.switch("BUILD_LICENSE_GPL", "gpl");
//         // configure.switch("BUILD_LICENSE_VERSION3", "version3");
//         // configure.switch("BUILD_LICENSE_NONFREE", "nonfree");

//         // [REMOVED] We enable all core libraries by default instead
//         // for lib in LIBRARIES.iter().filter(|lib| lib.optional) {
//         //     configure.switch(&lib.name.to_uppercase(), lib.name);
//         // }

//         // [REMOVED] We use minimal build with only built-in codecs
//         // for (cargo_feat, option_name) in EXTERNAL_BUILD_LIBS {
//         //     configure.enable(&format!("BUILD_LIB_{cargo_feat}"), option_name);
//         // }
//         // configure.enable("BUILD_DRM", "libdrm");
//         // configure.enable("BUILD_NVENC", "nvenc");
//         // configure.enable("BUILD_PIC", "pic");

//         // [MODIFIED FOR ac-ffmpeg] Enable core libraries explicitly
//         configure.arg("--enable-avcodec");
//         configure.arg("--enable-avformat");
//         configure.arg("--enable-avutil");
//         configure.arg("--enable-swresample");
//         configure.arg("--enable-swscale");
//         configure.arg("--enable-avfilter");

//         // Explicitly disable hardware acceleration on macOS (VideoToolbox doesn't support AV1)
//         if cfg!(target_os = "macos") {
//             configure.arg("--disable-videotoolbox");
//             configure.arg("--disable-audiotoolbox");
//         }

//         // Enable native AV1 decoder
//         configure.arg("--enable-decoder=av1");

//         // run ./configure
//         let output = configure
//             .output()
//             .unwrap_or_else(|_| panic!("{:?} failed", configure));
//         if !output.status.success() {
//             println!("configure: {}", String::from_utf8_lossy(&output.stdout));

//             return Err(io::Error::new(
//                 io::ErrorKind::Other,
//                 format!(
//                     "configure failed {}",
//                     String::from_utf8_lossy(&output.stderr)
//                 ),
//             ));
//         }

//         let num_jobs = if let Ok(cpus) = std::thread::available_parallelism() {
//             cpus.to_string()
//         } else {
//             "1".to_string()
//         };

//         // run make
//         if !Command::new("make")
//             .arg(format!("-j{num_jobs}"))
//             .current_dir(&source_dir)
//             .status()?
//             .success()
//         {
//             return Err(io::Error::new(io::ErrorKind::Other, "make failed"));
//         }

//         // run make install
//         if !Command::new("make")
//             .current_dir(&source_dir)
//             .arg("install")
//             .status()?
//             .success()
//         {
//             return Err(io::Error::new(io::ErrorKind::Other, "make install failed"));
//         }

//         rustc_link_extralibs(&source_dir);
//         Ok(install_dir)
//     }

//     // Copied from ffmpeg-sys-the-third with no changes
//     fn rustc_link_extralibs(source_dir: &Path) {
//         let config_mak = source_dir.join("ffbuild").join("config.mak");
//         let file = File::open(config_mak).unwrap();
//         let reader = BufReader::new(file);
//         let extra_libs = reader
//             .lines()
//             .find(|line| line.as_ref().unwrap().starts_with("EXTRALIBS"))
//             .map(|line| line.unwrap())
//             .unwrap();

//         let linker_args = extra_libs.split('=').next_back().unwrap().split(' ');
//         let include_libs = linker_args
//             .filter(|v| v.starts_with("-l"))
//             .map(|flag| &flag[2..]);

//         for lib in include_libs {
//             println!("cargo:rustc-link-lib={lib}");
//         }
//     }
// }

// fn main() -> Result<()> {
//     println!("cargo:rerun-if-changed=build.rs");

//     let out_dir = PathBuf::from(env::var("OUT_DIR").context("OUT_DIR not set")?);

//     // Build FFmpeg from source using the original build module
//     build_ffmpeg::build_ffmpeg();

//     // FFmpeg is installed to out_dir/dist by the build module
//     let ffmpeg_dist = out_dir.join("dist");

//     // Download and extract PDFium
//     download_pdfium(&out_dir)?;

//     // Copy binaries to binaries-output directory for local usage
//     copy_to_binaries_output(&out_dir, &ffmpeg_dist)?;

//     Ok(())
// }

// fn download_pdfium(out_dir: &PathBuf) -> Result<()> {
//     let platform = detect_platform();
//     let pdfium_dir = out_dir.join("pdfium");

//     // Skip if already downloaded
//     if pdfium_dir.exists() {
//         println!("PDFium already exists, skipping download");
//         return Ok(());
//     }

//     fs::create_dir_all(&pdfium_dir).context("Failed to create pdfium directory")?;

//     let url = format!(
//         "https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-{}.tgz",
//         platform
//     );

//     println!("Downloading PDFium from: {}", url);

//     let archive_path = out_dir.join(format!("pdfium-{}.tgz", platform));

//     // Download using curl
//     let status = Command::new("curl")
//         .arg("-L") // Follow redirects
//         .arg("-o")
//         .arg(&archive_path)
//         .arg(&url)
//         .status()
//         .context("Failed to execute curl")?;

//     if !status.success() {
//         anyhow::bail!("Failed to download PDFium");
//     }

//     println!("Extracting PDFium archive...");

//     // Extract tar.gz
//     let tar_gz = File::open(&archive_path).context("Failed to open archive")?;
//     let tar = GzDecoder::new(tar_gz);
//     let mut archive = Archive::new(tar);
//     archive
//         .unpack(&pdfium_dir)
//         .context("Failed to extract archive")?;

//     // Clean up archive
//     fs::remove_file(&archive_path).ok();

//     println!("PDFium downloaded and extracted successfully");

//     Ok(())
// }

// fn detect_platform() -> &'static str {
//     let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| env::consts::OS.to_string());
//     let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| env::consts::ARCH.to_string());

//     match (os.as_str(), arch.as_str()) {
//         ("macos", "aarch64") => "mac-arm64",
//         ("macos", "x86_64") => "mac-x64",
//         ("linux", "x86_64") => "linux-x64",
//         ("linux", "aarch64") => "linux-arm64",
//         ("windows", "x86_64") => "win-x64",
//         _ => panic!("Unsupported platform: {}-{}", os, arch),
//     }
// }

// fn copy_to_binaries_output(out_dir: &PathBuf, ffmpeg_dist: &PathBuf) -> Result<()> {
//     let platform = detect_platform();
//     let manifest_dir =
//         PathBuf::from(env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
//     let output_dir = manifest_dir.join("binaries-output").join(platform);
//     let output_lib = output_dir.join("lib");
//     let output_include = output_dir.join("include");

//     println!("Copying binaries to: {}", output_dir.display());

//     // Create output directories
//     fs::create_dir_all(&output_lib).context("Failed to create output lib directory")?;
//     fs::create_dir_all(&output_include).context("Failed to create output include directory")?;

//     // Copy FFmpeg
//     if ffmpeg_dist.exists() {
//         println!("Copying FFmpeg libraries...");
//         copy_dir_contents(&ffmpeg_dist.join("lib"), &output_lib)?;
//         copy_dir_contents(&ffmpeg_dist.join("include"), &output_include)?;
//     }

//     // Copy PDFium
//     let pdfium_dir = out_dir.join("pdfium");
//     if pdfium_dir.exists() {
//         println!("Copying PDFium libraries...");
//         copy_dir_contents(&pdfium_dir.join("lib"), &output_lib)?;
//         copy_dir_contents(&pdfium_dir.join("include"), &output_include)?;
//     }

//     println!("Binaries copied successfully to: {}", output_dir.display());
//     Ok(())
// }

// fn copy_dir_contents(src: &PathBuf, dst: &PathBuf) -> Result<()> {
//     if !src.exists() {
//         return Ok(());
//     }

//     for entry in fs::read_dir(src)? {
//         let entry = entry?;
//         let src_path = entry.path();
//         let dst_path = dst.join(entry.file_name());

//         if src_path.is_dir() {
//             fs::create_dir_all(&dst_path)?;
//             copy_dir_contents(&src_path, &dst_path)?;
//         } else {
//             fs::copy(&src_path, &dst_path)?;
//         }
//     }

//     Ok(())
// }
