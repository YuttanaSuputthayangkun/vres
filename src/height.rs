/// Returns the closest valid height: even and at least 2.
/// Odd heights are bumped up to the next even number.
pub fn normalize_height(height: u32) -> u32 {
    (height + height % 2).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_heights() {
        assert_eq!(normalize_height(0), 2);
        assert_eq!(normalize_height(1), 2);
        assert_eq!(normalize_height(2), 2);
        assert_eq!(normalize_height(853), 854);
        assert_eq!(normalize_height(1280), 1280);
    }
}
