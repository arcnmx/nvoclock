# nvoclock

[![travis-badge][]][travis] [![release-badge][]][cargo] [![license-badge][]][license]

`nvoclock` is a command-line interface to NVAPI that supports full monitoring
and overclocking of NVIDIA GPUs on Windows platforms.

## Usage

- `nvoclock info` displays information about the capabilities of detected GPUs
- `nvoclock status` displays monitoring information about the GPU
  - Use in combination with [watch(1)](https://linux.die.net/man/1/watch) for
    best results.
- `nvoclock status -a` shows fancy charts!
- `nvoclock set` encompasses the usual options to overclock and tweak a GPU.
  Check `-h` for all the details.

### Global Options

- `-g 0` can be used to filter results and operations to a specific GPU
- `-O json` prints out information in JSON format to be parsed or handled by
  automated scripts.

## Future Items

Some things can be improved, and since most tested was done with a single Pascal
GPU there are some missing features for older hardware.

- Maxwell power/voltage control
- Previous generation GPUs need testing/support
- RPC API + Daemon
  - Controls from another computer so autodetect can detect and survive crashes
    and full lock-ups.
  - Running on a VFIO host to control a guest GPU would be neat. SSH mostly does
    this already though.

## Out of Scope

`nvoclock` isn't meant to be an all-encompassing overclocking and monitoring
tool. The following features would better belong in a separate project:

- GUI ([nvapi-rs](https://github.com/arcnmx/nvapi-rs) does all the real work and
  makes it easy to create one though!)
- AMD GPU support
- CPU monitoring and/or overclocking
- Status overlays and game hooks
- Linux support (`nvapi` is not available)

[travis-badge]: https://img.shields.io/travis/arcnmx/nvoclock/master.svg?style=flat-square
[travis]: https://travis-ci.org/arcnmx/nvoclock
[release-badge]: https://img.shields.io/crates/v/nvoclock.svg?style=flat-square
[cargo]: https://crates.io/crates/nvoclock
[license-badge]: https://img.shields.io/badge/license-MIT-ff69b4.svg?style=flat-square
[license]: https://github.com/arcnmx/nvoclock/blob/master/COPYING
