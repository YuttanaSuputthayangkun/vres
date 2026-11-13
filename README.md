# vres

Resize videos to a given height with ffmpeg, keeping the aspect ratio. Shows a progress bar with an ETA for each conversion.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- `ffmpeg` and `ffprobe` on your `PATH`

## Install

```sh
cargo install --path .
```

## Usage

```sh
vres [OPTIONS] <INPUTS>...
```

`INPUTS` can be video files, directories, or both.

```sh
vres clip.mp4                    # -> clip_720.mp4 next to the source
vres -H 1080 -r 24 videos/       # every video in videos/ at 1080px, 24 fps
vres -R -o out -f "{source_name}-small" videos/
```

| Option | Env var | Default | Description |
|---|---|---|---|
| `-H, --height` | `VRES_HEIGHT` | `720` | Target height. Odd values are bumped up to the next even number. |
| `-r, --fps` | `VRES_FPS` | `30` | Output frame rate. |
| `-o, --output` | `VRES_OUTPUT` | source's folder | Output directory (created if missing). |
| `-f, --format` | `VRES_FORMAT` | `{source_name}_{height}` | Output file name; the source extension is kept. |
| `-R, --recursive` | `VRES_RECURSIVE` | off | Scan subdirectories too. |
| `--include-outputs` | `VRES_INCLUDE_OUTPUTS` | off | Also process files that already match the output name at the current height. |
| `--overwrite` | `VRES_OVERWRITE` | off | Replace existing output files instead of skipping them. |
| `-q, --quiet` | `VRES_QUIET` | off | Only print errors. |
| `--json` | `VRES_JSON` | off | Print a JSON summary to stdout when done (status lines move to stderr). |

With `--json`, stdout contains only an object with `converted`, `skipped` and `failed` lists. Each entry has `src` and `dst`; skipped entries add a `reason` (`matches_output_format`, `too_short` with `height`/`target`, or `output_exists`), and failed entries add an `error` message.

Files are skipped when they're shorter than the target height (no upscaling), when the output already exists, or when they look like a previous output (e.g. `clip_720.mp4`).

## Clean up sources

```sh
vres clean [OPTIONS] <INPUTS>...
```

Moves source videos whose output already exists (e.g. `clip.mp4` when `clip_720.mp4` is next to it) to the Recycle Bin (Trash on macOS/Linux). Outputs are never touched, and empty outputs (e.g. from a failed conversion) don't count. Use the same `-H`, `-f` and `-o` as the resize run so the outputs are found; the `VRES_*` variables and `.env` apply here too.

```sh
vres -H 1080 videos/          # resize
vres clean -H 1080 videos/    # lists the originals, then asks before trashing them
```

| Option | Description |
|---|---|
| `-H`, `-o`, `-f`, `-R`, `-q`, `--json` | Same as above. |
| `-y, --yes` | Skip the confirmation prompt. Required when stdin is not a terminal (scripts, pipes). Has no env var, so it can't be turned on from `.env`. |

Without `--yes`, the matching files are listed and you're asked `Move N files to the Recycle Bin? [y/N]`; only `y` or `yes` proceeds. With `--json`, stdout contains `{ "trashed": [...], "failed": [...] }`; each entry has `src` and `dst`, and failed entries add an `error`. The exit code is non-zero if any file could not be moved.

A file literally named `clean` must be passed as `./clean` when resizing.

## Configuration

Any option can be set in a `.env` file in the current directory. See [`.env.example`](.env.example). Command-line arguments take precedence over environment variables, which take precedence over `.env`.

## Library

vres is also a library. Turn off the default `cli` feature to skip the CLI-only dependencies (clap, dotenvy, indicatif):

```toml
vres = { git = "https://github.com/YuttanaSuputthayangkun/vres", default-features = false }
```

```rust
use vres::Resizer;

let resizer = Resizer::builder().height(1080).fps(24).recursive(true).build()?;
let summary = resizer.run(&["videos/"], &mut ())?;
```

Pass your own `vres::Reporter` instead of `&mut ()` to receive skip, progress and failure events.

To clean up originals, `Resizer::sources_with_outputs` lists sources whose output exists. Enable the `trash` feature to move them to the Recycle Bin. It deletes without asking, so confirm with the user first:

```toml
vres = { git = "https://github.com/YuttanaSuputthayangkun/vres", default-features = false, features = ["trash"] }
```

```rust
let found = resizer.sources_with_outputs(&["videos/"])?;
let summary = vres::trash_sources(found);
for failed in &summary.failed {
    eprintln!("{}: {}", failed.src.display(), failed.error);
}
```

## License

[MIT](LICENSE)
