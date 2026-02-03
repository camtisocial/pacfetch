# Title Customization Design (Issues #12, #13, #14)

## Overview

This design covers three related enhancements for title customization:
- **#12**: Configurable title width (title, content, or fixed value)
- **#13**: Embedded title style with custom line characters and caps
- **#14**: Multiple named titles that can be positioned anywhere in the stats array

## Config Structure

Titles are defined under `[display.titles.{name}]` where `{name}` is user-chosen:

```toml
[display.titles.header]
text = "default"              # "default" | "pacman_ver" | "pacfetch_ver" | custom string
text_color = "bright_yellow"  # any color name, hex, or "none"
line_color = "none"           # any color name, hex, or "none"
style = "stacked"             # "stacked" | "embedded"
width = "title"               # "title" | "content" | integer (e.g., 40)
align = "left"                # "left" | "center" | "right"
line = "-"                    # line character(s) - can be multi-char pattern
left_cap = ""                 # left cap for embedded style (e.g., "╭", "├")
right_cap = ""                # right cap for embedded style (e.g., "╮", "┤")
```

### Defaults

| Field | Default Value |
|-------|---------------|
| `text` | `""` (empty) |
| `text_color` | `"bright_yellow"` |
| `line_color` | `"none"` |
| `style` | `"stacked"` |
| `width` | `"title"` |
| `align` | `"left"` for stacked, `"center"` for embedded |
| `line` | `"-"` |
| `left_cap` | `""` |
| `right_cap` | `""` |

## Stats Array Usage

Reference titles using dot notation matching the config structure:

```toml
stats = [
    "title.header",
    "installed",
    "upgradable",
    "title.divider",
    "orphaned_packages",
    "cache_size",
    "title.footer",
]
```

## Rendering Behavior

### Stacked Style

Text on one line, line below (line always below regardless of position):

```
Pacman v7.1.0 - libalpm v16.0.1
-------------------------------
```

### Embedded Style

Text centered within the line with optional caps:

```
╭─────────── Pacman v7.1.0 ───────────╮
```

### Empty Text

When `text = ""`, render only the line (useful for dividers):

```
├─────────────────────────────────────┤
```

## Width Calculation

### `width = "title"`
Line matches the text length.

### `width = "content"`
Line matches the longest element across ALL content:
- All title texts (accounting for embedded padding)
- All stat lines (label + glyph + value)

### `width = 40` (fixed)
Line is exactly 40 characters (or specified number).

## Alignment

Applies to text positioning within the calculated width:
- `"left"` - text at left edge
- `"center"` - text centered
- `"right"` - text at right edge

Default depends on style:
- Stacked defaults to `"left"`
- Embedded defaults to `"center"`

## Two-Pass Rendering Algorithm

For `width = "content"` titles:

**Pass 1 - Calculate widths:**
1. For each stacked title: `text.len()`
2. For each embedded title: `left_cap.len() + min_padding + text.len() + min_padding + right_cap.len()`
3. For each stat line: `label.len() + glyph.len() + value.len()`
4. Find maximum width

**Pass 2 - Render at calculated width:**
All titles with `width = "content"` use the same calculated max width.

## Edge Cases

| Case | Behavior |
|------|----------|
| Missing title reference (`title.foo` not defined) | Skip silently, log warning |
| Empty `text` | Render line only |
| Embedded with no caps | Centered text in line |
| Text longer than fixed width | Render as-is (overflows) |
| Multi-char line pattern | Repeat full pattern |

## Migration from Old Config

The old `[display.title]` format is deprecated:

- When found, log deprecation warning to `~/.cache/pacfetch/pacfetch.log`
- `"title"` in stats array maps to `[display.title]` if it exists
- Document migration path in `default_config.toml`

## Logging

Add file logging to `~/.cache/pacfetch/pacfetch.log` for:
- Deprecation warnings (old config format)
- Missing title reference warnings
- Other non-fatal issues

## Files to Modify

1. **`src/stats.rs`** - Handle dynamic `title.{name}` parsing
2. **`src/config.rs`** - New config structures with HashMap for titles
3. **`src/ui/mod.rs`** - Two-pass rendering for width calculation
4. **`src/log.rs`** (new) - Simple file logging
5. **`default_config.toml`** - Updated format with documentation

## Example Output

```toml
[display.titles.header]
text = "default"
style = "stacked"
width = "content"

[display.titles.divider]
text = ""
style = "embedded"
line = "─"
left_cap = "├"
right_cap = "┤"

[display.titles.footer]
text = "pacfetch"
style = "embedded"
line = "─"
left_cap = "╰"
right_cap = "╯"
```

Output:
```
         Pacman v7.1.0 - libalpm v16.0.1
------------------------------------------------
Installed: 1268
Upgradable: 4
├──────────────────────────────────────────────┤
Package Cache: 0.00 MiB
Orphaned Packages: 12 (148.81 MiB)
╰─────────────────pacfetch─────────────────────╯
```
