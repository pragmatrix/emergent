use emergent_drawing::{Circle, Rect};

#[test]
fn draw_circle() {
    use emergent_drawing::{Canvas, Color, Paint, Painting, Render};

    let mut painting: Painting = Painting::new();
    let mut canvas = Canvas::from_target(&mut painting);
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000ff));
    canvas.draw(Circle((100, 100).into(), 100.into()), &paint);
    painting.render();
}
