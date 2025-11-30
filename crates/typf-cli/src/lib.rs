//! Command-line interface for Typf text rendering
//!
//! This crate provides the CLI that turns Typf's powerful pipeline
//! into commands you can run from your terminal or script.

/// Simple addition function for demonstration
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
