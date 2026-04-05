use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

const PREFIX: &str = "[dobby]";

pub fn run(args: &[String]) -> Result<i32> {
    let forge_root = ensure_vendor_sources()?;
    let binary = ensure_binary(&forge_root)?;

    let status = Command::new(&binary)
        .args(args)
        .current_dir(std::env::current_dir().unwrap_or_else(|_| forge_root.clone()))
        .status()
        .with_context(|| format!("Failed to launch Forge binary at {}", binary.display()))?;

    Ok(status.code().unwrap_or(1))
}

fn ensure_vendor_sources() -> Result<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/forgecode");
    if !path.exists() {
        bail!(
            "Forge sources missing. Run `git submodule update --init --recursive` before delegating commands."
        );
    }
    Ok(path)
}

fn ensure_binary(root: &Path) -> Result<PathBuf> {
    let binary_name = if cfg!(target_os = "windows") {
        "forge.exe"
    } else {
        "forge"
    };
    let binary = root.join("target").join("release").join(binary_name);
    if binary.exists() {
        return Ok(binary);
    }

    println!("{} Building Forge CLI from vendored sources...", PREFIX);
    build_binary(root)?;
    if !binary.exists() {
        bail!(
            "Forge build completed but binary missing at {}",
            binary.display()
        );
    }
    Ok(binary)
}

fn build_binary(root: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .current_dir(root)
        .args(["build", "--release"])
        .status()
        .context("Failed to spawn cargo for Forge build")?;

    if !status.success() {
        bail!("cargo build exited with status {:?}", status.code());
    }

    Ok(())
}
