pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod config;
mod conversion;
mod conversion_chain;
mod converter;
mod dict;
mod dict_entry;
mod error;
mod lexicon;
mod segmentation;
mod serialized_values;
mod simple_converter;

use std::ffi::{CString, c_void};
use std::str::FromStr;
use std::{ffi::CStr};
use libc::{c_char, c_int, size_t};

pub use config::Config;
pub use conversion::Conversion;
pub use conversion_chain::ConversionChain;
pub use converter::Converter;
pub use dict::*;
pub use dict::text::TextDict;
pub use dict::group::DictGroup;
pub use dict::marisa::MarisaDict;
pub use dict_entry::*;
pub use error::*;
pub use lexicon::Lexicon;
pub use segmentation::*;
pub use serialized_values::SerializedValues;
pub use simple_converter::SimpleConverter;

#[allow(non_camel_case_types)]
pub type opencc_t = *mut c_void;

#[unsafe(no_mangle)]
pub extern "C" fn opencc_open(config_file_name: *const c_char) -> *mut c_void {
    let name = unsafe { CStr::from_ptr(config_file_name) };
    let path = name.to_str().unwrap();
    let instance = SimpleConverter::build(path).unwrap();
    let boxed = Box::new(instance);
    Box::into_raw(boxed) as opencc_t
}

#[unsafe(no_mangle)]
pub extern "C" fn opencc_close(opencc: opencc_t) -> c_int {
    if !opencc.is_null() {
        let _ = unsafe { Box::from_raw(opencc as *mut SimpleConverter) };
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn opencc_convert_utf8(opencc: opencc_t, input: *const c_char, length: size_t) -> *mut c_char {
    let instance = unsafe { Box::from_raw(opencc as *mut SimpleConverter) };
    let input = unsafe { CStr::from_ptr(input).to_str().unwrap() };
    let converted = instance.convert(&input[0..length]);
    let output = CString::from_str(&converted).unwrap();
    output.into_raw()
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
