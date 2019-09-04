use std::mem;

pub trait ReplaceWith: Sized {
    fn replace_with(&mut self, f: impl FnOnce(Self) -> Self);
}

impl<T: Default> ReplaceWith for T {
    fn replace_with(&mut self, f: impl FnOnce(Self) -> Self) {
        let new_s = f(mem::replace(self, Self::default()));
        mem::replace(self, new_s);
    }
}
