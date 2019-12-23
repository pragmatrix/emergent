/*
/// A list component. Structures nested presentations in a scrolling list
/// and renders only those which are visible.

enum ListMsg {
    Pan(Vector),
}

struct List<Msg, Keys> {
    /// The top offset of the first element.
    /// Effectively the transformation of the nested elements.
    top_offset: usize,

    /// Index to the top element.
    top_element: usize,

    HashSet<Keys> keys,
}

impl List<Msg> {
    pub fn new() -> List<Msg> {
        Self {
            top_offset: 0,
            top_element: 0
        }
    }

    pub fn present(&mut self, p: &mut Presenter<Msg>, f: impl Fn(&mut Presenter, usize)) {




    }
}
*/
