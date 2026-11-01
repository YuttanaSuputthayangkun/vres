use std::fs;
use std::path::{Path, PathBuf};

use crate::error::InputError;

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "mov", "avi", "webm", "m4v", "wmv", "flv", "mpg", "mpeg", "ts",
];

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| VIDEO_EXTENSIONS.iter().any(|v| v.eq_ignore_ascii_case(ext)))
}

/// Expands the given inputs into a list of video files.
/// Files are taken as-is; directories are scanned for video files.
pub fn collect_videos(inputs: &[PathBuf], recursive: bool) -> Result<Vec<PathBuf>, InputError> {
    let mut videos = Vec::new();
    for input in inputs {
        if input.is_file() {
            videos.push(input.clone());
        } else if input.is_dir() {
            scan_dir(input, recursive, &mut videos)?;
        } else {
            return Err(InputError::NotFound(input.clone()));
        }
    }
    Ok(videos)
}

fn scan_dir(dir: &Path, recursive: bool, videos: &mut Vec<PathBuf>) -> Result<(), InputError> {
    let entries = fs::read_dir(dir).map_err(|source| InputError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })?;
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            if recursive {
                scan_dir(&path, recursive, videos)?;
            }
        } else if is_video(&path) {
            videos.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_tree(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("vres_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("sub")).unwrap();
        for f in ["a.mp4", "b.MKV", "notes.txt", "sub/c.mov"] {
            fs::write(root.join(f), b"").unwrap();
        }
        root
    }

    #[test]
    fn scans_top_level_only() {
        let root = temp_tree("flat");
        let videos = collect_videos(std::slice::from_ref(&root), false).unwrap();
        assert_eq!(videos, vec![root.join("a.mp4"), root.join("b.MKV")]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scans_recursively() {
        let root = temp_tree("recursive");
        let videos = collect_videos(std::slice::from_ref(&root), true).unwrap();
        assert_eq!(
            videos,
            vec![
                root.join("a.mp4"),
                root.join("b.MKV"),
                root.join("sub").join("c.mov")
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn errors_on_missing_input() {
        assert!(matches!(
            collect_videos(&[PathBuf::from("does/not/exist.mp4")], false),
            Err(InputError::NotFound(_))
        ));
    }
}
