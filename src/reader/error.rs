extern crate alloc;

use crate::Encoding;
use crate::reader::lexer::Token;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};

use core::borrow::Borrow;
use core::fmt;
use core::str;

use crate::common::{Position, TextPosition};
use crate::util;

#[derive(Debug)]
pub enum ErrorKind {
    Syntax(Cow<'static, str>),
    Io(String),
    Utf8(str::Utf8Error),
    UnexpectedEof,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub(crate) enum SyntaxError {
    CannotRedefineXmlnsPrefix,
    CannotRedefineXmlPrefix,
    /// Recursive custom entity expanded to too many chars, it could be DoS
    EntityTooBig,
    EmptyEntity,
    NoRootElement,
    ProcessingInstructionWithoutName,
    UnbalancedRootElement,
    UnexpectedEof,
    UnexpectedOpeningTag,
    /// Missing `]]>`
    UnclosedCdata,
    UnexpectedQualifiedName(Token),
    UnexpectedTokenOutsideRoot(Token),
    UnexpectedToken(Token),
    UnexpectedTokenInEntity(Token),
    UnexpectedTokenInClosingTag(Token),
    UnexpectedTokenInOpeningTag(Token),
    InvalidQualifiedName(Box<str>),
    UnboundAttribute(Box<str>),
    UnboundElementPrefix(Box<str>),
    UnexpectedClosingTag(Box<str>),
    UnexpectedName(Box<str>),
    /// Found <?xml-like PI not at the beginning of a document,
    /// which is an error, see section 2.6 of XML 1.1 spec
    UnexpectedProcessingInstruction(Box<str>, Token),
    CannotUndefinePrefix(Box<str>),
    InvalidCharacterEntity(u32),
    InvalidDefaultNamespace(Box<str>),
    InvalidNamePrefix(Box<str>),
    InvalidNumericEntity(Box<str>),
    InvalidStandaloneDeclaration(Box<str>),
    InvalidXmlProcessingInstruction(Box<str>),
    RedefinedAttribute(Box<str>),
    UndefinedEntity(Box<str>),
    UnexpectedEntity(Box<str>),
    UnexpectedNameInsideXml(Box<str>),
    UnsupportedEncoding(Box<str>),
    /// In DTD
    UnknownMarkupDeclaration(Box<str>),
    UnexpectedXmlVersion(Box<str>),
    ConflictingEncoding(Encoding, Encoding),
    UnexpectedTokenBefore(&'static str, char),
    /// Document has more stuff than `ParserConfig` allows
    ExceededConfiguredLimit,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_cow().fmt(f)
    }
}

impl SyntaxError {
    #[inline(never)]
    #[cold]
    pub(crate) fn to_cow(&self) -> Cow<'static, str> {
        match *self {
            Self::CannotRedefineXmlnsPrefix => "Cannot redefine XMLNS prefix".into(),
            Self::CannotRedefineXmlPrefix => "Default XMLNS prefix cannot be rebound to another value".into(),
            Self::EmptyEntity => "Encountered empty entity".into(),
            Self::EntityTooBig => "Entity too big".into(),
            Self::NoRootElement => "Unexpected end of stream: no root element found".into(),
            Self::ProcessingInstructionWithoutName => "Encountered processing instruction without a name".into(),
            Self::UnbalancedRootElement => "Unexpected end of stream: still inside the root element".into(),
            Self::UnclosedCdata => "Unclosed <![CDATA[".into(),
            Self::UnexpectedEof => "Unexpected end of stream".into(),
            Self::UnexpectedOpeningTag => "'<' is not allowed in attributes".into(),
            Self::CannotUndefinePrefix(ref ln) => alloc::format!("Cannot undefine prefix '{ln}'").into(),
            Self::ConflictingEncoding(a, b) => alloc::format!("Declared encoding {a}, but uses {b}").into(),
            Self::InvalidCharacterEntity(num) => alloc::format!("Invalid character U+{num:04X}").into(),
            Self::InvalidDefaultNamespace(ref name) => alloc::format!( "Namespace '{name}' cannot be default").into(),
            Self::InvalidNamePrefix(ref prefix) => alloc::format!("'{prefix}' cannot be an element name prefix").into(),
            Self::InvalidNumericEntity(ref v) => alloc::format!("Invalid numeric entity: {v}").into(),
            Self::InvalidQualifiedName(ref e) => alloc::format!("Qualified name is invalid: {e}").into(),
            Self::InvalidStandaloneDeclaration(ref value) => alloc::format!("Invalid standalone declaration value: {value}").into(),
            Self::InvalidXmlProcessingInstruction(ref name) => alloc::format!("Invalid processing instruction: <?{name} - \"<?xml\"-like PI is only valid at the beginning of the document").into(),
            Self::RedefinedAttribute(ref name) => alloc::format!("Attribute '{name}' is redefined").into(),
            Self::UnboundAttribute(ref name) => alloc::format!("Attribute {name} prefix is unbound").into(),
            Self::UnboundElementPrefix(ref name) => alloc::format!("Element {name} prefix is unbound").into(),
            Self::UndefinedEntity(ref v) => alloc::format!("Undefined entity: {v}").into(),
            Self::UnexpectedClosingTag(ref expected_got) => alloc::format!("Unexpected closing tag: {expected_got}").into(),
            Self::UnexpectedEntity(ref name) => alloc::format!("Unexpected entity: {name}").into(),
            Self::UnexpectedName(ref name) => alloc::format!("Unexpected name: {name}").into(),
            Self::UnexpectedNameInsideXml(ref name) => alloc::format!("Unexpected name inside XML declaration: {name}").into(),
            Self::UnexpectedProcessingInstruction(ref buf, token) => alloc::format!("Unexpected token inside processing instruction: <?{buf}{token}").into(),
            Self::UnexpectedQualifiedName(e) => alloc::format!("Unexpected token inside qualified name: {e}").into(),
            Self::UnexpectedToken(token) => alloc::format!("Unexpected token: {token}").into(),
            Self::UnexpectedTokenBefore(before, c) => alloc::format!("Unexpected token '{before}' before '{c}'").into(),
            Self::UnexpectedTokenInClosingTag(token) => alloc::format!("Unexpected token inside closing tag: {token}").into(),
            Self::UnexpectedTokenInEntity(token) => alloc::format!("Unexpected token inside entity: {token}").into(),
            Self::UnexpectedTokenInOpeningTag(token) => alloc::format!("Unexpected token inside opening tag: {token}").into(),
            Self::UnexpectedTokenOutsideRoot(token) => alloc::format!("Unexpected characters outside the root element: {token}").into(),
            Self::UnexpectedXmlVersion(ref version) => alloc::format!("Invalid XML version: {version}").into(),
            Self::UnknownMarkupDeclaration(ref v) => alloc::format!("Unknown markup declaration: {v}").into(),
            Self::UnsupportedEncoding(ref v) => alloc::format!("Unsupported encoding: {v}").into(),
            Self::ExceededConfiguredLimit => "This document is larger/more complex than allowed by the parser's configuration".into(),
        }
    }
}

/// An XML parsing error.
///
/// Consists of a 2D position in a document and a textual message describing the error.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Error {
    pub(crate) pos: TextPosition,
    pub(crate) kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ErrorKind::{Io, Syntax, UnexpectedEof, Utf8};

