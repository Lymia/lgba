use std::{process::Command};

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
fn arm_as_aeabi(source: &str, out_name: &str, o_files: &mut Vec<String>) {
    arm_as_0(source, out_name, o_files, |x| x.arg("-I").arg("asm/aeabi-cortexm0"))
}
fn arm_as_agbabi(source: &str, out_name: &str, o_files: &mut Vec<String>) {
    arm_as_0(source, out_name, o_files, |x| x.arg("-I").arg("asm/agbabi"))
}

fn main() {
    if !cfg!(docs_rs) {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=../lgba.ld");

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let mut o_files = Vec::new();

        arm_as("asm/crt0.s", "crt0.o", &mut o_files);
        arm_as("asm/header.s", "header.o", &mut o_files);
        arm_as("asm/save.s", "save.o", &mut o_files);
        arm_as("asm/sys.s", "sys.o", &mut o_files);

        //arm_as_aeabi("asm/aeabi-cortexm0/lmul.S", "aeabi_lmul.o", &mut o_files);

        arm_as_agbabi("asm/agbabi/memcpy.s", "agbabi_memcpy.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/memset.s", "agbabi_memset.o", &mut o_files);

        arm_as_agbabi("asm/agbabi/idiv.s", "agbabi_idiv.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/ldiv.s", "agbabi_ldiv.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/lmul.s", "agbabi_lmul.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/uidiv.s", "agbabi_uidiv.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/uldiv.s", "agbabi_uldiv.o", &mut o_files);
        arm_as_agbabi("asm/agbabi/uluidiv.s", "agbabi_uluidiv.o", &mut o_files);

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
    }
}
