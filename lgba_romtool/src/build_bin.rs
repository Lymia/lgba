use crate::BuildBin;
use anyhow::*;
use std::{path::PathBuf, process::Command};
use tracing::{debug, info};

pub(crate) fn build_bin(args: &BuildBin) -> Result<()> {
    info!("Compiling package {} to '{}'...", args.package_name, args.output_path.display());

    let home = dirs::home_dir().expect("Could not find home directory");
    let sysroot = String::from_utf8(
        Command::new("rustc")
            .args(["+nightly", "--print", "sysroot"])
            .output()?
            .stdout,
    )?;
    let rootdir = PathBuf::from(".").canonicalize()?;
    let linker_script = match &args.linker_script {
        None => {
            let tmp_path = PathBuf::from(format!("{}/target/lgba.ld", rootdir.display()));
            std::fs::write(&tmp_path, include_bytes!("lgba.ld"))?;
            tmp_path
        },
        Some(script) => script.clone(),
    };
    let rust_args = format!(
        "
            -Z trim-diagnostic-paths=on
            --remap-path-prefix {home}/=/
            --remap-path-prefix {home}/.cargo/=/
            --remap-path-prefix {home}/.cargo/registry/src/github.com-1ecc6299db9ec823/=/
            --remap-path-prefix {home}/.cargo/git/checkouts/=/
            --remap-path-prefix {sysroot}/lib/rustlib/src/=
            --remap-path-prefix {sysroot}/lib/rustlib/src/rust/library/=rustlib
            --remap-path-prefix {rootdir}/=
            --remap-path-prefix {rootdir}/src/=
            -C link-arg=-T{linker_script}
            -C target-cpu=arm7tdmi
            -C opt-level=3
            -C lto=fat
        ",
        home = home.display(),
        sysroot = sysroot.trim(),
        rootdir = rootdir.display(),
        linker_script = linker_script.display(),
    );

    let mut cleaned_args = Vec::new();
    for arg in rust_args.split(' ') {
        if !arg.trim().is_empty() {
            cleaned_args.push(arg.trim());
        }
    }
    let rust_args = cleaned_args.join(" ");

    debug!("rustc flags: {cleaned_args:?}");
    Command::new("cargo")
        .arg("+nightly")
        .args(["build", "-p", &args.package_name, "--release"])
        .args(["--target", "thumbv4t-none-eabi"])
        .args(["-Z", "build-std=core,alloc"])
        .args(["-Z", "build-std-features=compiler-builtins-mangled-names"])
        .env("RUSTFLAGS", rust_args)
        .status()?
        .exit_ok()?;

    info!("Copying binary...");
    let output_path =
        PathBuf::from(format!("target/thumbv4t-none-eabi/release/{}", args.package_name));
    debug!("output path: {}", output_path.display());
    std::fs::copy(output_path, &args.output_path)?;

    Ok(())
}
