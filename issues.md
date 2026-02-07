# Planned GitHub Issues

---

## Bugs

### ASCII art with embedded ANSI codes breaks stat alignment

Custom ASCII art containing ANSI color codes causes stats to misalign. Lines with more escape codes get less padding, making stats appear at different columns.

**Expected:**
```
 ❝ Quote ❞           Installed: 1328
    _                Upgradable: 18
   / `._             Last Update: 2 days
```

**Actual:**
```
 ❝ Quote ❞           Installed: 1328
    _                                    Upgradable: 18
   / `._                       Last Update: 2 days
```

**Root cause:**

`normalize_width()` in `src/ui/ascii.rs` uses `chars().count()` to calculate line widths for padding. ANSI escape codes like `\x1b[32m` are counted as 5 characters but have 0 visible width, causing incorrect padding calculations.

**Fix:**

Use `util::strip_ansi()` to calculate visible width before padding:

```rust
fn normalize_width(lines: Vec<String>) -> Vec<String> {
    use crate::util;
    let max_width = lines
        .iter()
        .map(|s| util::strip_ansi(s).chars().count())
        .max()
        .unwrap_or(0);
    lines
        .into_iter()
        .map(|line| {
            let visible_len = util::strip_ansi(&line).chars().count();
            let padding = max_width - visible_len;
            // ...
        })
        .collect()
}
```

---

## New Stats

### Add `disk` stat with configurable path

Add a `disk` stat showing disk usage in fastfetch style with color-coded percentage.

**Output:**
```
Disk (/): 43.24 GiB / 48.91 GiB (88%) - ext4
```

**Config:**
```toml
stats = ["disk", ...]

[disk]
path = "/"  # or "~/" or custom mount point
```

**Color thresholds (hardcoded):**
- Green: < 70%
- Yellow: 70-90%
- Red: > 90%

**Notes:**
- Use `nix` crate for direct `statvfs` syscall:
  ```rust
  use nix::sys::statvfs::statvfs;

  let stat = statvfs("/")?;
  let total = stat.blocks() * stat.fragment_size();
  let free = stat.blocks_available() * stat.fragment_size();
  ```
- Why `nix` over subprocess (`df`)?
  - Direct syscall, no process spawn overhead
  - Type-safe: no parsing `df` output strings
  - Consistent with codebase philosophy (libalpm over `pacman -Q`)
  - More reliable: `df` output varies by locale/version
- Get filesystem type by parsing `/proc/mounts` (match on mount point)
- `statvfs` doesn't expose fs type directly

---

### Add `aur` stat

Show AUR/foreign package stats. Auto-detect installed helper (yay, paru).

**Output:**
```
AUR Packages: 47 (3 upgradable)
```

**Config:**
```toml
stats = ["aur", ...]

[aur]
helper = "auto"  # or "yay", "paru"
```

**Notes:**
- Foreign packages via `pacman -Qm`
- Upgradable count requires AUR helper
- When used with `-Syu`, run helper instead of pacman

---

### Add `flatpak` stat

Show Flatpak package count (like neofetch does with package managers).

**Output:**
```
Flatpak: 12 packages
```

**Notes:**
- Simple subprocess: `flatpak list | wc -l`
- Good first issue

---

### Add `db_sync` stat

Show when package databases were last synced.

**Output:**
```
Last Sync: 2 hours ago
```

**Notes:**
- Parse timestamp from `/var/lib/pacman/sync/*.db`
- Good first issue

---

## Display Options

### Add `disk_inline` option

Append disk usage to Download Size instead of a separate `disk` stat line.

**Output:**
```
Download Size: 800 MiB (88% full)
```

Percentage is color-coded using the same thresholds as the `disk` stat:
- Green: < 70%
- Yellow: 70-90%
- Red: > 90%

**Config:**
```toml
[display]
disk_inline = true
```

**Notes:**
- Uses `[disk].path` setting for which mount point to check
- Requires `disk` stat implementation (shares the statvfs logic)
- Good for users who want disk info without a dedicated line

---

### Add `--mini` display mode

Compact output with smaller ASCII and essential stats only (pfetch-style).

**Notes:**
- Minimal ASCII or none
- Show only: installed, upgradable, last_update
- Could be flag or config option

---

### Add `--json` output

Machine-readable output for scripting.

**Output:**
```json
{"installed": 1247, "upgradable": 12, "download_size": 845000000}
```

**Notes:**
- Useful for status bars, scripts, CI

---

### Make `title` a configurable stat

Currently the title (version string + underline) is hardcoded to appear at the top. Make it a stat that can be positioned, removed, or reordered in the stats array.

**Config:**
```toml
stats = [
    "title",        # can be moved or removed
    "installed",
    "upgradable",
    # ...
]
```

**Current behavior:**
- Title always renders first, outside the stats loop

**New behavior:**
- `"title"` is a `StatId` variant
- Rendered in order with other stats
- Omitting it from the array removes it entirely

**Implementation notes:**
- Add `Title` variant to `StatId` enum in `src/stats.rs`
- Move title rendering logic into `StatId::format_value()` or handle as special case in display loop
- Update `default_stats()` to include `StatId::Title` at the start

