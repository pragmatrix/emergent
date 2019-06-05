use crate::{constraints, Layout, ResultRef};

struct Border<'a> {
    inner: &'a mut dyn Layout,
    border: [constraints::Linear; 4],
    result: ResultRef<'a>,
}
