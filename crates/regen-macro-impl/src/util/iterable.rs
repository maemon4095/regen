pub trait Iterable {
    type Item: ?Sized;
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_>;
}
impl<T: ?Sized, V> Iterable for T
where
    for<'a> &'a T: IntoIterator<Item = &'a V>,
{
    type Item = V;

    type Iter<'a>
        = <&'a T as IntoIterator>::IntoIter
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        self.into_iter()
    }
}
