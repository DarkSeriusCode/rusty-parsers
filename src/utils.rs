pub(crate) fn combinations<C, I, U>(len: usize, items: U) -> Vec<C>
where
    C: Default + Extend<I> + Clone,
    I: Clone,
    U: IntoIterator<Item = I> + Clone,
{
    let mut v = vec![];
    if len == 1 {
        for item in items {
            let mut container = C::default();
            container.extend([item]);
            v.push(container);
        }
    } else {
        for combination in combinations::<C, I, U>(len - 1, items.clone()) {
            for item in items.clone() {
                let mut new_combination = combination.clone();
                new_combination.extend([item]);
                v.push(new_combination);
            }
        }
    }
    v
}
