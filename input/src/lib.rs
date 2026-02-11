//! Input parsing and validation for Typf text processing
//!
//! The first stage of the Typf pipeline. Takes raw text and transforms
//! it into structured data the rest of the pipeline can work with.

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
