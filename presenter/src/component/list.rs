#![allow(dead_code)]

use emergent_drawing::Vector;
use emergent_presentation::Presentation;

/// A list component. Structures nested presentations in a scrolling list
/// and renders only those which are visible.

enum ListMsg {
    Pan(Vector),
}

struct List {
    /// Index to the top element.
    top_element: usize,

    /// The top offset of the first element.
    /// Effectively the transformation of the nested elements.
    top_offset: usize,
}

#[derive(Clone)]
struct Cached<Key> {
    key: Key,
    presentation: Presentation,
}

/*
impl List {
    pub fn new() -> List {
        Self {
            top_offset: 0,
            top_element: 0,
        }
    }

    pub fn present<Msg, Key>(
        &mut self,
        p: &mut Context<Msg>,
        keys: &[Key],
        f: impl Fn(&mut Context<Msg>, &Key, usize),
    ) where
        Key: Clone + 'static,
    {
        let cache: &mut Vec<Option<Cached<Key>>> = p.resolve(|| Vec::new());
        // trim cache.
        if cache.len() > keys.len() {
            cache.resize(keys.len(), None);
        }

        let mut top = self.top_offset;
        let element_index = self.top_element;
        let height = 1000; //TODO
        while top < height && element_index < keys.len() {
            if cache.len() < element_index {
                cache.resize(element_index + 1, None);
            }

            // f(p, &keys[element_index], element_index)
        }
    }
}

*/
