#[macro_export]
macro_rules! id_type {
    ($vis:vis $name:ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        $vis struct $name(usize);

        ferrodb_util::paste! {
            static [<ID_ $name:snake:upper>]: std::sync::atomic::AtomicUsize =
                std::sync::atomic::AtomicUsize::new(0);
        }

        impl $name {
            fn new() -> $name {
                let id = ferrodb_util::paste!([<ID_ $name:snake:upper>])
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                $name(id)
            }
        }
    }
}
