use crate::PathContainsPoint;
use emergent_drawing::{MeasureText, Path, Point, Text};
use emergent_ui::DPI;
use std::fmt::Debug;

pub(crate) trait Support: Debug {
    fn dpi(&self) -> DPI;
    fn measure_text(&self) -> &dyn MeasureText;
    fn path_contains_point(&self) -> &dyn PathContainsPoint;
}

#[derive(Debug)]
struct SupportImpl<MT, PCP>
where
    MT: Debug,
    PCP: Debug,
{
    dpi: DPI,
    measure_text: MT,
    path_contains_point: PCP,
}

impl dyn Support {
    pub fn new<MT, PCP>(dpi: DPI, measure_text: MT, path_contains_point: PCP) -> impl Support
    where
        MT: MeasureText + Debug,
        PCP: PathContainsPoint + Debug,
    {
        SupportImpl {
            dpi,
            measure_text,
            path_contains_point,
        }
    }
}

impl<MT, PCP> Support for SupportImpl<MT, PCP>
where
    MT: MeasureText + Debug,
    PCP: PathContainsPoint + Debug,
{
    fn dpi(&self) -> DPI {
        self.dpi
    }

    fn measure_text(&self) -> &dyn MeasureText {
        &self.measure_text
    }

    fn path_contains_point(&self) -> &dyn PathContainsPoint {
        &self.path_contains_point
    }
}
