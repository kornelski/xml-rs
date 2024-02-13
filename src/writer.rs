//! Contains high-level interface for an events-based XML emitter.
//!
//! The most important type in this module is `EventWriter` which allows writing an XML document
//! to some output stream.
extern crate alloc;

pub use self::config::EmitterConfig;
pub use self::emitter::EmitterError as Error;
pub use self::emitter::Result;
pub use self::events::XmlEvent;

use self::emitter::Emitter;

mod config;
mod emitter;
pub mod events;

/// A wrapper around a String which emits XML document according to provided
/// events.
pub struct EventWriter {
    sink: alloc::string::String,
    emitter: Emitter,
}

impl EventWriter {
    /// Creates a new `EventWriter` using the default
    /// configuration.
    #[inline]
    pub fn new() -> EventWriter {
        EventWriter::new_with_config(EmitterConfig::new())
    }

    /// Creates a new `EventWriter` using the provided
    /// configuration.
    #[inline]
    pub fn new_with_config(config: EmitterConfig) -> EventWriter {
        EventWriter {
            sink: alloc::string::String::new(),
            emitter: Emitter::new(config),
        }
    }

    /// Writes the next piece of XML document according to the provided event.
    ///
    /// Note that output data may not exactly correspond to the written event because
    /// of various configuration options. For example, `XmlEvent::EndElement` may
    /// correspond to a separate closing element or it may cause writing an empty element.
    /// Another example is that `XmlEvent::CData` may be represented as characters in
    /// the output stream.
    pub fn write<'a, E>(&mut self, event: E) -> Result<()> where E: Into<XmlEvent<'a>> {
        match event.into() {
            XmlEvent::StartDocument { version, encoding, standalone } =>
                self.emitter.emit_start_document(&mut self.sink, version, encoding.unwrap_or("UTF-8"), standalone),
            XmlEvent::ProcessingInstruction { name, data } =>
                self.emitter.emit_processing_instruction(&mut self.sink, name, data),
            XmlEvent::StartElement { name, attributes, namespace } => {
                self.emitter.namespace_stack_mut().push_empty().checked_target().extend(namespace.as_ref());
                self.emitter.emit_start_element(&mut self.sink, name, &attributes)
            }
            XmlEvent::EndElement { name } => {
                let r = self.emitter.emit_end_element(&mut self.sink, name);
                self.emitter.namespace_stack_mut().try_pop();
                r
            }
            XmlEvent::Comment(content) => self.emitter.emit_comment(&mut self.sink, content),
            XmlEvent::CData(content) => Ok(self.emitter.emit_cdata(&mut self.sink, content)),
            XmlEvent::Characters(content) => Ok(self.emitter.emit_characters(&mut self.sink, content)),
        }
    }

    /// Returns a mutable reference to the underlying String.
    pub fn inner_mut(&mut self) -> &mut alloc::string::String {
        &mut self.sink
    }

    /// Unwraps this `EventWriter`, returning the String the writer has written to.
    /// This is the primary method for retrieving the output of the `no-std` writer.
    pub fn into_inner(self) -> alloc::string::String {
        self.sink
    }
}
