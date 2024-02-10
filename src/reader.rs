//! Contains high-level interface for a pull-based XML parser.
//!
//! The most important type in this module is `EventReader`, which provides an iterator
//! view for events in XML document.

use core::iter::FusedIterator;
use core::result;

use crate::common::{Position, TextPosition};

pub use self::config::ParserConfig;
pub use self::config::ParserConfig2;
pub use self::error::{Error, ErrorKind};
pub use self::events::XmlEvent;

use self::parser::PullParser;

mod config;
mod events;
mod lexer;
mod parser;
mod error;


/// A result type yielded by `XmlReader`.
pub type Result<T, E = Error> = result::Result<T, E>;

/// A wrapper around a Source instance which provides pull-based XML parsing.
pub struct EventReader<'a, S: Iterator<Item = &'a u8>> {
    source: S,
    parser: PullParser,
}

impl<'a, S: Iterator<Item = &'a u8>> EventReader<'a, S> {
    /// Creates a new reader, consuming the given stream.
    #[inline]
    pub fn new(source: S) -> EventReader<'a, S> {
        EventReader::new_with_config(source, ParserConfig2::new())
    }

    /// Creates a new reader with the provded configuration, consuming the given stream.
    #[inline]
    pub fn new_with_config(source: S, config: impl Into<ParserConfig2>) -> EventReader<'a, S> {
        EventReader { source, parser: PullParser::new(config) }
    }

    /// Pulls and returns next XML event from the stream.
    ///
    /// If returned event is `XmlEvent::Error` or `XmlEvent::EndDocument`, then
    /// further calls to this method will return this event again.
    #[inline]
    pub fn next(&mut self) -> Result<XmlEvent> {
        self.parser.next(&mut self.source)
    }

    /// Skips all XML events until the next end tag at the current level.
    ///
    /// Convenience function that is useful for the case where you have
    /// encountered a start tag that is of no interest and want to
    /// skip the entire XML subtree until the corresponding end tag.
    #[inline]
    pub fn skip(&mut self) -> Result<()> {
        let mut depth = 1;

        while depth > 0 {
            match self.next()? {
                XmlEvent::StartElement { .. } => depth += 1,
                XmlEvent::EndElement { .. } => depth -= 1,
                XmlEvent::EndDocument => unreachable!(),
                _ => {}
            }
        }

        Ok(())
    }

    pub fn source(&self) -> &S { &self.source }
    pub fn source_mut(&mut self) -> &mut S { &mut self.source }

    /// Unwraps this `EventReader`, returning the underlying reader.
    ///
    /// Note that this operation is destructive; unwrapping the reader and wrapping it
    /// again with `EventReader::new()` will create a fresh reader which will attempt
    /// to parse an XML document from the beginning.
    pub fn into_inner(self) -> S {
        self.source
    }
}

impl<'a, S: Iterator<Item = &'a u8>> Position for EventReader<'a, S> {
    /// Returns the position of the last event produced by the reader.
    #[inline]
    fn position(&self) -> TextPosition {
        self.parser.position()
    }
}

impl<'a, S: Iterator<Item = &'a u8>> IntoIterator for EventReader<'a, S> {
    type Item = Result<XmlEvent>;
    type IntoIter = Events<'a, S>;

    fn into_iter(self) -> Events<'a, S> {
        Events { reader: self, finished: false }
    }
}

/// An iterator over XML events created from some type implementing `Read`.
///
/// When the next event is `xml::event::Error` or `xml::event::EndDocument`, then
/// it will be returned by the iterator once, and then it will stop producing events.
pub struct Events<'a, S: Iterator<Item = &'a u8>> {
    reader: EventReader<'a, S>,
    finished: bool,
}

impl<'a, S: Iterator<Item = &'a u8>> Events<'a, S> {
    /// Unwraps the iterator, returning the internal `EventReader`.
    #[inline]
    pub fn into_inner(self) -> EventReader<'a, S> {
        self.reader
    }

    pub fn source(&self) -> &S { &self.reader.source }
    pub fn source_mut(&mut self) -> &mut S { &mut self.reader.source }

}

impl<'a, S: Iterator<Item = &'a u8>> FusedIterator for Events<'a, S> {
}

impl<'a, S: Iterator<Item = &'a u8>> Iterator for Events<'a, S> {
    type Item = Result<XmlEvent>;

    #[inline]
    fn next(&mut self) -> Option<Result<XmlEvent>> {
        if self.finished && !self.reader.parser.is_ignoring_end_of_stream() {
            None
        } else {
            let ev = self.reader.next();
            if let Ok(XmlEvent::EndDocument) | Err(_) = ev {
                self.finished = true;
            }
            Some(ev)
        }
    }
}

impl<'a> EventReader<'a, core::slice::Iter<'a, u8>> {
    /// A convenience method to create an `XmlReader` from a string slice.
    #[inline]
    #[must_use]
    pub fn from_str(source: &'a str) -> EventReader<core::slice::Iter<'a, u8>> {
        EventReader::new(source.as_bytes().into_iter())
    }
}