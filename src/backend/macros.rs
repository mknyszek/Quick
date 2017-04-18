#[macro_export]
macro_rules! return_error {
    ($($i:expr),*) => {{
        let mut s = String::new();
        write!(s, $($i),*).ok().unwrap();
        return Err(s);
    }}
}

#[macro_export]
macro_rules! unreachable {
    () => { panic!("Broken logic; unreachable point!"); }
}

#[macro_export]
macro_rules! irt_table {
    ($(fn[$s:ident] $i:ident($n:expr) $b:block)*) => {
        $(
        pub fn $i($s: &mut Vec<Value>) {
            $b
        }
        )*
        pub static IRT_STRINGS: &'static [&'static str] = &[
            $(stringify!($i)),*
        ];
        pub static IRT_TABLE: &'static [IRTFunction] = &[
            $(IRTFunction { entry: $i, arity: $n }),* 
        ];
    }
}
