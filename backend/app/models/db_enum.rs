//! Helpers for DB integer ↔ API UPPER_SNAKE enums.

/// Normalize wire input: trim and uppercase ASCII (legacy lowercase accepted before parse).
pub fn normalize_wire(s: &str) -> String {
    s.trim().to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_trims_and_uppercases() {
        assert_eq!(normalize_wire("  brasil  "), "BRASIL");
    }
}
