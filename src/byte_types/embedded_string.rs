use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
};
use zerocopy::{AsBytes, FromBytes, Unaligned};

pub trait EmbeddedStringCommon {
    fn len(&self) -> usize;
    fn raw_data(&self) -> &[u8];

    fn data(&self) -> &[u8] {
        &self.raw_data()[..self.len()]
    }

    fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(self.data()).ok()
    }
    fn as_str_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.data())
    }
}
pub trait EmbeddedStringMake: Sized {
    fn from_slice(slice: &[u8]) -> Option<Self>;
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum EmbeddedStringData {
    String(String),
    Bytes(Vec<u8>),
}
impl EmbeddedStringData {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            EmbeddedStringData::Bytes(b) => b.as_slice(),
            EmbeddedStringData::String(s) => s.as_bytes(),
        }
    }
}

impl<T> From<T> for EmbeddedStringData
where
    T: EmbeddedStringCommon,
{
    fn from(s: T) -> Self {
        if let Some(data) = s.as_str() {
            EmbeddedStringData::String(data.to_string())
        } else {
            EmbeddedStringData::Bytes(s.data().to_vec())
        }
    }
}
impl<const SIZE: usize> TryFrom<EmbeddedStringData> for EmbeddedString<SIZE> {
    type Error = String;

    fn try_from(value: EmbeddedStringData) -> Result<Self, Self::Error> {
        Self::from_slice(value.as_bytes()).ok_or_else(String::default)
    }
}
impl<const SIZE: usize> TryFrom<EmbeddedStringData> for DynamicEmbeddedString<SIZE> {
    type Error = String;

    fn try_from(value: EmbeddedStringData) -> Result<Self, Self::Error> {
        Self::from_slice(value.as_bytes()).ok_or_else(String::default)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(into = "EmbeddedStringData")]
#[serde(try_from = "EmbeddedStringData")]
pub struct EmbeddedString<const SIZE: usize>(pub [u8; SIZE]);

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(into = "EmbeddedStringData")]
#[serde(try_from = "EmbeddedStringData")]
pub struct DynamicEmbeddedString<const CAPACITY: usize> {
    pub buffer: [u8; CAPACITY],
    pub length: usize,
}

impl<const CAPACITY: usize> EmbeddedStringMake for EmbeddedString<CAPACITY> {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        let length = slice.len();
        if length > CAPACITY {
            None
        } else {
            let mut result = Self([0; CAPACITY]);
            result.0[..length].copy_from_slice(slice);
            Some(result)
        }
    }
}
impl<const CAPACITY: usize> EmbeddedStringMake for DynamicEmbeddedString<CAPACITY> {
    fn from_slice(slice: &[u8]) -> Option<Self> {
        let length = slice.len();
        if length > CAPACITY {
            None
        } else {
            let mut result = Self {
                buffer: [0; CAPACITY],
                length,
            };
            result.buffer[..length].copy_from_slice(slice);
            Some(result)
        }
    }
}

unsafe impl<const SIZE: usize> Unaligned for EmbeddedString<SIZE> {
    fn only_derive_is_allowed_to_implement_this_trait()
    where
        Self: Sized,
    {
    }
}
unsafe impl<const SIZE: usize> AsBytes for EmbeddedString<SIZE> {
    fn only_derive_is_allowed_to_implement_this_trait()
    where
        Self: Sized,
    {
    }
}
unsafe impl<const SIZE: usize> FromBytes for EmbeddedString<SIZE> {
    fn only_derive_is_allowed_to_implement_this_trait()
    where
        Self: Sized,
    {
    }
}

impl<const SIZE: usize> EmbeddedStringCommon for EmbeddedString<SIZE> {
    fn len(&self) -> usize {
        self.0
            .iter()
            .copied()
            .enumerate()
            .find(|(_, v)| *v == 0)
            .map(|(pos, _)| pos)
            .unwrap_or(SIZE)
    }

    fn raw_data(&self) -> &[u8] {
        &self.0
    }
}
impl<const CAPACITY: usize> EmbeddedStringCommon for DynamicEmbeddedString<CAPACITY> {
    fn len(&self) -> usize {
        self.length
    }

    fn raw_data(&self) -> &[u8] {
        &self.buffer
    }
}

impl<const SIZE: usize> Display for EmbeddedString<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.as_str() {
            write!(f, "{}", s)
        } else {
            write!(f, "{}", hex::encode(self.data()))
        }
    }
}
impl<const SIZE: usize> Debug for EmbeddedString<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}


impl<const CAPACITY: usize> Display for DynamicEmbeddedString<CAPACITY> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.as_str() {
            write!(f, "{}", s)
        } else {
            write!(f, "{}", hex::encode(self.data()))
        }
    }
}
impl<const CAPACITY: usize> Debug for DynamicEmbeddedString<CAPACITY> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}
