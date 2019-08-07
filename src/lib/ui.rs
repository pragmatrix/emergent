#[cfg(test)]
mod tests {
    use emergent_drawing::{circle, Canvas, Color, DrawingCanvas, Paint};

    #[test]
    fn draw_circle() {
        let mut canvas = DrawingCanvas::new();
        let mut paint = &mut Paint::default();
        paint.color = Color::from(0xff0000ff);
        canvas.draw(circle((100, 100), 100), &paint);
        canvas.render();
    }
}
