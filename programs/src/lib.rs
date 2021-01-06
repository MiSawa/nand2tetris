pub mod assembly;
pub mod common;
pub mod ir;
pub mod jack;

#[macro_use]
extern crate enumset;
extern crate once_cell;

#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}
