use std::path::{Path, PathBuf};

use regex::Regex;

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

/// Builds a regex matching file stems produced by `format` at `width`,
/// with any {source_name} (one or more characters).
pub fn output_pattern(format: &str, width: u32) -> Result<Regex, regex::Error> {
    let pattern = regex::escape(format)
        .replace(r"\{source_name\}", ".+")
        .replace(r"\{width\}", &width.to_string());
    Regex::new(&format!("^{pattern}$"))
}

/// Returns true if the file stem of `path` matches `pattern`.
pub fn is_output(path: &Path, pattern: &Regex) -> bool {
    path.file_stem()
        .is_some_and(|stem| pattern.is_match(&stem.to_string_lossy()))
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

    #[test]
    fn matches_default_output_names() {
        let pattern = output_pattern("{source_name}_{width}", 720).unwrap();
        assert!(is_output(Path::new("dir/clip_720.mp4"), &pattern));
        assert!(is_output(Path::new("3314267-01_720.mp4"), &pattern));
        assert!(is_output(Path::new("my_clip_720.MKV"), &pattern));
        assert!(!is_output(Path::new("clip_1280.mp4"), &pattern));
        assert!(!is_output(Path::new("clip_7200.mp4"), &pattern));
        assert!(!is_output(Path::new("clip.mp4"), &pattern));
        assert!(!is_output(Path::new("_720.mp4"), &pattern));
    }

    #[test]
    fn escapes_literal_parts() {
        let pattern = output_pattern("small.{width}(x){source_name}", 640).unwrap();
        assert!(is_output(Path::new("small.640(x)clip.mp4"), &pattern));
        assert!(!is_output(Path::new("smallX640(x)clip.mp4"), &pattern));
    }
}
