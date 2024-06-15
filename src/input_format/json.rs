use erased_serde::Deserializer as ErasedDeserializer;
use serde_json::de::StrRead;
use super::{InputFormat, InputFormatData};

pub struct InputFormatJSON;

impl InputFormat for InputFormatJSON {
    fn name(&self) -> &'static str {
        "json"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["json"]
    }

    fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
        Box::new(InputFormatJSONData {
            deserializer: serde_json::Deserializer::from_str(contents)
        })
    }
}

struct InputFormatJSONData<'input> {
    deserializer: serde_json::de::Deserializer<StrRead<'input>>
}

impl<'input> InputFormatData<'input> for InputFormatJSONData<'input> {
    fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
        Box::new(<dyn ErasedDeserializer<'input>>::erase(&mut self.deserializer))
    }
}
