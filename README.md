# Brillo - Brightness Control for Linux (Re-write)
Control the brightness of backlight, keyboard LEDs, and external monitors on Linux.

This is a rewrite with AI help of the fantastic tool
[brillo](https://gitlab.com/cameronnemo/brillo/), so all credits goes to
**Cameron Nemo**. The reimplementation is in Rust, extending functionality to
control external monitors as well. 

- Caching is done for all devices with `-O`,
writting a cache file for each device, then resuming value is also done for all
devices available.
- I have removed transition time as it was not useful for me

## Features

- **Controller detection** - Finds all backlight controllers to be used
- **Multiple targets** - Backlight, keyboard LEDs, and external monitors (via DDC/CI)
- **Save/Restore ALL devices** - One command saves monitors + backlight + keyboard
- **Per-device caching** - Each device saved to individual cache file
- **DDC/CI support** - Control external monitor brightness via `ddcutil`
- **Logging** - Configurable verbosity via `RUST_LOG`

## Installation

### Build from source

```bash
cargo build --release
```

### Dependencies

- For external monitor support: `ddcutil` v2.0+ (install via your package manager)

> **Note**: The Rust implementation uses the `ddcutil` binary rather than libddcutil bindings
> to avoid version compatibility issues. The response is rather slow, but using
> `busnum` instead of display number would improve it. That's also the reason
> why I removed brightness ramp.

## TODO

- [ ] Fixup identifying controllers and keep a list of them
- [ ] Fixup slow `ddcutil` calls by caching `busnum`. 
 - Example in Noctalia: https://github.com/noctalia-dev/noctalia/blob/abd21e85bc354c7d1e3d624ed064b6c729ed4846/tests/brightness_service_test.cpp#L87
 - WARN: Need to update the busnum when monitors connection changed, perhaps
   each monitor should have a hash, instead of number ID. Hash={Product code,
   binary serial number}

## Usage

### Overview 

#### 1. Save/Restore Behavior

| Command | Saves/Restores |
|---------|----------------|
| `brillo -O` | **ALL devices** (monitors + laptop backlight + keyboard) |
| `brillo -I` | **ALL devices** (monitors + laptop backlight + keyboard) |
| `brillo -O -t backlight` | Only laptop backlight |
| `brillo -O -t keyboard` | Only keyboard backlight |
| `brillo -O -t monitor` | **ALL monitors** (not just one) |
| `brillo -I -t monitor` | **ALL monitors** (not just one) |
| `brillo --test -t monitor` | Full 3-step workflow |

#### 2. Cache Directory Structure

```
~/.cache/brillo/
├── monitor.1.brightness      # External monitor 1
├── monitor.2.brightness      # External monitor 2 (if present)
├── intel_backlight.brightness # Laptop backlight
└── dell::kbd_backlight.brightness # Keyboard backlight
```

- Per-device files - each device saved individually  
- Easy cleanup - `rm -rf ~/.cache/brillo`

### Basic Operations

```bash
# Get current brightness
brillo

# Set brightness to 50% (percentage format)
brillo -S 50%

# Set brightness to raw value 5000
brillo -S 5000

# Increase brightness by 10%
brillo -A 10%

# Decrease brightness by 10%
brillo -U 10%

# List all controllers
brillo -L

# Save ALL devices (monitors + backlight + keyboard)
brillo -O

# Restore ALL devices
brillo -I
```

### External Monitors (DDC/CI)

```bash
# List DDC/CI capable monitors
brillo -L -t monitor

# Set external monitor brightness to 75%
brillo -S 75% -t monitor

# Test workflow: save → dim to 5% → restore
brillo --test --target monitor

# Individual workflow steps:
# Step 1: Save all monitor brightness values
brillo -O --target monitor

# Step 2: Dim all monitors to 5%
brillo -S 5% --target monitor  # Applies to all monitors by default

# Step 3: Restore all monitors to saved values
brillo -I --target monitor
```

## Command Line Options

| Option | Description |
|--------|-------------|
| `-G` | Get current brightness |
| `-S` | Set brightness (e.g., `-S 50%` or `-S 5000`) |
| `-A` | Add to brightness (e.g., `-A 10%` or `-A 1000`) |
| `-U` | Subtract from brightness (e.g., `-U 10%` or `-U 1000`) |
| `-O` | Save current brightness |
| `-I` | Restore saved brightness |
| `-L` | List all controllers |
| `--test` | Test workflow: save → dim to 5% → restore (monitors) |
| `--target monitor` | Target external monitors |
| `-s` | Specify controller name (auto-detect if not set) |

## DDC/CI Monitor Support

Brillo integrates with `ddcutil` to control external monitor brightness. This requires:

1. `ddcutil` installed and in PATH
2. Monitor supports DDC/CI (most modern monitors do)
3. Proper permissions (typically requires being in the `i2c` group or running as root)

### Setup for DDC/CI

```bash
# Install ddcutil
sudo pacman -S ddcutil  # Arch
sudo apt install ddcutil  # Debian/Ubuntu

# Add user to i2c group
sudo usermod -aG i2c $USER

# Verify ddcutil works
ddcutil detect

```

## Original Project

This is a Rust rewrite of the original [brillo](https://gitlab.com/CameronNemo/brillo) project by Cameron Nemo.
