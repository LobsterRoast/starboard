// There are some times when it's nice to be given some extra debug information at runtime, but it's
// will throttle performance to check if a certain `debug` flag is enabled every time you want to
// print debug information. This macro will check at compile time, and only at a `println!` call if
// the `debug` feature is enabled.
#[macro_export]
macro_rules! printdbg {
    ($base:expr, $($args:tt), *) => {
        #[cfg(feature = "debug")]
        println!($base, $($args), *);
    };
    ($base:expr, $($args:expr), *) => {
        #[cfg(feature = "debug")]
        println!($base, $($args), *);
    };
    ($base:expr) => {
        #[cfg(feature = "debug")]
        println!($base);
    }
}
