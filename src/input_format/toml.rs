use super::{InputFormat, InputFormatData};
use erased_serde::Deserializer as ErasedDeserializer;

pub struct InputFormatTOML;

impl InputFormat for InputFormatTOML {
    fn name(&self) -> &'static str {
        "toml"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["toml"]
    }

    fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
        Box::new(InputFormatDataTOML { contents })
    }

    fn has_string_keys(&self) -> bool {
        true
    }
}

pub struct InputFormatDataTOML<'a> {
    contents: &'a str,
}

impl<'input> InputFormatData<'input> for InputFormatDataTOML<'input> {
    fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
        Box::new(<dyn ErasedDeserializer<'input>>::erase(
            toml::Deserializer::new(self.contents),
        ))
    }
}
