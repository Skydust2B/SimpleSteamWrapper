pub trait FindIndex<Item> {
    fn find_index<F>(&self, predicate: F) -> Option<i32>
    where
        F: Fn(&Item) -> bool;
}

impl<'a, T: 'a, I> FindIndex<T> for I
where
    I: Iterator<Item = &'a T> + Clone,
{
    fn find_index<F>(&self, predicate: F) -> Option<i32>
    where
        F: Fn(&T) -> bool,
    {
        self.clone()
            .position(predicate)
            .and_then(|idx| i32::try_from(idx).ok())
    }
}
