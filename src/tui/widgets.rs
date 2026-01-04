struct ScrollableList<T>
where
    T: std::fmt::Display,
{
    list: Vec<T>,
}
