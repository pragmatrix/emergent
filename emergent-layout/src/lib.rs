use emergent_drawing::DrawingCanvas;

#[test]
fn draw_circle() {
    use emergent_drawing::{Canvas, Color, Drawing, Paint, Radius, Rect, Render, RoundedRect};

    let mut canvas = DrawingCanvas::new();
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000f0));
    let rect = Rect::from(((0, 0).into(), (200, 100).into()));
    canvas.draw(RoundedRect::from((rect, Radius(10.0))), &paint);
    canvas.render();
}
