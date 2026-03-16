//! Shared Ogham domain primitives and cross-crate utilities.

/// Returns the crate identifier.
pub fn crate_id() -> &'static str {
    "ogham-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "ogham-core");
    }
}
