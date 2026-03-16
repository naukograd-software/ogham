//! Ogham compiler entry points: parsing, imports, and semantic analysis.

/// Returns the crate identifier.
pub fn crate_id() -> &'static str {
    "ogham-compiler"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "ogham-compiler");
    }
}
