//! A library to wrap pure functions to cache and optimize them.

use std::cell::Cell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

struct Production {
    /// The current production run (starting with 0, increased before each run).
    /// TODO: use NonZero.
    run: Rc<Cell<u64>>,
}

impl Production {
    /// Create a new Production.
    pub fn new() {
        Production {
            run: Rc::new(Cell::new(0)),
        };
    }

    /// Begin a new production run.
    pub fn next_run(&mut self) -> &mut Production {
        let cell = &*self.run;
        let current = cell.get();
        cell.set(current + 1);
        self
    }
}

/// The cache strategy that is related to production runs.
struct CacheStrategy {
    /// The number of equality matches after which the output should be cached.
    cache_after: usize,
    /// The number runs of inequality matches the output should be removed.
    /// Never 0
    /// TODO: replace by NonZero as soon it's available in stable.
    drop_after: usize,
}

impl Production {
    pub fn produce<I, O>(&self, f: impl Fn(I) -> O) -> impl FnMut(I) -> O
    where
        I: Eq + Hash + Clone,
        O: Clone,
    {
        let run = self.run.clone();
        let mut cache: HashMap<I, O> = HashMap::new();

        move |i| match cache.get(&i) {
            None => {
                let o = f(i.clone());
                cache.insert(i, o.clone());
                o
            }
            Some(o) => o.clone(),
        }
    }
}
