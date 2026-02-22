pub trait VecAddons<T> {
    fn find_index<F>(&self, predicate: F) -> Option<i32>
    where F: Fn(&T) -> bool;
}

impl<T> VecAddons<T> for Vec<T> {
    fn find_index<F>(&self, predicate: F) -> Option<i32>
    where
        F: Fn(&T) -> bool
    {
        self.iter()
            .position(predicate)
            .and_then(|idx| i32::try_from(idx).ok())
    }
}
