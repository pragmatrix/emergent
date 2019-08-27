//! nice stuff.

pub trait FirstAndLast<E> {
    fn first_and_last(&self) -> Option<(&E, &E)>;
}

impl<E> FirstAndLast<E> for [E] {
    fn first_and_last(&self) -> Option<(&E, &E)> {
        self.first().map(|f| (f, self.last().unwrap()))
    }
}
