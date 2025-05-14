##################################################
# Variables
#
rust_env := "rustup show"
rust_edition := "2021"
open := if os() == "linux" {
  "xdg-open"
} else if os() == "macos" {
  "open"
} else {
  "start \"\" /max"
}
args := ""
project_directory := justfile_directory()
url := "https://github.com/tschinz/smart-favag"

##################################################
# COMMANDS
#

# List all commands
@default:
  just --list

# Information about the environment
@info:
  echo "Environment Informations\n------------------------\n"
  echo "OS   : {{os()}}({{arch()}})"
  echo "Open : {{open}}"
  echo "Rust :"
  echo "`{{rust_env}}`"

# Install dependencies
install-common:
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  cargo install --locked trunk
  rustup target add thumbv6m-none-eabi

# Install missing dependencies on macos
[macos]
install: install-common
  # probe-rs
  brew install arm-none-eabi-gdb
  brew tap probe-rs/probe-rs
  brew install probe-rs

# flash cyw43 firmware for rp2350
flash-cyw43-rp2350:
  probe-rs download ../embassy/cyw43-firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000
  probe-rs download ../embassy/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000

# flash cyw43 firmware for rp2040
flash-cyw43-rp2040:
  probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
  probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000

# flash firmware and run the program
flash-run-rp2350: flash-cyw43-rp2350 run

# flash firmware and run the program
flash-run-rp2040: flash-cyw43-rp2040 run

# Run the program in debug mode
run args=args:
  cargo run -- {{args}}

# Build and copy the release version of the program
build:
  cargo build --release

# Run rustfmt with custom configuration
rustfmt:
  find {{invocation_directory()}} -name \*.rs -exec rustfmt --config tab_spaces=2 --edition {{rust_edition}} {} \;
