use crate::{constraints, Layout, ResultRef};

struct Border<'a> {
    center: &'a mut Layout,
    border: [constraints::Dim; 4],
    result: ResultRef<'a>,
}
