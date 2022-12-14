pub mod login;

mod __private {
    include!(concat!(env!("OUT_DIR"), "/frontend.rs"));
}
pub use self::__private::frontend;
