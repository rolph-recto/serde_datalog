use erased_serde::Deserializer as ErasedDeserializer;
use super::{InputFormat, InputFormatData};

pub struct InputFormatYAML;

impl InputFormat for InputFormatYAML {
    fn name(&self) -> &'static str {
        "yaml"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["yaml", "yml"]
    }

    fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
        Box::new(InputFormatDataYAML { contents })
    }
}

pub struct InputFormatDataYAML<'input> {
    contents: &'input str
}

impl<'input> InputFormatData<'input> for InputFormatDataYAML<'input> {
    fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
        Box::new(<dyn ErasedDeserializer<'input>>::erase(serde_yaml::Deserializer::from_str(self.contents)))
    }
}