# nvoclock

[![release-badge][]][cargo] [![license-badge][]][license]

`nvoclock` is a command-line interface to NVAPI that supports full monitoring
and overclocking of NVIDIA GPUs on Windows platforms.

## Download

[Binary downloads](https://github.com/arcnmx/nvoclock/releases) are available
here on Github for both x86 and x86_64 Windows platforms. No installer
necessary, just bring the GPU and ensure the drivers are installed properly. It
can also be installed and built from source using `cargo install nvoclock`

## Features

While the interface may be a bit clunky, it supports everything you'd expect out
of a modern overclocking tool:

- GPU detection and displaying of stats, capabilities, etc. similar to GPU-Z
- Monitor the status of a GPU including power draw, load usage, clocks, voltage,
  temperatures, fans, and so on - anything Afterburner would have a chart for
- Fan control, thermal, and power limits
- Traditional (pstate) offset overclocking
- GPU Boost 3.0 frequency curve controls (VFP)
  - Import/export to CSV file
  - Voltage lock (single point testing)
- Pascal voltage boost

## Usage

- `nvoclock info` displays information about the capabilities of detected GPUs
- `nvoclock status` displays monitoring information about the GPU
  - `nvoclock status -a` shows some fancy tables!
  - Use in combination with [watch(1)](https://linux.die.net/man/1/watch) for
    best results.
- `nvoclock set` encompasses the usual options to overclock and tweak a GPU.
  Check `-h` for all the details.

### Global Options

- `-g 0` flag can be used to filter results and operations to a specific GPU
- `-O json` prints out information in JSON format to be parsed or handled by
  automated scripts.
- `set RUST_LOG=trace` to get excessive debugging information. You'll probably
  want to use `nvoclock info 2> nvolog.txt` to save to a file for later
  interpretation.

## Auto VF curve
Now most of the overclocking process can be offloaded to `nvoclock`. It will take care of running tests and adjusting GPU frequencies.  
Available with `nvoclock set vfp auto`:
```
-s - step to start at. For laptops - around 25-30.
-e - step to end at. Laptops again, around 70
-t - test binary to run. Must return 0 if test passed, anything otherwise.
```
### How-to
0. Laptop users - find a way to keep GPU active. If GPU goes idle, then status query fails.
1. Build a test binary that will return status code 0 if test went fine.
2. Determine GPU border points - use `nvoclock status -a` and look for starred point.  
2.1. Start - when the GPU is idling.  
2.2. End - at full load + a few points more. There's no harm in selecting higher end point.
3. Run `nvoclock set vfp auto`. The program will scan your VF curve from top to bottom and output results.  
3.1. Your PC may (and will) crash a few times during the overclocking. That's normal, that means that overclocking settings went too far. All properly tested points will result in a `tempres` file with a part of expected output. Save partial results and append them to final ones afterwards.

### How the overclocking works?
Techincally, this isn't overclock (not fully). This is undervolting. It makes your GPU run same frequencies on lower voltages, resulting in lower temps and more performance. As long as we don't increase voltage, in the worst case system will just hang and reboot.  
`nvoclock` takes these steps to find optimal clocks:
1. Read current VF curve and select last point
2. Run a test operation and background monitoring
3. Check that GPU keeps same clocks during the test process
4. If clocks aren't stable - return limiting reason, set the point as a limit and go to the next one
5. If test failed - step three steps back, set the point as limit and go to the next one
6. Otherwise, up frequencies one step and repeat from step 2  
...until the end of the VF curve.
> We are going from top to bottom because of monotonicity rule - otherwise previous points affect next ones
## Future Items

Some things can be improved, and since most testing was done with a single
pascal GPU there are some missing features for older hardware.

- Previous generation GPUs need testing/support
  - Overvolting support needs doing
- RPC API + Daemon
  - Controls from another computer so autodetect can detect and survive crashes
    and full lock-ups.
  - Running on a VFIO host to control a guest GPU would be neat. SSH mostly does
    this already though.

## Out of Scope

`nvoclock` isn't meant to be an all-encompassing overclocking and monitoring
tool. The following features would better belong in a separate project:

- GUI ([nvapi-rs](https://crates.io/crates/nvapi) does all the real work and
  makes it easy to create one though!)
- AMD GPU support
- CPU monitoring and/or overclocking
- Status overlays and game hooks
- Linux support (`nvapi` is not available)
- Software fan curve control (I'll get around to a daemon for this eventually)

[release-badge]: https://img.shields.io/crates/v/nvoclock.svg?style=flat-square
[cargo]: https://crates.io/crates/nvoclock
[license-badge]: https://img.shields.io/badge/license-MIT-ff69b4.svg?style=flat-square
[license]: https://github.com/arcnmx/nvoclock/blob/master/COPYING
