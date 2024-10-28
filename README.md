# Turiya OS

A simple operating system written in Rust. This is a learning project to understand how operating systems work.

Most of the code is based on the [Writing an OS in Rust](https://os.phil-opp.com/) blog series by Philipp Oppermann.

## Topics Covered

- [x] Booting
- [x] VGA Text Mode
- [x] Serial Port
- [x] CPU Exceptions
- [x] Interrupts
- [x] Keyboard Input

## Setup

1. Follow the libraries version in the [Cargo.toml](Cargo.toml) file.
2. Install the `bootimage` tool by running `cargo install bootimage`.
3. Use nightly Rust  for experimental features.
4. Instal qemu for testing the OS.