---

### Make `color_palette` a configurable stat

Currently the color palette blocks are hardcoded to appear at the bottom. Make it a stat that can be positioned, removed, or reordered.

**Config:**
```toml
stats = [
    "title",
    "installed",
    # ...
    "color_palette",  # can be moved or removed
]
```

**Current behavior:**
- Color rows always render last, outside the stats loop

**New behavior:**
- `"color_palette"` is a `StatId` variant
- Rendered in order with other stats
- Omitting it from the array removes it entirely

**Implementation notes:**
- Add `ColorPalette` variant to `StatId` enum in `src/stats.rs`
- Move color palette rendering into the stats loop
- Update `default_stats()` to include `StatId::ColorPalette` at the end

---

## Customization

### ASCII color

Allow setting ASCII art color via config. Establishes color parsing utility that future color options will reuse.

**Current behavior:**
- ASCII art is hardcoded to cyan (`.cyan()` in `ui/mod.rs`)

**New behavior:**

| `ascii_color` | Art has embedded ANSI | Result |
|---------------|----------------------|--------|
| `"yellow"` | No | Yellow applied |
| `"yellow"` | Yes | Yellow applied (overrides embedded) |
| `"NONE"` | No | Plain text (no color) |
| `"NONE"` | Yes | Embedded colors passthrough |

Setting a color always applies that color. `"NONE"` means "don't apply any color" - embedded ANSI codes pass through, or plain text if no embedded codes.

**Config:**
```toml
[display]
ascii_color = "yellow"
```

**Supported colors:**
- `NONE` - no color applied, passthrough for embedded ANSI
- Named: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- Bright variants: `bright_red`, `bright_green`, etc.
- Hex: `#ffcc00`

Named colors use the terminal's 16-color palette - the actual color displayed depends on the user's terminal theme (e.g., "red" shows as Solarized red, Dracula red, etc.). Hex colors are absolute RGB values, displayed the same regardless of theme (requires true color terminal support).

**Default:** `cyan` (preserves current behavior)

**Implementation notes:**

1. Create `src/color.rs` utility module:
   ```rust
   use crossterm::style::Color;

   /// Returns None for "NONE", Some(Color) for valid colors
   pub fn parse_color(s: &str) -> Option<Color> {
       match s.to_lowercase().as_str() {
           "none" => None,  // explicit no-color
           "black" => Some(Color::Black),
           "red" => Some(Color::DarkRed),
           "green" => Some(Color::DarkGreen),
           // ... etc
           "bright_red" => Some(Color::Red),
           // ... etc
           s if s.starts_with('#') => parse_hex(s),
           _ => None,
       }
   }

   fn parse_hex(s: &str) -> Option<Color> {
       let hex = s.trim_start_matches('#');
       if hex.len() != 6 { return None; }
       let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
       let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
       let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
       Some(Color::Rgb { r, g, b })
   }
   ```

2. Add `ascii_color` to `DisplayConfig` in `src/config.rs`:
   ```rust
   #[serde(default = "default_ascii_color")]
   pub ascii_color: String,
   ```

3. Update `ui/mod.rs`:
   ```rust
   match color::parse_color(&config.display.ascii_color) {
       Some(color) => println!(" {}   {}", art_line.with(color), stat_line),
       None => println!(" {}   {}", art_line, stat_line),  // NONE or invalid
   }
   ```

4. Add `has_ansi()` helper to `src/util.rs` (near `strip_ansi`):
   ```rust
   pub fn has_ansi(s: &str) -> bool {
       s.contains("\x1b[")
   }
   ```
   (Used elsewhere, not needed for this feature since color always applies or never applies)

---

### Add `--color` CLI flag

Override ASCII art color for a single run.

```bash
pacfetch --color green
pacfetch --color "#ff8800"
pacfetch --color none
```

Depends on: ASCII color config issue (uses same `parse_color()` function)

**Implementation notes:**

1. Add to clap args in `main.rs`:
   ```rust
   #[arg(long)]
   color: Option<String>,
   ```

2. Flag overrides config value:
   ```rust
   let ascii_color = cli.color
       .as_ref()
       .unwrap_or(&config.display.ascii_color);
   ```

---

### Title customization

Configure title text, color, and underline style.

**Current behavior:**
- Title shows pacman version string (e.g., `Pacman v7.1.0 - libalpm v16.0.1`)
- Underline is `-` repeated to match title length

**Config:**
```toml
[display.title]
text = "version"          # "version" | "pacman" | "pacfetch" | custom string
color = "yellow"
underline_char = "─"      # repeated character (or "-", "=", etc.)
underline_start = ""      # optional start cap
underline_end = ""        # optional end cap
underline_color = "white"
```

**Underline styles using box-drawing characters:**
```
# underline_char = "-" (default)
-------------------------------

# underline_char = "─" (continuous line)
───────────────────────────────

# underline_char = "─", underline_start = "├", underline_end = "┤"
├─────────────────────────────┤

# underline_char = "─", underline_start = "┌", underline_end = "┐"
┌─────────────────────────────┐

# underline_char = "═" (double line)
═══════════════════════════════
```

