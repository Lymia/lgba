use anyhow::*;
use derive_setters::Setters;
use log::{debug, info};
use std::{path::PathBuf, process::Command};

#[derive(Setters)]
#[setters(strip_option)]
pub struct CompileConfig {
    #[setters(skip)]
    package: String,
    #[setters(skip)]
    output: PathBuf,
    #[setters(into)]
    linker_script: Option<PathBuf>,
    linker_script_data: Option<String>,
    #[setters(bool)]
    release: bool,
}
impl CompileConfig {
    pub fn new(package: String, output: PathBuf) -> Self {
        CompileConfig {
            package,
            output,
            linker_script: None,
            linker_script_data: None,
            release: false,
        }
    }
}

pub fn compile(args: &CompileConfig) -> Result<()> {
    assert!(!(args.linker_script.is_some() && args.linker_script_data.is_some()));

    info!("Compiling package {} to '{}'...", args.package, args.output.display());

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
            if let Some(linker_script_data) = &args.linker_script_data {
                std::fs::write(&tmp_path, linker_script_data)?;
            } else {
                std::fs::write(&tmp_path, include_bytes!("lgba.ld"))?;
            }
            tmp_path
        }
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
            -C debuginfo=full
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

    let release_args: &[&str] = if args.release { &["--release"] } else { &[] };

    debug!("rustc flags: {cleaned_args:?}");
    Command::new("cargo")
        .arg("+nightly")
        .args(["build", "-p", &args.package])
        .args(release_args)
        .args(["--target", "thumbv4t-none-eabi"])
        .args(["-Z", "build-std=core,alloc"])
        .args(["-Z", "build-std-features=compiler-builtins-mangled-names"])
        .env("RUSTFLAGS", rust_args)
        .status()?
        .exit_ok()?;

    info!("Copying binary...");
    let final_path = PathBuf::from(format!("target/thumbv4t-none-eabi/release/{}", args.package));
    debug!("output path: {}", final_path.display());
    std::fs::copy(final_path, &args.output)?;

    Ok(())
}
