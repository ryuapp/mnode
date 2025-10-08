pub mod lib;

pub fn load_encoding() -> &'static str {
    include_str!("encoding.js")
}
