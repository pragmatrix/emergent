pub trait ReplaceWith: Sized {
    fn replace_with(&mut self, f: impl FnOnce(Self) -> Self);
}

impl<T> ReplaceWith for T {
    fn replace_with(&mut self, f: impl FnOnce(Self) -> Self) {
        replace_with::replace_with_or_abort(self, f);
    }
}
