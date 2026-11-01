/// Returns the closest valid width: even and at least 2.
/// Odd widths are bumped up to the next even number.
pub fn normalize_width(width: u32) -> u32 {
    (width + width % 2).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_widths() {
        assert_eq!(normalize_width(0), 2);
        assert_eq!(normalize_width(1), 2);
        assert_eq!(normalize_width(2), 2);
        assert_eq!(normalize_width(853), 854);
        assert_eq!(normalize_width(1280), 1280);
    }
}
