#[macro_export]
macro_rules! id_type {
    ($vis:vis $name:ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        $vis struct $name(usize);
    }
}
