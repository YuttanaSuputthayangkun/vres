use std::path::{Path, PathBuf};

/// Builds the output path for `src` by applying `format` ({source_name}, {width}),
/// keeping the source extension. Defaults to the source's directory.
pub fn output_path(src: &Path, out_dir: Option<&Path>, format: &str, width: u32) -> PathBuf {
    let stem = src.file_stem().unwrap_or_default().to_string_lossy();
    let mut name = format
        .replace("{source_name}", &stem)
        .replace("{width}", &width.to_string());
    if let Some(ext) = src.extension() {
        name.push('.');
        name.push_str(&ext.to_string_lossy());
    }
    let dir = out_dir.unwrap_or_else(|| src.parent().unwrap_or(Path::new("")));
    dir.join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_format() {
        let out = output_path(
            Path::new("videos/clip.mp4"),
            None,
            "{source_name}_{width}",
            854,
        );
        assert_eq!(out, Path::new("videos").join("clip_854.mp4"));
    }

    #[test]
    fn custom_format_and_dir() {
        let out = output_path(
            Path::new("videos/clip.mkv"),
            Some(Path::new("out")),
            "small-{width}-{source_name}",
            640,
        );
        assert_eq!(out, Path::new("out").join("small-640-clip.mkv"));
    }
}
