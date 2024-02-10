extern crate alloc;

use core::fmt;
use core::fmt::Write;
use core::result;

use alloc::string::String;
use alloc::vec::Vec;

use crate::attribute::Attribute;
use crate::common;
use crate::common::XmlVersion;
use crate::escape::{AttributeEscapes, Escaped, PcDataEscapes};
use crate::name::{Name, OwnedName};
use crate::namespace::{NamespaceStack, NS_EMPTY_URI, NS_NO_PREFIX, NS_XMLNS_PREFIX, NS_XML_PREFIX};

use crate::writer::config::EmitterConfig;

macro_rules! write {
    ($dst:expr, $($arg:tt)*) => {
        $dst.push_str(&alloc::format!($($arg)*))
    };
}

/// An error which may be returned by `XmlWriter` when writing XML events.
#[derive(Debug)]
pub enum EmitterError {
    /// An I/O error occured in the underlying `Write` instance.
    Io(String),

    /// Document declaration has already been written to the output stream.
    DocumentStartAlreadyEmitted,

    /// The name of the last opening element is not available.
    LastElementNameNotAvailable,

    /// The name of the last opening element is not equal to the name of the provided
    /// closing element.
    EndElementNameIsNotEqualToLastStartElementName,

    /// End element name is not specified when it is needed, for example, when automatic
    /// closing is not enabled in configuration.
    EndElementNameIsNotSpecified,
}

impl fmt::Display for EmitterError {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("emitter error: ")?;
        match self {
            EmitterError::Io(e) => f.write_str(&alloc::format!("I/O error: {e}")),
            EmitterError::DocumentStartAlreadyEmitted => f.write_str("document start event has already been emitted"),
            EmitterError::LastElementNameNotAvailable => f.write_str("last element name is not available"),
            EmitterError::EndElementNameIsNotEqualToLastStartElementName => f.write_str("end element name is not equal to last start element name"),
            EmitterError::EndElementNameIsNotSpecified => f.write_str("end element name is not specified and can't be inferred"),
        }
    }
}

/// A result type yielded by `XmlWriter`.
pub type Result<T, E = EmitterError> = result::Result<T, E>;

// TODO: split into a low-level fast writer without any checks and formatting logic and a
// high-level indenting validating writer
pub struct Emitter {
    config: EmitterConfig,

    nst: NamespaceStack,

    indent_level: usize,
    indent_stack: Vec<IndentFlags>,

    element_names: Vec<OwnedName>,

    start_document_emitted: bool,
    just_wrote_start_element: bool,
}

