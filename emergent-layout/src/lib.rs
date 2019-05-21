#[test]
fn draw_circle() {
    use emergent_drawing::{Canvas, Color, Paint, Painting, Radius, Rect, Render, RoundedRect};

    let mut painting: Painting = Painting::new();
    let mut canvas = Canvas::from_target(&mut painting);
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000f0));

    let rect = Rect::from(((0, 0).into(), (200, 100).into()));
    canvas.draw(RoundedRect::from((rect, Radius(10.0))), &paint);

    // canvas.draw(Circle((100, 100).into(), 100.into()), &paint);
    painting.render();
}
