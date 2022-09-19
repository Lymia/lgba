fn arm_as(source: &str, out_name: &str, o_files: &mut Vec<String>) {
    println!("cargo:rerun-if-changed={source}");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir_file = format!("{out_dir}/{out_name}");
    let as_output = std::process::Command::new("arm-none-eabi-as")
        .args(&["-o", out_dir_file.as_str()])
        .arg("-mthumb-interwork")
        .arg("-mcpu=arm7tdmi")
        .arg("-g")
        .arg(source)
        .output()
        .expect("failed to run arm-none-eabi-as");
    if !as_output.status.success() {
        panic!("{}", String::from_utf8_lossy(&as_output.stderr));
    }
    o_files.push(out_dir_file);
}

fn main() {
    if !cfg!(docs_rs) {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=../lgba.ld");

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let mut o_files = Vec::new();

        arm_as("src/crt0.s", "crt0.o", &mut o_files);
        arm_as("src/sys.s", "sys.o", &mut o_files);

        let archive_name = format!("{out_dir}/liblgba_as.a");
        std::fs::remove_file(&archive_name).ok();
        let ar_out = std::process::Command::new("arm-none-eabi-ar")
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
