use crate::node::Node;
use elementary::view::View;
use elementary::{Cmd, Component, Id};

enum Msg {}

struct Concept {
    id: Id,
    nodes: Vec<Node>,
    properties: (),
}

impl Component<(), Msg> for Concept {
    fn id(&self) -> Id {
        self.id
    }

    fn update(&mut self, msg: Msg) -> Cmd {
        unimplemented!()
    }

    fn properties(&mut self) -> &mut () {
        &mut self.properties
    }
}

impl View<Vec<Node>> for Concept {
    fn render(&self) -> Vec<Node> {
        self.nodes.clone()
    }
}
