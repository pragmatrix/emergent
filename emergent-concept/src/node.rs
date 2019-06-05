use emergent_drawing::{scalar, Padding, Painting, PaintingCanvas, Rect, Render, Size};
use tears::{Cmd, Model};

pub enum Message {}

pub struct Node {
    pub text: String,
    pub width: scalar,
}

impl Model<Message> for Node {
    fn update(&mut self, msg: Message) -> Cmd<Message> {
        unimplemented!()
    }
}

impl Render<Painting> for Node {
    fn render(&self) {
        // TODO: we want the font to be derived somewhat globally.
        // TODO: the border size should be defined somewhere else.
        let mut canvas = PaintingCanvas::new();
        // TODO: this is used on to the canvas, but probably should be bound to
        //       the Font.
        // let text_size = canvas.measure_text(self.Text, self.width);
        let text_size = Size::from((100.0, 50.0));
        let inner_padding = Padding::from(4.0);

        let paint = Paint::default();

        let mut canvas = canvas.set_mode(Stroke, 4);

        // TODO: everything should be 0,0 based?
        canvas.draw_rounded_rect(text_size + Padding::from((4.0, 4.0)))
    }
}

#[test]
fn simple_node() {
    let node = Node {
        text: String::from("Hello World"),
        width: 200.0,
    };
}
