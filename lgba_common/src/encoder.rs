use anyhow::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[derive(Debug)]
pub struct BaseEncoder {
    base: usize, // should actually be u32, but, convenience
    data: Vec<u8>,
    usage: HashMap<String, usize>,
    usage_hint: String,
    pub cached_objects: HashMap<[u8; 32], usize>,
}
impl BaseEncoder {
    pub fn new(base: usize) -> Self {
        BaseEncoder {
            base,
            data: vec![],
            usage: Default::default(),
            usage_hint: "Game Data".to_string(),
            cached_objects: Default::default(),
        }
    }
    pub fn set_usage_hint(&mut self, str: &str) {
        self.usage_hint = str.to_string();
    }
    pub fn iter_usage(&self) -> impl Iterator<Item = (&str, usize)> + '_ {
        self.usage.iter().map(|x| (x.0.as_str(), *x.1))
    }
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn cur_offset(&self) -> usize {
        self.base + self.data.len()
    }
    fn mark_usage(&mut self, start_offset: usize) {
        if self.usage.contains_key(&self.usage_hint) {
            *self.usage.get_mut(&self.usage_hint).unwrap() += self.cur_offset() - start_offset;
        } else {
            self.usage
                .insert(self.usage_hint.clone(), self.cur_offset() - start_offset);
        }
    }

    pub fn align<T>(&mut self) -> usize {
        let align_start = self.cur_offset();
        while self.cur_offset() % std::mem::align_of::<T>() != 0 {
            self.data.push(0);
        }
        self.mark_usage(align_start);
        self.cur_offset()
    }
    pub fn encode<T: Serialize>(&mut self, data: &T) -> Result<usize> {
        let start_off = self.align::<T>();

        let start = self.data.len();
        self.data.resize(start + std::mem::size_of::<T>(), 0);
        let written = ssmarshal::serialize(&mut self.data[start..], data)?;
        assert_eq!(written, std::mem::size_of::<T>());
        self.mark_usage(start_off);

        Ok(start_off)
    }
    pub fn encode_bytes_raw(&mut self, data: &[u8]) {
        let start_off = self.cur_offset();
        let start = self.data.len();
        self.data.resize(start + data.len(), 0);
        self.data[start..].copy_from_slice(data);
        self.mark_usage(start_off);
    }
    pub fn encode_bytes(&mut self, data: &[u8]) -> Result<usize> {
        if let Some(start_off_raw) = self.data.windows(data.len()).position(|x| x == data) {
            Ok(start_off_raw + self.base)
        } else {
            let start_off = self.cur_offset();
            self.encode_bytes_raw(data);
            Ok(start_off)
        }
    }
}
