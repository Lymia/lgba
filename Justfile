default: build

build:
    just _build_example lgba_test_rom

run-example name:
    just _build_example "{{name}}"
    mgba-qt "target/roms/{{name}}.gba"

#####################
# Builder functions #
#####################

_roms_directory:
    mkdir -p target/roms

_build_romtool:
    cargo build --target "{{local_target}}" -Z build-std=std,core,alloc -p lgba_romtool

_build_example name: _build_romtool _roms_directory
    cargo build -p "{{name}}" --release
    "{{romtool}}" build-rom "target/thumbv4t-none-eabi/release/{{name}}" "target/roms/{{name}}.gba"

####################
# Helper functions #
####################

target_suffix := if os() == "linux" {
    "-unknown-linux-gnu"
} else if os() == "windows" {
    "-pc-windows-msvc"
} else if os() == "macos" {
    "-apple-darwin"
} else {
    error("unknown platform")
}
local_target := arch() + target_suffix

romtool := if os_family() == "windows" {
    "target/" + local_target + "/debug/lgba_romtool.exe"
} else {
    "target/" + local_target + "/debug/lgba_romtool"
}
