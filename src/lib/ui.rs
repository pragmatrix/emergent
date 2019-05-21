#[test]
fn draw_circle() {
    use emergent_drawing::PaintingCanvas;
    use emergent_drawing::{Canvas, Circle, Color, Paint};

    let mut canvas = PaintingCanvas::new();
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000ff));
    canvas.draw(Circle((100, 100).into(), 100.into()), &paint);
    canvas.render();
}
