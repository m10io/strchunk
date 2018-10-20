use super::StrChunkMut;
use bytes::{Bytes, BytesMut, IntoBuf};
use std::{
    borrow::Borrow,
    error::Error,
    fmt::{self, Debug, Display},
    io::Cursor,
    iter::FromIterator,
    ops::Deref,
    str,
};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrChunk {
    bytes: Bytes,
}

impl StrChunk {
    #[inline]
    pub fn from_static(s: &'static str) -> StrChunk {
        StrChunk {
            bytes: Bytes::from_static(s.as_bytes()),
        }
    }

    pub fn extract_utf8(
        src: &mut BytesMut,
    ) -> Result<Option<StrChunk>, ExtractUtf8Error> {
        match str::from_utf8(src) {
            Ok(_) => {
                // Valid UTF-8 fills the entire source buffer
                let bytes = src.take().freeze();
                Ok(Some(StrChunk { bytes }))
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                let extracted = if valid_len == 0 {
                    None
                } else {
                    let bytes = src.split_to(valid_len).freeze();
                    Some(StrChunk { bytes })
                };
                match e.error_len() {
                    None => {
                        // Incomplete UTF-8 sequence seen at the end
                        Ok(extracted)
                    }
                    Some(error_len) => {
                        // Invalid UTF-8 encountered
                        Err(ExtractUtf8Error {
                            extracted,
                            error_len,
                        })
                    }
                }
            }
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.bytes) }
    }
}

impl Debug for StrChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for StrChunk {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for StrChunk {
    #[inline]
    fn from(src: String) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl<'a> From<&'a str> for StrChunk {
    #[inline]
    fn from(src: &'a str) -> StrChunk {
        StrChunk { bytes: src.into() }
    }
}

impl From<StrChunk> for Bytes {
    #[inline]
    fn from(src: StrChunk) -> Bytes {
        src.bytes
    }
}

impl From<StrChunk> for String {
    #[inline]
    fn from(src: StrChunk) -> String {
        String::from(src.as_str())
    }
}

impl AsRef<[u8]> for StrChunk {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl AsRef<str> for StrChunk {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for StrChunk {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for StrChunk {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl IntoBuf for StrChunk {
    type Buf = Cursor<Bytes>;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        self.bytes.into_buf()
    }
}

impl<'a> IntoBuf for &'a StrChunk {
    type Buf = Cursor<&'a Bytes>;

    #[inline]
    fn into_buf(self) -> Self::Buf {
        (&self.bytes).into_buf()
    }
}

impl FromIterator<char> for StrChunk {
    fn from_iter<T: IntoIterator<Item = char>>(into_iter: T) -> Self {
        StrChunkMut::from_iter(into_iter).into()
    }
}

#[derive(Clone, Debug)]
pub struct ExtractUtf8Error {
    extracted: Option<StrChunk>,
    error_len: usize,
}

impl ExtractUtf8Error {
    pub fn into_extracted(self) -> Option<StrChunk> {
        self.extracted
    }

    pub fn error_len(&self) -> usize {
        self.error_len
    }
}

impl Display for ExtractUtf8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid UTF-8 sequence in input")
    }
}

impl Error for ExtractUtf8Error {}
