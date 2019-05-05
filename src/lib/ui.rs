use emergent_drawing::{Canvas, Drawing, Paint, Painting, Point, Radius, Render, Shape};

#[test]
fn draw_circle() {
    let mut painting: Painting = Painting::new();
    let mut canvas = Canvas::from_target(&mut painting);
    let paint = &Paint::default();
    canvas.draw_circle((100, 100), 50, paint);
    painting.render();
}
