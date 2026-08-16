use std::borrow::Cow;

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

    /// Apply every conversion in the chain.
    ///
    /// Segments that no conversion touched stay borrowed from `input`; only
    /// segments that actually changed allocate. Each conversion consumes the
    /// previous stage's segments, so owned segments are moved (not cloned)
    /// between stages.
    pub fn convert<'a>(&self, input: &'a [Cow<'a, str>]) -> Vec<Cow<'a, str>> {
        self.conversions
            .iter()
            .fold(input.to_vec(), |segments, conversion| {
                segments
                    .into_iter()
                    .map(|segment| conversion.convert_segment(segment))
                    .collect()
            })
    }
}
