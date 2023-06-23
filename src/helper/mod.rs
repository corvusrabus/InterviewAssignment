pub(crate) fn is_descending_by_key<T, F, K>(values: &[T], f: F) -> bool
where
    F: Fn(&T) -> K,
    K: PartialOrd<K>,
{
    values.windows(2).all(|x| f(&x[0]) >= f(&x[1]))
}

pub(crate) fn is_ascending_by_key<T, F, K>(values: &[T], f: F) -> bool
where
    F: Fn(&T) -> K,
    K: PartialOrd<K>,
{
    values.windows(2).all(|x| f(&x[0]) <= f(&x[1]))
}
