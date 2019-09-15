use crate::FrameLayout;

pub trait Window: Send + Sync + 'static {
    fn frame_layout(&self) -> FrameLayout;
}
