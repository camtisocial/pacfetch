<!-- Banner -->
<p align="center">
  <img src="docs/assets/pacfetch_banner.png" alt="Project Banner" width="600" height="170" />
</p>

<!-- Badges -->
<p align="center">
  <a href="https://github.com/camtisocial/pacfetch/releases">
    <img src="https://img.shields.io/github/v/release/camtisocial/pacfetch" alt="Release" />
  </a>
  <a href="https://github.com/camtisocial/pacfetch/actions/workflows/ci.yml">
    <img src="https://github.com/camtisocial/pacfetch/actions/workflows/ci.yml/badge.svg" alt="CI" />
  </a>
  <img src="https://img.shields.io/github/issues/camtisocial/pacfetch" />
  <a href="https://aur.archlinux.org/packages/pacfetch">
    <img src="https://img.shields.io/aur/version/pacfetch" alt="AUR version" />
  </a>
  <img src="https://img.shields.io/badge/license-GPL-blue" />
</p>

<p align="center">Stat fetcher and sys upgrade wrapper for pacman</p>

---

## Installation

### AUR
```
wip
```

### Cargo
```
wip
```

### Build from source
```
git clone https://github.com/camtisocial/pacfetch
cd pacfetch
cargo build --release
```
<br>

## Usage
- Run without arguments to see a snapshot of pacman without modifying your local package databases
- or use familiar pacman flags like -Syu to run pacfetch as a neofetch style wrapper

<br>


## Flags and options


  | Flag | Description |
  |------|-------------|
  | `-Sy` | Sync package databases, then display stats |
  | `-Su` | Display stats, then upgrade packages |
  | `-Syu` | Sync databases, display stats, then upgrade 
  | `--ascii <PATH>` | Custom ASCII art file, built-in name, or `NONE` to disable |
  | `--local` | Use local cached database |
  | `-d, --debug` | Show verbose output and execution times per function |
  | `-h, --help` | Print help |
  | `-V, --version` | Print version |

  <br>

## Screenshots

<p align="center">
  <img src="docs/assets/pacfetch-screenshot.png" alt="Demo" width="1200" height="500" />
</p>


## Config
```
wip
```