**Title text options:**
- `"version"` - pacman version string (current default)
- `"pacman"` - just "Pacman"
- `"pacfetch"` - just "pacfetch"
- Any custom string - displayed as-is

**Implementation notes:**
1. Add `TitleConfig` struct to `config.rs`
2. Build underline: `format!("{}{}{}", start, char.repeat(width), end)`
3. Apply colors using `parse_color()` from color module

---

### Custom underlines with unicode caps and section separators

Expand title underlines to support unicode box-drawing characters with end caps, and add configurable section separators for visual grouping.

**Motivation:**

The basic underline config (`underline_char = "-"`) is limited. Users should be able to create more visually interesting borders like:
- `╭──────────╮` (rounded caps)
- `├──────────┤` (box-drawing style)
- `|----------|` (ASCII style)
- `═══════════` (double lines)

**Config:**
```toml
[display.title.underline]
line = "─"                    # Main line character (repeated to fill width)
left_cap = "╭"                # Optional left end piece
right_cap = "╮"               # Optional right end piece
color = "white"

# For section separators (reusable pattern)
[display.separators.default]
line = "─"
left_cap = "├"
right_cap = "┤"
color = "white"

# Bottom closer for the display box
[display.bottom]
line = "─"
left_cap = "╰"
right_cap = "╯"
color = "white"
```

**Example output:**
```
upkg v0.1.0
╭──────────────────╮
 Installed: 1234
 Upgradable: 5
├──────────────────┤
 Cache: 2.3 GB
 Orphans: 12
╰──────────────────╯
```

**Presets (optional enhancement):**
```toml
[display.title.underline]
preset = "rounded"  # Uses ╭─╮ automatically
```

Preset options:
- `rounded`: `╭─╮` / `╰─╯`
- `box`: `┌─┐` / `└─┘`
- `double`: `╔═╗` / `╚═╝`
- `ascii`: `+--+` or `|--|`
- `heavy`: `┏━┓` / `┗━┛`

**Design notes:**
- All cap fields optional (defaults to no caps)
- Line character defaults to `-` for ASCII compatibility
- Width auto-calculates based on content or is configurable
- Presets provide common patterns without manual specification

---

### Per-element colors

Set colors for labels, values, and glyphs independently.

**Config:**
```toml
[display.colors]
label = "yellow"
value = "white"
glyph = "white"
```

---

### Separator customization

Change the glyph between label and value.

**Current behavior:**
- Hardcoded `: ` between label and value (e.g., `Installed: 1328`)

**New behavior:**
- User can customize the separator character and spacing

**Config:**
```toml
[display.glyph]
separator = ":"           # or "→" or "|" or " ->"
spacing = true            # "Label: value" vs "Label:value"
```

**Examples:**
```
# separator = ":", spacing = true (default)
Installed: 1328

# separator = "→", spacing = true
Installed → 1328

# separator = " |", spacing = false
Installed | 1328

# separator = "", spacing = false (no separator)
Installed 1328
```

**Implementation notes:**
1. Add to `DisplayConfig` in `config.rs`:
   ```rust
   #[serde(default)]
   pub glyph: GlyphConfig,

   #[derive(Deserialize)]
   pub struct GlyphConfig {
       #[serde(default = "default_separator")]
       pub separator: String,  // default ":"
       #[serde(default = "default_spacing")]
       pub spacing: bool,      // default true
   }
   ```

2. Update stat formatting in `ui/mod.rs`:
   ```rust
   let sep = if config.display.glyph.spacing {
       format!("{} ", config.display.glyph.separator)
   } else {
       config.display.glyph.separator.clone()
   };
   println!("{}{}{}", label, sep, value);
   ```

**Notes:**
- Good first issue

---

### Custom stat labels

Override default label text for any stat.

**Config:**
```toml
[display.labels]
installed = "Pkgs"
upgradable = "Updates"
download_size = "Download"
```

**Notes:**
- Enables localization or personal preference

---

## Compatibility Testing

### Terminal compatibility

Test and document compatibility with various terminals.

- [ ] Alacritty
- [ ] Konsole
- [ ] Ghostty
- [ ] GNOME Terminal
- [ ] Xfce Terminal
- [ ] Terminator
- [x] Kitty

**Notes:**
- Good first issue, no code required
- Document any rendering issues

---

### Arch distro compatibility

Test on Arch-based distributions.

- [ ] EndeavourOS
- [ ] Manjaro
- [ ] Garuda
- [ ] ArcoLinux

**Notes:**
- Good first issue
- Check for path differences, pacman config variations

---

## Utility Flags

### Add `-Sc` cache clean option

Wrapper to clean package cache after displaying stats.

**Notes:**
- Run `pacman -Sc` with confirmation
- Good first issue

---

### Add orphan removal option

Quick way to remove orphaned packages.

**Notes:**
- Run `pacman -Rns $(pacman -Qtdq)`
- Show orphan list before confirmation
- Good first issue
