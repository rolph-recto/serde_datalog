use erased_serde::Deserializer as ErasedDeserializer;
use super::{InputFormat, InputFormatData};

pub struct InputFormatRON;

impl InputFormat for InputFormatRON {
    fn name(&self) -> &'static str {
        "ron"
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["ron"]
    }

    fn create<'input>(&self, contents: &'input str) -> Box<dyn InputFormatData<'input> + 'input> {
        Box::new(InputFormatDataRON { 
            deserializer: ron::Deserializer::from_str(contents).unwrap()
        })
    }
}

pub struct InputFormatDataRON<'input> {
    deserializer: ron::Deserializer<'input>
}

impl<'input> InputFormatData<'input> for InputFormatDataRON<'input> {
    fn deserializer<'de>(&'de mut self) -> Box<dyn ErasedDeserializer<'input> + 'de> {
        Box::new(<dyn ErasedDeserializer<'input>>::erase(&mut self.deserializer))
    }
}