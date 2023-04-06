#[macro_export]
macro_rules! map {
    ($( $key:literal => $value:expr ),* $(,)?) => {{
        let mut m = ::std::collections::HashMap::new();
        $(
        m.insert($key.into(), $value.into());
        )*
        m
    }};
}
