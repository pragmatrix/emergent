use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;

///! Simple generation aware cache used in situations where the cache is used in phases that usually
///! do the same or similar lookups.
pub struct GenerationCache<K, V> {
    max_generations: usize,
    current_generation: usize,

    cache: RefCell<HashMap<K, (V, usize)>>,
}

impl<K, V> GenerationCache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new(max_generations: usize) -> Self {
        assert!(max_generations > 0);
        Self {
            max_generations,
            current_generation: 0,
            cache: HashMap::new().into(),
        }
    }

    /// Marks the  current generation as done.
    ///
    /// This flushes generations that are older than max_generations.
    pub fn mark_generation_done(&mut self) {
        self.current_generation += 1;
        let oldest_generation = max(0, self.current_generation - self.max_generations);
        if oldest_generation > 0 {
            self.cache
                .get_mut()
                .retain(|_k, (_, gen)| *gen >= oldest_generation);
        }
    }

    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    pub fn resolve<Q: ?Sized>(&self, k: &Q, f: impl FnOnce() -> V) -> V
    where
        K: Borrow<Q>,
        Q: ToOwned<Owned = K>,
        Q: Eq + Hash,
    {
        let mut cache = self.cache.borrow_mut();
        match cache.get_mut(k) {
            Some(v) => {
                let (v, g) = v;
                *g = self.current_generation;
                v.clone()
            }
            None => {
                let k = k.to_owned();
                let v = f();
                cache.insert(k, (v.clone(), self.current_generation));
                v
            }
        }
    }
}
