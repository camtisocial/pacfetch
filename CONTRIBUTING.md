# Contributing to pacfetch

## Project Structure

```
src/
├── main.rs      # CLI parsing , entry point
├── pacman.rs    # Data collection
├── stats.rs     # Stat definitions, labels, formatting
├── config.rs    # Config file parsing
├── util.rs      # Helper functions
└── ui/
    ├── mod.rs   # Display logic, colors, layout
    └── ascii.rs # ASCII art loading and built-ins
```

### Adding a New Stat

**`src/stats.rs`**
- Add new stat to `StatId` enum
- Add label in `label()`
- Add formatting in `format_value()`

**`src/pacman.rs`**
- Add field to `PacmanStats` struct
- Fetch the data in `get_stats()`

**`src/config.rs`**
- Add to `default_stats()` if shown by default

**`default_config.toml`**
- Add the stat name to the list of available stats in the default config

<br>

### Adding a Config Option (ex. `ascii_color = "blue"`)
- Add field to `DisplayConfig` in `src/config.rs`
- Add default function if needed or `#[serde(default)]`
- Update `default_config.toml`
- Use the value in `src/ui/mod.rs`

<br>


### Adding a CLI Flag

- Flags are defined with clap in `src/main.rs`
- Look at existing flags like `--ascii` or `--local` for examples

<br>

### Before Submitting

```bash
cargo fmt && cargo clippy
./testing/smoke-test.sh
```
