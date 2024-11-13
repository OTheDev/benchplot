/*
Copyright 2024 Owain Davies
SPDX-License-Identifier: Apache-2.0 OR MIT
*/

/// Function to check if all items in an iterator are equal.
///
/// If the iterator is empty, this function returns `true`.
pub fn all_items_equal<I, T>(iter: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: PartialEq,
{
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        iter.all(|item| item == first)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_iterator() {
        let empty: Vec<i32> = vec![];
        assert!(all_items_equal(empty));
    }

    #[test]
    fn test_single_element() {
        let single = vec![42];
        assert!(all_items_equal(single));
    }

    #[test]
    fn test_all_elements_equal() {
        let equal_elements = vec![7, 7, 7, 7];
        assert!(all_items_equal(equal_elements));
    }

    #[test]
    fn test_different_elements() {
        let different_elements = vec![1, 2, 1, 1];
        assert!(!all_items_equal(different_elements));
    }
}
