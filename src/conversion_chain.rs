
use crate::Conversion;

pub struct ConversionChain {
    conversions: Vec<Conversion>,
}

impl ConversionChain {
    pub fn new(conversions: Vec<Conversion>) -> Self {
        Self { conversions }
    }

    pub fn conversions(&self) -> &Vec<Conversion> {
        &self.conversions
    }

    pub fn convert(&self, input: &Vec<String>) -> Vec<String> {
        self.conversions
            .iter()
            .fold(input.to_vec(), |output, conversion| {
                conversion.convert_segments(&output)
            })
    }
}
