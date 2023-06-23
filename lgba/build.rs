use std::process::Command;

fn arm_as_0(
    source: &str,
    out_name: &str,
    o_files: &mut Vec<String>,
    exflag: impl FnOnce(&mut Command) -> &mut Command,
) {
    println!("cargo:rerun-if-changed={source}");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir_file = format!("{out_dir}/{out_name}");
    let as_output = exflag(
        Command::new("arm-none-eabi-as")
            .args(&["-o", out_dir_file.as_str()])
            .arg("-mthumb-interwork")
            .arg("-mcpu=arm7tdmi"),
    )
    .arg("-g")
    .arg(source)
    .output()
    .expect("failed to run arm-none-eabi-as");
    if !as_output.status.success() {
        panic!("{}", String::from_utf8_lossy(&as_output.stderr));
    }
    o_files.push(out_dir_file);
}
fn arm_as(source: &str, out_name: &str, o_files: &mut Vec<String>) {
    arm_as_0(source, out_name, o_files, |x| x)
}

fn main() {
    if !cfg!(docs_rs) {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=../lgba.ld");

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let mut o_files = Vec::new();

        // lgba-specific code
        arm_as("src_asm/lgba/crt0.s", "crt0.o", &mut o_files);
        arm_as("src_asm/lgba/header.s", "header.o", &mut o_files);
        arm_as("src_asm/lgba/save.s", "save.o", &mut o_files);
        arm_as("src_asm/lgba/sys.s", "sys.o", &mut o_files);

        // memory operations
        arm_as("src_asm/aeabi/memmove.S", "aeabi_memcpy.o", &mut o_files);
        arm_as("src_asm/aeabi/memset.S", "aeabi_memset.o", &mut o_files);

        // numeric operations
        arm_as("src_asm/aeabi/idiv.S", "aeabi_idiv.o", &mut o_files);
        arm_as("src_asm/aeabi/idivmod.S", "aeabi_idivmod.o", &mut o_files);
        arm_as("src_asm/aeabi/lasr.S", "aeabi_lasr.o", &mut o_files);
        arm_as("src_asm/aeabi/ldivmod.S", "aeabi_ldivmod.o", &mut o_files);
        arm_as("src_asm/aeabi/llsl.S", "aeabi_llsl.o", &mut o_files);
        arm_as("src_asm/aeabi/llsr.S", "aeabi_llsr.o", &mut o_files);
        arm_as("src_asm/aeabi/lmul.S", "aeabi_lmul.o", &mut o_files);

        let archive_name = format!("{out_dir}/liblgba_as.a");
        std::fs::remove_file(&archive_name).ok();
        let ar_out = Command::new("arm-none-eabi-ar")
            .arg("-crs")
            .arg(&archive_name)
            .args(&o_files)
            .output()
            .expect("Failed to create static library");
        if !ar_out.status.success() {
            panic!("{}", String::from_utf8_lossy(&ar_out.stderr));
        }

        println!("cargo:rustc-link-search={out_dir}");
        println!("cargo:rustc-link-lib=static=lgba_as");
    }
}
