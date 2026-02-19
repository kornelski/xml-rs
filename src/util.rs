use std::fmt;
use std::io::{self, Read};
use std::str::{self, FromStr};

#[derive(Debug)]
pub(crate) enum CharReadError {
    UnexpectedEof,
    Utf8(str::Utf8Error),
    Io(io::Error),
}

impl From<str::Utf8Error> for CharReadError {
    #[cold]
    fn from(e: str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<io::Error> for CharReadError {
    #[cold]
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl fmt::Display for CharReadError {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::CharReadError::{Io, UnexpectedEof, Utf8};
        match *self {
            UnexpectedEof => write!(f, "unexpected end of stream"),
            Utf8(ref e) => write!(f, "UTF-8 decoding error: {e}"),
            Io(ref e) => write!(f, "I/O error: {e}"),
        }
    }
}

/// Character encoding used for parsing
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Encoding {
    /// Explicitly UTF-8 only
    Utf8,
    /// UTF-8 fallback, but can be any 8-bit encoding
    Default,
    /// ISO-8859-1
    Latin1,
    /// US-ASCII
    Ascii,
    /// Big-Endian
    Utf16Be,
    /// Little-Endian
    Utf16Le,
    /// Unknown endianness yet, will be sniffed
    Utf16,
    /// Not determined yet, may be sniffed to be anything
    Unknown,
}

// Rustc inlines eq_ignore_ascii_case and creates kilobytes of code!
#[inline(never)]
fn icmp(lower: &str, varcase: &str) -> bool {
    lower.bytes().zip(varcase.bytes()).all(|(l, v)| l == v.to_ascii_lowercase())
}

impl FromStr for Encoding {
    type Err = &'static str;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        if ["utf-8", "utf8"].into_iter().any(move |label| icmp(label, val)) {
            Ok(Self::Utf8)
        } else if ["iso-8859-1", "latin1"].into_iter().any(move |label| icmp(label, val)) {
            Ok(Self::Latin1)
        } else if ["utf-16", "utf16"].into_iter().any(move |label| icmp(label, val)) {
            Ok(Self::Utf16)
        } else if ["ascii", "us-ascii"].into_iter().any(move |label| icmp(label, val)) {
            Ok(Self::Ascii)
        } else {
            Err("unknown encoding name")
        }
    }
}

impl fmt::Display for Encoding {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Utf8 |
            Self::Default => "UTF-8",
            Self::Latin1 => "ISO-8859-1",
            Self::Ascii => "US-ASCII",
            Self::Utf16Be |
            Self::Utf16Le |
            Self::Utf16 => "UTF-16",
            Self::Unknown => "(unknown)",
        })
    }
}

#[derive(Clone)]
pub(crate) struct CharReader {
    pub encoding: Encoding,
    pub buf: Buf,
    pub is_last: bool,
}

#[derive(Clone)]
pub(crate) struct Buf {
    data: Vec<u8>,
    pos: usize,
}

