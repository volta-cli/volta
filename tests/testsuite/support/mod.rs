macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err(e) => panic!("{} failed with {}", stringify!($e), e),
        }
    };
}

pub mod matchers;
pub mod paths;
pub mod process;
pub mod sandbox;
