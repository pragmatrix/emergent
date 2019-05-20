#[test]
fn draw_circle() {
    use emergent_drawing::{Canvas, Color, Paint, Painting, Render};

    let mut painting: Painting = Painting::new();
    let mut canvas = Canvas::from_target(&mut painting);
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff000010));
    canvas.draw_circle((100, 100), 100, &paint);
    painting.render();
}