#[allow(unused)]
impl Buf {
    pub fn is_empty(&self) -> bool {
        self.data.len() == self.pos
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn get(&self) -> &[u8] {
        let res = self.data.get(self.pos..);
        debug_assert!(res.is_some());
        res.unwrap_or_default()
    }

    pub fn spare_capacity(&self) -> usize {
        self.data.capacity() - self.data.len()
    }

    pub fn reserve(&mut self) {
        if self.pos > self.data.len()/2 {
            let remaining = self.data.len() - self.pos;
            self.data.copy_within(self.pos.., 0);
            self.pos = 0;
            self.data.truncate(remaining);
        }
    }
}

impl CharReader {
    pub fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            buf: Buf {
                data: Vec::with_capacity(128),
                pos: 0,
            },
            is_last: false,
        }
    }

    pub fn next_char_from<R: Read>(&mut self, reader: &mut R) -> Result<Option<char>, CharReadError> {
        loop {
            match self.consuming_next() {
                Some(Ok(ch)) => {
                    return Ok(Some(ch))
                },
                Some(Err(e)) => return Err(e),
                None if self.is_last => {
                    if self.buf.is_empty() {
                        return Ok(None);
                    }
                    return Err(CharReadError::UnexpectedEof)
                },
                None => {
                    self.buf.reserve();
                    let spare = self.buf.spare_capacity();
                    let read = reader.take(spare as u64).read_to_end(&mut self.buf.data)?;
                    if read == 0 {
                        if self.buf.is_empty() {
                            return Ok(None);
                        }
                        self.is_last = true;
                    }
                    if let Encoding::Unknown | Encoding::Utf16 = self.encoding {
                        self.sniff_bom()?;
                    }
                },
            }
        }
    }

    // None means "needs more input"
    #[inline]
    pub fn consuming_next(&mut self) -> Option<Result<char, CharReadError>> {
        let bytes = self.buf.data.get(self.buf.pos..)?;
        let bytes = &bytes[..bytes.len().min(6)];
        let next = bytes.get(0).copied()?;

        match self.encoding {
            Encoding::Utf8 | Encoding::Default => {
                // fast path for ASCII subset
                if next.is_ascii() {
                    self.buf.pos += 1;
                    return Some(Ok(next.into()));
                }

                for char_len in 1..5 {
                    match str::from_utf8(bytes.get(..char_len)?) {
                        Ok(s) => {
                            self.buf.pos += s.len();
                            return s.chars().next().map(Ok);
                        },
                        Err(e) if char_len == 4 => {
                            return Some(Err(CharReadError::Utf8(e)));
                        },
                        Err(_) => {},
                    }
                }
                return None;
            },
            Encoding::Latin1 => {
                self.buf.pos += 1;
                return Some(Ok(next.into()));
            },
            Encoding::Ascii => {
                return if next.is_ascii() {
                    self.buf.pos += 1;
                    Some(Ok(next.into()))
                } else {
                    return Some(Err(CharReadError::Io(io::ErrorKind::InvalidData.into())));
                };
            },
            Encoding::Utf16Be => {
                if !self.is_last && bytes.len() < 4 {
                    return None;
                }
                let mut consumed = 0;
                let mut chars = char::decode_utf16(bytes.chunks(2).map_while(|ch: &[u8]| {
                    consumed += ch.len();
                    Some(u16::from_be_bytes(ch.try_into().ok()?))
                }));
                let ch = chars.next()?;
                let ch = ch.ok()?;
                self.buf.pos += consumed;
                return Some(Ok(ch));
            },
            Encoding::Utf16Le => {
                if !self.is_last && bytes.len() < 4 {
                    return None;
                }
                let mut consumed = 0;
                let mut chars = char::decode_utf16(bytes.chunks(2).map_while(|ch: &[u8]| {
                    consumed += ch.len();
                    Some(u16::from_le_bytes(ch.try_into().ok()?))
                }));
                let ch = chars.next()?;
                let ch = ch.ok()?;
                self.buf.pos += consumed;
                return Some(Ok(ch));
            },
            Encoding::Unknown | Encoding::Utf16 => {
                return None
            },
        }
    }

    #[cold]
    fn sniff_bom(&mut self) -> Result<(), CharReadError> {
        let buf = self.buf.get();

        if buf.len() < 3 && !self.is_last {
            // it will be called again until encoding is changed to a known one
            return Ok(());
        }

        if buf.len() > 1 {
            // sniff BOM
            if self.encoding != Encoding::Utf16 && buf.starts_with(&[0xEF, 0xBB, 0xBF]) {
                self.encoding = Encoding::Utf8;
                self.buf.pos += 3;
                return Ok(());
            }
            if buf.starts_with(&[0xFE, 0xFF]) {
                self.encoding = Encoding::Utf16Be;
                self.buf.pos += 2;
                return Ok(());
            }
            if buf.starts_with(&[0xFF, 0xFE]) {
                self.encoding = Encoding::Utf16Le;
                self.buf.pos += 2;
                return Ok(());
            }

            // sniff ASCII char in UTF-16
            if self.encoding == Encoding::Utf16 {
                if buf[0] == 0 && buf[1] != 0 {
                    self.encoding = Encoding::Utf16Be;
                    return Ok(());
                }
                if buf[0] != 0 && buf[1] == 0 {
                    self.encoding = Encoding::Utf16Le;
                    return Ok(());
                }
            }
        }
        if self.encoding != Encoding::Utf16 {
            // UTF-8 is the default, but XML decl can change it to other 8-bit encoding
            self.encoding = Encoding::Default;
            return Ok(());
        }
        Err(CharReadError::Io(io::ErrorKind::InvalidData.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::{CharReadError, CharReader, Encoding};

    #[test]
    fn test_next_char_from() {
        use std::io;

        let mut bytes: &[u8] = b"correct";    // correct ASCII
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('c'));

        let mut bytes: &[u8] = b"\xEF\xBB\xBF\xE2\x80\xA2!";  // BOM
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('•'));

        let mut bytes: &[u8] = b"\xEF\xBB\xBFx123";  // BOM
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('x'));

        let mut bytes: &[u8] = b"\xEF\xBB\xBF";  // Nothing after BOM
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), None);

        let mut bytes: &[u8] = b"\xEF\xBB";  // Nothing after BO
        assert!(matches!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes), Err(CharReadError::UnexpectedEof)));

        let mut bytes: &[u8] = b"\xEF\xBB\x42";  // Nothing after BO
        assert!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).is_err());

        let mut bytes: &[u8] = b"\xFE\xFF\x00\x42";  // UTF-16
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('B'));

        let mut bytes: &[u8] = b"\xFF\xFE\x42\x00";  // UTF-16
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('B'));

        let mut bytes: &[u8] = b"\xFF\xFE";  // UTF-16
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), None);

        let mut bytes: &[u8] = b"\xFF\xFE\x00";  // UTF-16
        assert!(matches!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes), Err(CharReadError::UnexpectedEof)));

        let mut bytes: &[u8] = "правильно".as_bytes();  // correct BMP
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('п'));

        let mut bytes: &[u8] = "правильно".as_bytes();
        assert_eq!(CharReader::new(Encoding::Utf16Be).next_char_from(&mut bytes).unwrap(), Some('킿'));

        let mut bytes: &[u8] = "правильно".as_bytes();
        assert_eq!(CharReader::new(Encoding::Utf16Le).next_char_from(&mut bytes).unwrap(), Some('뿐'));

        let mut bytes: &[u8] = b"\xD8\xD8\x80";
        assert!(CharReader::new(Encoding::Utf16).next_char_from(&mut bytes).is_err());

        let mut bytes: &[u8] = b"\x00\x42";
        assert_eq!(CharReader::new(Encoding::Utf16).next_char_from(&mut bytes).unwrap(), Some('B'));

        let mut bytes: &[u8] = b"\x42\x00";
        assert_eq!(CharReader::new(Encoding::Utf16).next_char_from(&mut bytes).unwrap(), Some('B'));

        let mut bytes: &[u8] = &[0xEF, 0xBB, 0xBF, 0xFF, 0xFF];
        assert!(CharReader::new(Encoding::Utf16).next_char_from(&mut bytes).is_err());

        let mut bytes: &[u8] = b"\x00";
        assert!(CharReader::new(Encoding::Utf16Be).next_char_from(&mut bytes).is_err());

        let mut bytes: &[u8] = "😊".as_bytes();          // correct non-BMP
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), Some('😊'));

        let mut bytes: &[u8] = b"";                     // empty
        assert_eq!(CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap(), None);

        let mut bytes: &[u8] = b"\xf0\x9f\x98";         // incomplete code point
        match CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap_err() {
            super::CharReadError::UnexpectedEof => {},
            e => panic!("Unexpected result: {e:?}")
        }

        let mut bytes: &[u8] = b"\xff\x9f\x98\x32";     // invalid code point
        match CharReader::new(Encoding::Unknown).next_char_from(&mut bytes).unwrap_err() {
            super::CharReadError::Utf8(_) => {},
            e => panic!("Unexpected result: {e:?}"),
        }

        // error during read
        struct ErrorReader;
        impl io::Read for ErrorReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "test error"))
            }
        }

        let mut r = ErrorReader;
        match CharReader::new(Encoding::Unknown).next_char_from(&mut r).unwrap_err() {
            super::CharReadError::Io(ref e) if e.kind() == io::ErrorKind::Other &&
                                               e.to_string().contains("test error") => {},
            e => panic!("Unexpected result: {e:?}")
        }
    }
}
