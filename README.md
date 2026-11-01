# vres

Resize videos to a given width with ffmpeg, keeping the aspect ratio. Shows a progress bar with an ETA for each conversion.

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
vres -w 1280 -r 24 videos/       # every video in videos/ at 1280px, 24 fps
vres -R -o out -f "{source_name}-small" videos/
```

| Option | Env var | Default | Description |
|---|---|---|---|
| `-w, --width` | `VRES_WIDTH` | `720` | Target width. Odd values are bumped up to the next even number. |
| `-r, --fps` | `VRES_FPS` | `30` | Output frame rate. |
| `-o, --output` | `VRES_OUTPUT` | source's folder | Output directory (created if missing). |
| `-f, --format` | `VRES_FORMAT` | `{source_name}_{width}` | Output file name; the source extension is kept. |
| `-R, --recursive` | `VRES_RECURSIVE` | off | Scan subdirectories too. |
| `--include-outputs` | `VRES_INCLUDE_OUTPUTS` | off | Also process files that already match the output name at the current width. |
| `--overwrite` | `VRES_OVERWRITE` | off | Replace existing output files instead of skipping them. |

Files are skipped when they're narrower than the target width (no upscaling), when the output already exists, or when they look like a previous output (e.g. `clip_720.mp4`).

## Configuration

Any option can be set in a `.env` file in the current directory. See [`.env.example`](.env.example). Command-line arguments take precedence over environment variables, which take precedence over `.env`.

## License

[MIT](LICENSE)
