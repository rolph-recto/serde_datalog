use erased_serde::Deserializer as ErasedDeserializer;

/// An input format from which data can be extracted into Datalog facts.
/// Implementations of this trait can generate
pub trait InputFormat {
    /// The name of the input format. This name can be passed explicitly as the
    /// format of an input file.
    fn name(&self) -> &'static str;

    /// Returns a list of file extensions associated with the input format.
    /// This will be used to determine the format of an input file,
    /// if its format is not explcitly specified
    fn file_extensions(&self) -> Vec<&'static str>;

    /// Create an [InputFormatData] instance from the contents of an input file.
    fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input>;
}

/// Data that is used to create a [serde::Deserializer] from the contents
/// of an input file.
pub trait InputFormatData<'input> {
    /// This returns an erased version of [serde::Deserializer] from the
    /// [erased_serde](https://crates.io/crates/erased-serde) create, which
    /// allows conversion to a trait object.
    fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de>;
}

#[cfg(feature = "ron")]
pub mod json;

#[cfg(feature = "ron")]
pub mod ron;

#[cfg(feature = "toml")]
pub mod toml;

#[cfg(feature = "yaml")]
pub mod yaml;
