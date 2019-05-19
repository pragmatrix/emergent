use crate::{
    Arc, Circle, ForceMoveTo, LineSegments, Oval, PathVerb, Point, Polygon, Radius, Rect,
    RoundedRect, Size,
};

pub struct Outline(pub Vec<PathVerb>);

impl Outline {
    /// Returns a fast / conservative rectangular bounds of that outline.
    /// Returns None if there is no path, a rect with zero dimensions at
    /// the points location if the path describes only one point.
    pub fn fast_bounds_rect(&self) -> Option<Rect> {
        let mut current: Option<Point> = None;

        // TODO: we don't need to store the points here and could do boundary
        // computation while we iterate through the path ops.
        let mut points = Vec::new();

        for v in self.0.iter() {
            match v {
                PathVerb::MoveTo(p) => current = Some(*p),
                PathVerb::LineTo(p) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p]);
                    current = Some(p1);
                }
                PathVerb::QuadTo(p2, p3) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3]);
                    current = Some(*p3);
                }
                PathVerb::ConicTo(p2, p3, _) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3]);
                    current = Some(*p3);
                }
                PathVerb::CubicTo(p2, p3, p4) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3, *p4]);
                    current = Some(*p4);
                }
                PathVerb::ArcTo(Arc(Oval(r), _, _, _), ForceMoveTo(fmt)) => {
                    // TODO: clarify exactly what ForceMoveTo means.
                    if *fmt {
                        let current = current.unwrap_or_default();
                        points.push(current);
                    };

                    points.extend(&r.points());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    current = Some(r.center())
                }
                PathVerb::Close => {}
                PathVerb::AddRect(r, _) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    current = Some(r.center())
                }
                PathVerb::AddOval(Oval(r), _) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    current = Some(r.center())
                }
                PathVerb::AddCircle(Circle(p, Radius(r)), _) => {
                    let sector_size = Size::from((*r, *r));
                    let r = Rect::from((*p - sector_size, sector_size * 2.0));
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    current = Some(r.center())
                }
                PathVerb::AddArc(Arc(Oval(r), ..)) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    current = Some(r.center())
                }
                PathVerb::AddRoundedRect(RoundedRect(r, _), _) => {
                    points.extend(&r.points());
                    current = Some(r.center())
                }
                PathVerb::AddLineSegments(LineSegments(pts)) => {
                    points.extend(pts);
                    current = pts.last().cloned().or(current);
                }
                PathVerb::AddPolygon(Polygon(pts)) => {
                    points.extend(pts);
                    current = pts.last().cloned().or(current);
                }
            }
        }

        Rect::from_points_as_bounds(&points)
    }
}