        write!(f, "{} ", self.pos)?;
        match &self.kind {
            Io(io_error) => io_error.fmt(f),
            Utf8(reason) => reason.fmt(f),
            Syntax(msg) => f.write_str(msg),
            UnexpectedEof => f.write_str("Unexpected EOF"),
        }
    }
}

impl Position for Error {
    #[inline]
    fn position(&self) -> TextPosition { self.pos }
}

impl Error {
    /// Returns a reference to a message which is contained inside this error.
    #[cold]
    #[doc(hidden)]
    #[allow(deprecated)]
    #[must_use] pub fn msg(&self) -> &str {
        use self::ErrorKind::{Io, Syntax, UnexpectedEof, Utf8};
        match &self.kind {
            Io(io_error) => &io_error,
            Utf8(reason) => "UTF8 Error",
            Syntax(msg) => msg.as_ref(),
            UnexpectedEof => "Unexpected EOF",
        }
    }

    #[must_use]
    #[inline]
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl<'a, P, M> From<(&'a P, M)> for Error where P: Position, M: Into<Cow<'static, str>> {
    #[cold]
    fn from(orig: (&'a P, M)) -> Self {
        Error {
            pos: orig.0.position(),
            kind: ErrorKind::Syntax(orig.1.into()),
        }
    }
}

impl From<util::CharReadError> for Error {
    #[cold]
    fn from(e: util::CharReadError) -> Self {
        use crate::util::CharReadError::{Io, UnexpectedEof, Utf8};
        Error {
            pos: TextPosition::new(),
            kind: match e {
                UnexpectedEof => ErrorKind::UnexpectedEof,
                Utf8(reason) => ErrorKind::Utf8(reason),
                Io(io_error) => ErrorKind::Io(io_error),
            },
        }
    }
}


impl Clone for ErrorKind {
    #[cold]
    fn clone(&self) -> Self {
        use self::ErrorKind::{Io, Syntax, UnexpectedEof, Utf8};
        match self {
            UnexpectedEof => UnexpectedEof,
            Utf8(reason) => Utf8(*reason),
            Io(io_error) => Io(io_error.clone()),
            Syntax(msg) => Syntax(msg.clone()),
        }
    }
}
impl PartialEq for ErrorKind {
    #[allow(deprecated)]
    fn eq(&self, other: &ErrorKind) -> bool {
        use self::ErrorKind::{Io, Syntax, UnexpectedEof, Utf8};
        match (self, other) {
            (UnexpectedEof, UnexpectedEof) => true,
            (Utf8(left), Utf8(right)) => left == right,
            (Io(left), Io(right)) =>
                left == right,
            (Syntax(left), Syntax(right)) =>
                left == right,

            (_, _) => false,
        }
    }
}
impl Eq for ErrorKind {}

#[test]
fn err_size() {
    assert!(std::mem::size_of::<SyntaxError>() <= 24);
}
