//! Xtensa Rust Toolchain source and installation tools

use super::InstallOpts;
use crate::chip::Chip;
use crate::emoji;
use crate::utils::{download_file, get_dist_path};
use anyhow::Result;
use embuild::cmd;
use log::info;
use std::path::PathBuf;

const DEFAULT_XTENSA_RUST_REPOSITORY: &str =
    "https://github.com/esp-rs/rust-build/releases/download";

pub struct RustToolchain {
    /// Xtensa Rust toolchain file.
    pub dist_file: String,
    /// Xtensa Rust toolchain url.
    pub dist_url: String,
    /// Xtensa Src Rust toolchain file.
    pub src_dist_file: String,
    /// Xtensa Src Rust toolchain url.
    pub src_dist_url: String,
    /// ESP targets.
    pub targets: Vec<Chip>,
    /// Extra crates to install.
    pub extra_crates: String,
    /// Nightly version to install.
    pub nightly_version: String,
    /// Path to the cargo home directory.
    pub cargo_home: PathBuf,
    /// Path to the rustup home directory.
    pub rustup_home: PathBuf,
    /// Xtensa Rust toolchain destination path.
    pub toolchain_destination: PathBuf,
    /// Xtensa Rust Toolchain version.
    pub version: String,
}

impl RustToolchain {
    /// Gets the artifact extension based on the host architecture.
    fn get_artifact_extension(host_triple: &str) -> &str {
        match host_triple {
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "zip",
            _ => "tar.xz",
        }
    }

    /// Gets the installer file.
    pub fn get_installer(host_triple: &str) -> &str {
        match host_triple {
            "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => "",
            _ => "./install.sh",
        }
    }

    /// Installs the RiscV target.
    pub fn install_riscv_target(&self) -> Result<()> {
        info!("{} Installing Riscv target", emoji::WRENCH);
        cmd!(
            "rustup",
            "component",
            "add",
            "rust-src",
            "--toolchain",
            self.version.clone()
        )
        .run()?;
        cmd!(
            "rustup",
            "target",
            "add",
            "--toolchain",
            self.version.clone(),
            "riscv32imac-unknown-none-elf"
        )
        .run()?;
        Ok(())
    }

    /// Installs the Xtensa Rust toolchain.
    pub fn install_xtensa(&self) -> Result<()> {
        let host_triple = guess_host_triple::guess_host_triple().unwrap();

        // Some platfroms like Windows are available in single bundle rust + src, because install
        // script in dist is not available for the plaform. It's sufficient to extract the toolchain
        if Self::get_installer(host_triple).to_string().is_empty() {
            download_file(
                self.dist_url.clone(),
                "rust.zip",
                &self.toolchain_destination.display().to_string(),
                true,
            )?;
        } else {
            download_file(
                self.dist_url.clone(),
                "rust.tar.xz",
                &get_dist_path("rust"),
                true,
            )?;
            info!("{} Installing rust esp toolchain", emoji::WRENCH);
            let arguments = format!(
                "{}/rust-nightly-{}/install.sh --destdir={} --prefix='' --without=rust-docs",
                get_dist_path("rust"),
                host_triple,
                self.toolchain_destination.display()
            );
            cmd!("/bin/bash", "-c", arguments).run()?;

            download_file(
                self.src_dist_url.clone(),
                "rust-src.tar.xz",
                &get_dist_path("rust-src"),
                true,
            )?;
            info!("{} Installing rust-src for esp toolchain", emoji::WRENCH);
            let arguments = format!(
                "{}/rust-src-nightly/install.sh --destdir={} --prefix='' --without=rust-docs",
                get_dist_path("rust-src"),
                self.toolchain_destination.display()
            );
            cmd!("/bin/bash", "-c", arguments).run()?;
        }
        Ok(())
    }

    // TODO: Some fields are not needed in Windows
    /// Create a new instance.
    pub fn new(args: &InstallOpts, arch: &str, targets: &[Chip]) -> Self {
        let artifact_extension = Self::get_artifact_extension(arch);
        let version = args.toolchain_version.clone();

        let dist = format!("rust-{}-{}", args.toolchain_version, arch);
        let dist_file = format!("{}.{}", dist, artifact_extension);
        let dist_url = format!(
            "{}/v{}/{}",
            DEFAULT_XTENSA_RUST_REPOSITORY, version, dist_file
        );
        let src_dist = format!("rust-src-{}", args.toolchain_version);
        let src_dist_file = format!("{}.{}", src_dist, artifact_extension);
        let src_dist_url = format!(
            "{}/v{}/{}",
            DEFAULT_XTENSA_RUST_REPOSITORY, version, src_dist_file
        );

        Self {
            dist_file,
            dist_url,
            src_dist_file,
            src_dist_url,
            targets: targets.to_vec(),
            extra_crates: args.extra_crates.clone(),
            nightly_version: args.nightly_version.clone(),
            cargo_home: args.cargo_home.clone(),
            rustup_home: args.rustup_home.clone(),
            toolchain_destination: args.toolchain_destination.clone(),
            version,
        }
    }
}