impl Emitter {
    pub fn new(config: EmitterConfig) -> Emitter {
        let mut indent_stack = Vec::with_capacity(16);
        indent_stack.push(IndentFlags::WroteNothing);

        Emitter {
            config,

            nst: NamespaceStack::empty(),

            indent_level: 0,
            indent_stack,

            element_names: Vec::new(),

            start_document_emitted: false,
            just_wrote_start_element: false,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum IndentFlags {
    WroteNothing,
    WroteMarkup,
    WroteText,
}

impl Emitter {
    /// Returns the current state of namespaces.
    #[inline]
    pub fn namespace_stack_mut(&mut self) -> &mut NamespaceStack {
        &mut self.nst
    }

    #[inline]
    fn wrote_text(&self) -> bool {
        self.indent_stack.last().map_or(false, |&e| e == IndentFlags::WroteText)
    }

    #[inline]
    fn wrote_markup(&self) -> bool {
        self.indent_stack.last().map_or(false, |&e| e == IndentFlags::WroteMarkup)
    }

    #[inline]
    fn set_wrote_text(&mut self) {
        if let Some(e) = self.indent_stack.last_mut() {
            *e = IndentFlags::WroteText;
        }
    }

    #[inline]
    fn set_wrote_markup(&mut self) {
        if let Some(e) = self.indent_stack.last_mut() {
            *e = IndentFlags::WroteMarkup;
        }
    }

    fn write_newline(&mut self, target: &mut String, level: usize){
        target.push_str(&self.config.line_separator);
        for _ in 0..level {
            target.push_str(&self.config.indent_string);
        }
    }

    fn before_markup(&mut self, target: &mut String) {
        if self.config.perform_indent && !self.wrote_text() &&
           (self.indent_level > 0 || self.wrote_markup()) {
            let indent_level = self.indent_level;
            self.write_newline(target, indent_level);
            if self.indent_level > 0 && self.config.indent_string.len() > 0 {
                self.after_markup();
            }
        }
    }

    fn after_markup(&mut self) {
        self.set_wrote_markup();
    }

    fn before_start_element(&mut self, target: &mut String) {
        self.before_markup(target);
        self.indent_stack.push(IndentFlags::WroteNothing);
    }

    fn after_start_element(&mut self) {
        self.after_markup();
        self.indent_level += 1;
    }

    fn before_end_element(&mut self, target: &mut String) {
        if self.config.perform_indent && self.indent_level > 0 && self.wrote_markup() &&
           !self.wrote_text() {
            let indent_level = self.indent_level;
            self.write_newline(target, indent_level - 1)
        }
    }

    fn after_end_element(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
            self.indent_stack.pop();
        }
        self.set_wrote_markup();
    }

    fn after_text(&mut self) {
        self.set_wrote_text();
    }

    pub fn emit_start_document(&mut self, target: &mut String,
                                         version: XmlVersion,
                                         encoding: &str,
                                         standalone: Option<bool>) -> Result<()> {
        if self.start_document_emitted {
            return Err(EmitterError::DocumentStartAlreadyEmitted);
        }
        self.start_document_emitted = true;

        self.before_markup(target);
        let result = {
            let mut write = move || {
                write!(target, "<?xml version=\"{version}\" encoding=\"{encoding}\"");

                if let Some(standalone) = standalone {
                    write!(target, " standalone=\"{}\"", if standalone { "yes" } else { "no" });
                }

                write!(target, "?>");

                Ok(())
            };
            write()
        };
        self.after_markup();

        result
    }

    fn check_document_started(&mut self, target: &mut String) -> Result<()> {
        if !self.start_document_emitted && self.config.write_document_declaration {
            self.emit_start_document(target, common::XmlVersion::Version10, "utf-8", None)
        } else {
            Ok(())
        }
    }

    fn fix_non_empty_element(&mut self, target: &mut String) {
        if self.config.normalize_empty_elements && self.just_wrote_start_element {
            self.just_wrote_start_element = false;
            target.push_str(">")
        }
    }

    pub fn emit_processing_instruction(&mut self,
                                                 target: &mut String,
                                                 name: &str,
                                                 data: Option<&str>) -> Result<()> {
        self.check_document_started(target)?;
        self.fix_non_empty_element(target);

        self.before_markup(target);

        let result = {
            let mut write = move || {
                write!(target, "<?{name}");

                if let Some(data) = data {
                    write!(target, " {data}");
                }

                write!(target, "?>");

                Ok(())
            };
            write()
        };

        self.after_markup();

        result
    }

    #[track_caller]
    fn emit_start_element_initial(&mut self, target: &mut String,
                                     name: Name<'_>,
                                     attributes: &[Attribute<'_>]) -> Result<()>
    {
        self.check_document_started(target)?;
        self.fix_non_empty_element(target);
        self.before_start_element(target);
        write!(target, "<{}", name.repr_display());
        self.emit_current_namespace_attributes(target);
        self.emit_attributes(target, attributes);
        self.after_start_element();
        Ok(())
    }

    #[track_caller]
    pub fn emit_start_element(&mut self, target: &mut String,
                                 name: Name<'_>,
                                 attributes: &[Attribute<'_>]) -> Result<()>
    {
        if self.config.keep_element_names_stack {
            self.element_names.push(name.to_owned());
        }

        self.emit_start_element_initial(target, name, attributes)?;
        self.just_wrote_start_element = true;

        if !self.config.normalize_empty_elements {
            write!(target, ">");
        }

        Ok(())
    }

    #[track_caller]
    pub fn emit_current_namespace_attributes(&mut self, target: &mut String)
    {
        for (prefix, uri) in self.nst.peek() {
            match prefix {
                // internal namespaces are not emitted
                NS_XMLNS_PREFIX | NS_XML_PREFIX => (),
                //// there is already a namespace binding with this prefix in scope
                //prefix if self.nst.get(prefix) == Some(uri) => Ok(()),
                // emit xmlns only if it is overridden
                NS_NO_PREFIX => if uri != NS_EMPTY_URI {
                    write!(target, " xmlns=\"{uri}\"")
                },
                // everything else
                prefix => write!(target, " xmlns:{prefix}=\"{uri}\"")
            };
        }
    }

    pub fn emit_attributes(&mut self, target: &mut String,
                                      attributes: &[Attribute<'_>]) {
        for attr in attributes {            
            write!(target, " {}=\"", attr.name.repr_display());
            if self.config.perform_escaping {
                write!(target, "{}", Escaped::<AttributeEscapes>::new(attr.value));
            } else {
                write!(target, "{}", attr.value);
            }
            write!(target, "\"");
        }
    }

    pub fn emit_end_element(&mut self, target: &mut String,
                                      name: Option<Name<'_>>) -> Result<()> {
        let owned_name = if self.config.keep_element_names_stack {
            Some(self.element_names.pop().ok_or(EmitterError::LastElementNameNotAvailable)?)
        } else {
            None
        };

        // Check that last started element name equals to the provided name, if there are both
        if let Some(ref last_name) = owned_name {
            if let Some(ref name) = name {
                if last_name.borrow() != *name {
                    return Err(EmitterError::EndElementNameIsNotEqualToLastStartElementName);
                }
            }
        }

        if let Some(name) = owned_name.as_ref().map(|n| n.borrow()).or(name) {
            Ok(if self.config.normalize_empty_elements && self.just_wrote_start_element {
                self.just_wrote_start_element = false;
                let termination = if self.config.pad_self_closing { " />" } else { "/>" };
                target.push_str(termination);
                self.after_end_element();
            } else {
                self.just_wrote_start_element = false;

                self.before_end_element(target);
                write!(target, "</{}>", name.repr_display());
                self.after_end_element();
            })
        } else {
            Err(EmitterError::EndElementNameIsNotSpecified)
        }
    }

    pub fn emit_cdata(&mut self, target: &mut String, content: &str) {
        self.fix_non_empty_element(target);
        if self.config.cdata_to_characters {
            self.emit_characters(target, content)
        } else {
            // TODO: escape ']]>' characters in CDATA as two adjacent CDATA blocks
            target.push_str("<![CDATA[");
            target.push_str(content);
            target.push_str("]]>");

            self.after_text();
        }
    }

    pub fn emit_characters(&mut self, target: &mut String, content: &str) {
        self.check_document_started(target);
        self.fix_non_empty_element(target);

        if self.config.perform_escaping {
            write!(target, "{}", Escaped::<PcDataEscapes>::new(content));
        } else {
            target.push_str(content);
        }

        self.after_text();
    }

    pub fn emit_comment(&mut self, target: &mut String, content: &str) -> Result<()> {
        self.fix_non_empty_element(target);

        // TODO: add escaping dashes at the end of the comment

        let autopad_comments = self.config.autopad_comments;
        let write = move |target: &mut String| -> Result<()> {
            target.push_str("<!--");

            if autopad_comments && !content.starts_with(char::is_whitespace) {
                target.push_str(" ");
            }

            target.push_str(content);

            if autopad_comments && !content.ends_with(char::is_whitespace) {
                target.push_str(" ");
            }

            target.push_str("-->");

            Ok(())
        };

        self.before_markup(target);
        let result = write(target);
        self.after_markup();

        result
    }
}
