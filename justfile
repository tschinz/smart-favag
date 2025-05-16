##################################################
# Variables
#
set dotenv-load
rust_env := "rustup show"
rust_edition := "2024"
open := if os() == "linux" {
  "xdg-open"
} else if os() == "macos" {
  "open"
} else {
  "start \"\" /max"
}
args := ""
target := '${TARGET}'

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

# Setup target files
@target target=target:
  echo "--------------------------------------------------"
  echo "-- Change .cargo/config.toml to the selected target: {{target}}"
  echo "--"
  cp .cargo/config-{{target}}.toml .cargo/config.toml
  #cp memory-{{target}}.x memory.x

# flash cyw43 firmware for rp2350
@flash-cyw43-rp2350:
  echo "--------------------------------------------------"
  echo "-- Flash Wifi Chip with Firmware for rp235x"
  echo "--"
  probe-rs download ../embassy/cyw43-firmware/43439A0.bin --binary-format bin --chip RP235x --base-address 0x10100000
  probe-rs download ../embassy/cyw43-firmware/43439A0_clm.bin --binary-format bin --chip RP235x --base-address 0x10140000

# flash cyw43 firmware for rp2040
@flash-cyw43-rp2040:
  echo "--------------------------------------------------"
  echo "-- Flash Wifi Chip with Firmware for rp2040"
  echo "--"
  probe-rs download 43439A0.bin --binary-format bin --chip RP2040 --base-address 0x10100000
  probe-rs download 43439A0_clm.bin --binary-format bin --chip RP2040 --base-address 0x10140000

# Run the program in debug mode
@run features=target args=args:
  echo "--------------------------------------------------"
  echo "-- Run with features {{features}} and args {{args}}"
  echo "--"
  cargo run --features {{features}} -- {{args}}

# flash firmware and run the program
flash-run-rp2350: flash-cyw43-rp2350
  just run rp235x

# flash firmware and run the program
flash-run-rp2040: flash-cyw43-rp2040
  just run rp2040

# Build and copy the release version of the program
build:
  cargo build --release

# Run rustfmt with custom configuration
rustfmt:
  find {{invocation_directory()}} -name \*.rs -exec rustfmt --config tab_spaces=2 --edition {{rust_edition}} {} \;
