use std::{
    cell::RefCell,
    ffi::CString,
    fs::File,
    io::{Read, Write},
    ptr,
    rc::Rc,
};

use crate::{Dict, DictEntryFactory, Error, Lexicon, SerializableDict};

pub struct SerializedValues {
    lexicon: Rc<RefCell<Lexicon>>,
}

fn read_integer<I: Default + Copy + Sized, R: Read>(reader: &mut R) -> Result<I, Error> {
    let mut num = I::default();
    let num_bytes = size_of::<I>();
    let buffer =
        unsafe { std::slice::from_raw_parts_mut(&mut num as *mut I as *mut u8, num_bytes) };
    if reader.read_exact(buffer).is_err() {
        return Err(Error::InvalidFormat(
            "Invalid OpenCC binary dictionary.".to_string(),
        ));
    }
    Ok(num)
}

fn write_integer<I, W: Write>(writer: &mut W, num: I) -> Result<(), Error>
where
    I: Sized,
{
    let num_bytes = size_of::<I>();
    let buffer = unsafe { std::slice::from_raw_parts(&num as *const I as *const u8, num_bytes) };
    let units_write = writer.write(buffer)?;
    if units_write != num_bytes {
        return Err(Error::InvalidFormat(
            "Cannot write binary dictionary.".to_string(),
        ));
    }
    Ok(())
}

impl SerializedValues {
    pub fn from_lexicon(lexicon: Rc<RefCell<Lexicon>>) -> Self {
        Self { lexicon }
    }

    fn construct_buffer(
        &self,
        value_buffer: &mut String,
        value_bytes: &mut Vec<u16>,
        value_total_length: &mut u32,
    ) {
        *value_total_length = 0;
        let lexicon = self.lexicon.borrow();
        for entry in lexicon.iter() {
            assert!(!entry.values().is_empty());
            for value in entry.values() {
                *value_total_length += value.len() as u32 + 1;
            }
        }

        value_buffer.clear();
        unsafe {
            value_buffer
                .as_mut_vec()
                .resize(*value_total_length as usize, 0);
        }
        let mut p_value_buffer = unsafe { value_buffer.as_mut_vec().as_mut_ptr() };

        for entry in lexicon.iter() {
            for value in entry.values() {
                let c_string = CString::new(value.as_str()).unwrap();
                let bytes = c_string.as_bytes_with_nul();
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr(), p_value_buffer, bytes.len());
                    p_value_buffer = p_value_buffer.add(bytes.len());
                }
                value_bytes.push(bytes.len() as u16);
            }
        }
        unsafe {
            assert_eq!(
                value_buffer.as_ptr().add(*value_total_length as usize),
                p_value_buffer
            );
        }
    }
}

impl Dict for SerializedValues {
    fn key_max_length(&self) -> usize {
        0
    }

    fn lexicon(&self) -> Rc<RefCell<Lexicon>> {
        self.lexicon.clone()
    }
}

impl SerializableDict for SerializedValues {
    fn new_from_file(file: &mut File) -> Result<Rc<Self>, crate::Error> {
        let mut lexicon = Lexicon::new();
        let num_items: u32 = read_integer(file)?;
        let value_total_length: u32 = read_integer(file)?;
        let mut value_buffer = vec![0u8; value_total_length as usize];
        if file.read_exact(&mut value_buffer).is_err() {
            return Err(Error::InvalidFormat(
                "Invalid OpenCC binary dictionary (valueBuffer)".to_string(),
            ));
        }

        let mut offset = 0;
        for _ in 0..num_items {
            let num_values: u16 = read_integer(file)?;
            let mut values = Vec::with_capacity(num_values as usize);
            for _ in 0..num_values {
                let num_value_bytes: u16 = read_integer(file)?;
                let chunk = &value_buffer[offset..offset + num_value_bytes as usize];
                // Trim tailing zero
                let end_idx = chunk.iter().position(|&b| b == 0).unwrap_or(chunk.len());
                offset += num_value_bytes as usize;
                let value = String::from_utf8(chunk[..end_idx].to_vec()).unwrap();
                values.push(value);
            }
            let entry = DictEntryFactory::new_with_key_and_values("", values);
            lexicon.add(entry);
        }
        Ok(Rc::new(Self::from_lexicon(Rc::new(RefCell::new(lexicon)))))
    }

    fn serialize_to_file(&self, file: &mut File) -> Result<(), Error> {
        let guard = self.lexicon();
        let lexicon = guard.borrow();
        let mut value_total_length: u32 = 0;
        let mut value_buf = String::new();
        let mut value_bytes: Vec<u16> = Vec::new();
        self.construct_buffer(&mut value_buf, &mut value_bytes, &mut value_total_length);
        // Number of items
        let num_items = lexicon.len() as u32;
        write_integer(file, num_items)?;

        // Data
        write_integer(file, value_total_length)?;
        file.write(value_buf.as_bytes())?;

        let mut value_cursor = 0;
        for entry in lexicon.iter() {
            // Number of values
            let num_values = entry.values().len();
            write_integer(file, num_values as u16)?;
            // Values offet
            for _ in 0..num_values {
                let num_value_bytes = value_bytes[value_cursor];
                value_cursor += 1;
                write_integer(file, num_value_bytes)?;
            }
        }
        Ok(())
    }
}
