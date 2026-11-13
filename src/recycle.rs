use crate::error::TrashError;
use crate::report::Converted;

/// A source video that could not be moved to the Recycle Bin.
#[derive(Debug)]
pub struct TrashFailed {
    pub src: std::path::PathBuf,
    pub dst: std::path::PathBuf,
    pub error: TrashError,
}

/// What happened to each source passed to [`trash_sources`], in input order.
#[derive(Debug, Default)]
pub struct TrashSummary {
    pub trashed: Vec<Converted>,
    pub failed: Vec<TrashFailed>,
}

impl TrashSummary {
    /// True if every source was moved.
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Moves each `src` to the Recycle Bin (Trash on macOS/Linux), typically the
/// result of [`Resizer::sources_with_outputs`](crate::Resizer::sources_with_outputs).
/// Outputs (`dst`) are left alone. A failure on one file does not stop the rest.
///
/// This deletes files without asking; confirm with the user first.
pub fn trash_sources(items: impl IntoIterator<Item = Converted>) -> TrashSummary {
    let mut summary = TrashSummary::default();
    for item in items {
        match ::trash::delete(&item.src) {
            Ok(()) => summary.trashed.push(item),
            Err(e) => summary.failed.push(TrashFailed {
                src: item.src,
                dst: item.dst,
                error: TrashError(e),
            }),
        }
    }
    summary
}
