//! LLVM Toolchain source and installation tools

use crate::{
    emoji,
    toolchain::{download_file, espidf::get_tool_path},
};
use anyhow::{bail, Ok, Result};
use log::info;
use std::path::{Path, PathBuf};

const DEFAULT_LLVM_COMPLETE_REPOSITORY: &str =
    "https://github.com/espressif/llvm-project/releases/download";
const DEFAULT_LLVM_MINIFIED_REPOSITORY: &str =
    "https://github.com/esp-rs/rust-build/releases/download/llvm-project-14.0-minified";
const DEFAULT_LLVM_VERSION: &str = "esp-14.0.0-20220415";

#[derive(Debug)]
pub struct LlvmToolchain {
    /// The repository containing LVVM sources.
    pub repository_url: String,
    /// Repository release version to use.
    pub version: String,
    /// LLVM Toolchain file name.
    pub file_name: String,
    /// LLVM Toolchain path.
    pub path: PathBuf,
}

impl LlvmToolchain {
    /// Gets the name of the LLVM arch based on the host triple.
    fn get_arch(host_triple: &str) -> Result<String> {
        match host_triple {
            "aarch64-apple-darwin" | "x86_64-apple-darwin" => Ok("macos".to_string()),
            "x86_64-unknown-linux-gnu" => Ok("linux-amd64".to_string()),
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => Ok("win64".to_string()),
            _ => bail!(
                "{} No LLVM arch found for the host triple: {}",
                emoji::ERROR,
                host_triple
            ),
        }
    }

    /// Gets the artifact extension based on the host architecture.
    fn get_artifact_extension(host_triple: &str) -> &str {
        match host_triple {
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "zip",
            _ => "tar.xz",
        }
    }

    /// Gets the binary path.
    fn get_lib_path(&self) -> String {
        #[cfg(windows)]
        let llvm_path = format!("{}/xtensa-esp32-elf-clang/bin", self.path.to_str().unwrap());
        #[cfg(unix)]
        let llvm_path = format!("{}/xtensa-esp32-elf-clang/lib", self.path.to_str().unwrap());
        llvm_path
    }

    /// Installs the LLVM toolchain.
    pub fn install(&self) -> Result<Vec<String>> {
        let mut exports: Vec<String> = Vec::new();

        if Path::new(&self.path).exists() {
            bail!(
            "{} Previous installation of LLVM exist in: {}.\n Please, remove the directory before new installation.",
            emoji::WARN,
            self.path.to_str().unwrap()
        );
        } else {
            info!("{} Installing Xtensa elf Clang", emoji::WRENCH);
            download_file(
                self.repository_url.clone(),
                &format!(
                    "idf_tool_xtensa_elf_clang.{}",
                    Self::get_artifact_extension(guess_host_triple::guess_host_triple().unwrap())
                ),
                self.path.to_str().unwrap(),
                true,
            )?;
        }
        // Set environment variables.
        #[cfg(windows)]
        exports.push(format!(
            "$Env:LIBCLANG_PATH=\"{}/libclang.dll\"",
            self.get_lib_path()
        ));
        #[cfg(windows)]
        exports.push(format!("$Env:PATH+=\";{}\"", self.get_lib_path()));
        #[cfg(unix)]
        exports.push(format!("export LIBCLANG_PATH=\"{}\"", self.get_lib_path()));

        Ok(exports)
    }

    /// Create a new instance with default values and proper toolchain version.
    pub fn new(minified: bool) -> Self {
        let host_triple = guess_host_triple::guess_host_triple().unwrap();
        let version = DEFAULT_LLVM_VERSION.to_string();
        let file_name: String;
        let repository_url: String;
        if minified {
            file_name = format!(
                "xtensa-esp32-elf-llvm{}-{}-{}.{}",
                get_release_with_underscores(&version),
                &version,
                host_triple,
                Self::get_artifact_extension(host_triple)
            );
            repository_url = format!("{}/{}", DEFAULT_LLVM_MINIFIED_REPOSITORY, file_name,);
        } else {
            file_name = format!(
                "xtensa-esp32-elf-llvm{}-{}-{}.{}",
                get_release_with_underscores(&version),
                &version,
                Self::get_arch(host_triple).unwrap(),
                Self::get_artifact_extension(host_triple)
            );
            repository_url = format!(
                "{}/{}/{}",
                DEFAULT_LLVM_COMPLETE_REPOSITORY, &version, file_name
            );
        }
        let path = format!(
            "{}/{}-{}",
            get_tool_path("xtensa-esp32-elf-clang"),
            version,
            host_triple
        )
        .into();
        Self {
            repository_url,
            version,
            file_name,
            path,
        }
    }
}

/// Gets the parsed version name.
fn get_release_with_underscores(version: &str) -> String {
    let version: Vec<&str> = version.split('-').collect();
    let llvm_dot_release = version[1];
    llvm_dot_release.replace('.', "_")
}

#[cfg(test)]
mod tests {
    use crate::toolchain::llvm_toolchain::get_release_with_underscores;

    #[test]
    fn test_get_release_with_underscores() {
        assert_eq!(
            get_release_with_underscores("esp-14.0.0-20220415"),
            "14_0_0".to_string()
        );
    }
}