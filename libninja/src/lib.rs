pub mod command;
pub mod extractor;

pub use command::Generate;
pub use hir::Language;

pub fn default<T: Default>() -> T {
    Default::default()
}
