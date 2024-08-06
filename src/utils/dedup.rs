use std::{collections::HashSet, hash::Hash};

/// An iterator for dedupping elements.
pub struct Dedup<I: Iterator>
where
    I::Item: Hash + Eq + Clone,
{
    base: I,
    set: HashSet<I::Item>,
}

impl<I: Iterator> Iterator for Dedup<I>
where
    I::Item: Hash + Eq + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.base.next() {
            if !self.set.contains(&next) {
                self.set.insert(next.clone());
                return Some(next);
            }
        }
        None
    }
}

pub trait DedupIterator: Iterator + Sized
where
    Self::Item: Hash + Eq + Clone,
{
    /// Returns an iterator that depuplicates elements.
    fn dedup(self) -> Dedup<Self> {
        Dedup {
            base: self,
            set: HashSet::new(),
        }
    }
}

impl<I: Iterator> DedupIterator for I where I::Item: Hash + Eq + Clone {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_vector_iter() {
        let nums = vec![1, 2, 3, 3, 3, 4, 1, 3, 2];
        let deduped = nums.iter().dedup().collect::<Vec<&i32>>();
        assert_eq!(deduped, vec![&1, &2, &3, &4])
    }
}
