
<p align="center">
  <img src="docs/assets/pacfetch_banner.png" alt="Project Banner" width="600" height="170" />
</p>

<div align="center">

<a href="https://github.com/camtisocial/pacfetch/releases">
  <img src="https://img.shields.io/github/v/release/camtisocial/pacfetch?label=release&style=flat" />
</a>
<a href="https://github.com/camtisocial/pacfetch/issues">
  <img src="https://img.shields.io/github/issues/camtisocial/pacfetch?label=issues&color=red&style=flat" />
</a>
<a href="https://github.com/camtisocial/pacfetch/actions/workflows/ci.yml">
  <img src="https://img.shields.io/github/actions/workflow/status/camtisocial/pacfetch/ci.yml?label=ci&style=flat" />
</a>
<a href="https://crates.io/crates/pacfetch">
  <img src="https://img.shields.io/crates/v/pacfetch?label=crates.io&color=orange&style=flat" />
</a>
<a href="https://aur.archlinux.org/packages/pacfetch">
  <img src="https://img.shields.io/aur/version/pacfetch?label=aur&color=1793d1&style=flat" />
</a>

<a href="https://github.com/camtisocial/pacfetch/blob/main/LICENSE">
  <img src="https://img.shields.io/github/license/camtisocial/pacfetch?label=license&color=blueviolet&style=flat" />
</a>

</div>


<p align="center">Neofetch style snapshot and sys update wrapper for the Arch linux package manager</p>

---

## Installation

#### AUR
```
yay -S pacfetch
```

#### Cargo
```
cargo install pacfetch
```

#### Build from source
```
git clone https://github.com/camtisocial/pacfetch
cd pacfetch
sudo make install
```
<br>
<br>

## Usage

Run `pacfetch -Syu` to sync and upgrade, or just `pacfetch` to see stats synced to a temp database [(no risk of partial upgrades)](https://wiki.archlinux.org/title/Pacman#Upgrading_packages)


<br>


  | Flag | Description |
  |------|-------------|
  | `no args` | Show stats with sync to temp databases|
  | `-Syu` | Sync databases, display stats, then upgrade 
  | `-Sy` | Sync package databases, then display stats |
  | `-Su` | Display stats, then upgrade packages |
  | `--ascii <PATH>` | Custom ASCII art file, built-in name, or `NONE` to disable |
  | `--local` | Use local cached database |
  | `-d, --debug` | Show verbose output and execution times per function |
  | `-h, --help` | Print help |
  | `-V, --version` | Print version |

  <br>
  <br>

### Configuration & Logs

- User config is automatically created on first run at `~/.config/pacfetch/pacfetch.toml`  
- Error logs are written to `~/.cache/pacfetch/pacfetch.log`

  <br>
  <br>

## Roadmap

**Display customization overhaul**  
>`colors` · `glyphs` · `more underlining options` · `true color support` · `spinners/progress bars` · `stat aliases` 

**Image rendering support**  
>`kitty` · `sixel` · `iterm`

**AUR helper integration**  
>`yay` · `paru`

**More options, more stats**  
> `--packages` · `--mini` · `--image` · `--json` · `--aur` · `--news` · `--color` 

**Distro and terminal compatibility testing** 
> `Manjaro` · `Endeavor`
> 
> ~`kitty`~ · `alacritty` · `konsole` · `gnome` · `ghostty`

  <br>
  <br>
  
## Screenshots

<p align="center">
  <img src="docs/assets/ghostty-test.png" alt="Demo" width="49%"  />
  <img src="docs/assets/gnome-test.png" alt="Demo" width="49%" />
</p>
<p align="center">
  <img src="docs/assets/demo.gif" alt="Full demo" width="920" height="500" />
</p>

<br>
<br>

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) 


  <br>
  <br>
  <br>
  <br>



