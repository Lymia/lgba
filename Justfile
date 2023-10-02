default: build

build:
    just _build_example lgba_test_rom

run-example name:
    just _build_example "{{name}}"
    mgba-qt "target/roms/{{name}}.gba"

doc:
    cargo doc -p lgba -p lgba_phf --all-features

run-codegen:
    cargo run -p maintenance_scripts

#####################
# Builder functions #
#####################

_roms_directory:
    mkdir -p target/roms

_build_romtool:
    cargo build -p lgba_romtool

_build_example name: _build_romtool _roms_directory
    "{{romtool}}" compile -p "{{name}}" -o "target/roms/{{name}}.elf"
    "{{romtool}}" build-rom \
      -b "target/roms/{{name}}.elf" -o "target/roms/{{name}}.gba" \
      -d "examples/{{name}}/RomData.toml"

####################
# Helper functions #
####################

romtool := if os_family() == "windows" {
    "target/debug/lgba_romtool.exe"
} else {
    "target/debug/lgba_romtool"
}

home_path := env_var('HOME')
sysroot_path := `rustc +nightly --print sysroot`
rootdir := justfile_directory()
export RUSTFLAGS := "
    -Z trim-diagnostic-paths=on
    --remap-path-prefix " + home_path + "/=/
    --remap-path-prefix " + home_path + "/.cargo/=/
    --remap-path-prefix " + home_path + "/.cargo/registry/src/github.com-1ecc6299db9ec823/=/
    --remap-path-prefix " + home_path + "/.cargo/git/checkouts/=/
    --remap-path-prefix " + sysroot_path + "/lib/rustlib/src/=
    --remap-path-prefix " + sysroot_path + "/lib/rustlib/src/rust/library/=rustlib
    --remap-path-prefix " + rootdir + "/=
    --remap-path-prefix " + rootdir + "/src/=
"