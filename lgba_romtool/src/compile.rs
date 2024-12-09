use anyhow::*;
use derive_setters::Setters;
use log::{debug, info};
use std::{path::PathBuf, process::Command};

// TODO: Support debug_assertions and fix the issue with compiler_builtins

#[derive(Setters)]
#[setters(strip_option)]
pub struct CompileConfig {
    #[setters(skip)]
    package: String,
    #[setters(skip)]
    output: PathBuf,
    #[setters(into)]
    linker_start_target: String,
    #[setters(skip)]
    linker_ewram_config: (usize, usize),
    #[setters(skip)]
    linker_iwram_config: (usize, usize),
    #[setters(skip)]
    linker_rom_config: (usize, usize),
    #[setters(skip)]
    extra_rust_flags: Vec<String>,
}
impl CompileConfig {
    pub fn new(package: String, output: PathBuf) -> Self {
        CompileConfig {
            package,
            output,
            linker_start_target: "__start".to_string(),
            linker_ewram_config: (0x02000000, 1024 * 256),
            linker_iwram_config: (0x03000000, 1024 * 32),
            linker_rom_config: (0x08000000, 1024 * 1024 * 32),
            extra_rust_flags: vec![],
        }
    }

    pub fn extra_rust_flags(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.extra_rust_flags.push(arg.to_string());
        }
        self
    }

    pub fn linker_ewram(mut self, origin: usize, len: usize) -> Self {
        self.linker_ewram_config = (origin, len);
        self
    }
    pub fn linker_iwram(mut self, origin: usize, len: usize) -> Self {
        self.linker_iwram_config = (origin, len);
        self
    }
    pub fn linker_rom(mut self, origin: usize, len: usize) -> Self {
        self.linker_rom_config = (origin, len);
        self
    }

    pub fn make_linker_script(&self) -> String {
        let start_symbol = &self.linker_start_target;
        let (ewram_origin, ewram_len) = self.linker_ewram_config;
        let (iwram_origin, iwram_len) = self.linker_iwram_config;
        let (rom_origin, rom_len) = self.linker_rom_config;
        format!(
            include_str!("lgba_config.ld.inc"),
            start_symbol = start_symbol,
            ewram_origin = ewram_origin,
            ewram_len = ewram_len,
            iwram_origin = iwram_origin,
            iwram_len = iwram_len,
            rom_origin = rom_origin,
            rom_len = rom_len,
            rest = include_str!("lgba_main.ld.inc"),
        )
    }
}

pub fn compile(args: &CompileConfig) -> Result<()> {
    info!("Compiling package {} to '{}'...", args.package, args.output.display());

    let home = dirs::home_dir().expect("Could not find home directory");
    let sysroot = String::from_utf8(
        Command::new("rustc")
            .args(["+nightly", "--print", "sysroot"])
            .output()?
            .stdout,
    )?;
    let rootdir = PathBuf::from(".").canonicalize()?;
    let linker_script = {
        let tmp_path = PathBuf::from(format!("{}/target/lgba.ld", rootdir.display()));
        std::fs::write(&tmp_path, args.make_linker_script())?;
        tmp_path
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
            -C opt-level=2
            -C debuginfo=full
            -Z macro-backtrace
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
    for arg in &args.extra_rust_flags {
        cleaned_args.push(arg)
    }
    let rust_args = cleaned_args.join(" ");

    debug!("rustc flags: {cleaned_args:?}");
    Command::new("cargo")
        .arg("+nightly")
        .args(["--config", "profile.release.lto=\"fat\""])
        .args(["--config", "profile.release.panic=\"abort\""])
        .args(["build", "-p", &args.package, "--release"])
        .args(["--target", "thumbv4t-none-eabi"])
        .args(["-Z", "build-std=core,alloc"])
        .env("RUSTFLAGS", rust_args)
        .status()?
        .exit_ok()?;

    info!("Copying binary...");
    let final_path = PathBuf::from(format!("target/thumbv4t-none-eabi/release/{}", args.package));
    debug!("output path: {}", final_path.display());
    std::fs::copy(final_path, &args.output)?;

    Ok(())
}
