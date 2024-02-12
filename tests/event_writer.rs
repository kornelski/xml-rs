#![forbid(unsafe_code)]

use std::fs::File;
use std::io::prelude::*;

use xml::reader::EventReader;
use xml::writer::EmitterConfig;

macro_rules! unwrap_all {
    ($($e:expr);+) => {{
        $($e.unwrap();)+
    }}
}

#[test]
fn reading_writing_equal_with_namespaces() {
    let mut f = String::new();
    let _ = File::open("tests/documents/sample_2.xml").unwrap().read_to_string(&mut f);

    let r = EventReader::new(f.as_bytes().iter());
    let mut w = EmitterConfig::default().perform_indent(true).create_writer();

    for e in r {
        match e {
            Ok(e) => if let Some(e) = e.as_writer_event() {
                match w.write(e) {
                    Ok(()) => {},
                    Err(e) => panic!("Writer error: {e:?}")
                }
            },
            Err(e) => panic!("Error: {e}"),
        }
    }
    

    assert_eq!(f.trim(), w.into_inner().trim());
}

#[test]
fn writing_simple() {
    use xml::writer::XmlEvent;
    
    let mut w = EmitterConfig::new().write_document_declaration(false).create_writer();
    w.write(XmlEvent::start_element("h:hello").ns("h", "urn:hello-world")).unwrap();
    w.write("hello world").unwrap();
    w.write(XmlEvent::end_element()).unwrap();
    

    assert_eq!(
        &w.into_inner(),
        r#"<h:hello xmlns:h="urn:hello-world">hello world</h:hello>"#
    );
}

#[test]
fn writing_empty_elements_with_normalizing() {
    use xml::writer::XmlEvent;

   let mut w = EmitterConfig::new().write_document_declaration(false).create_writer();

    unwrap_all! {
        w.write(XmlEvent::start_element("hello"));
        w.write(XmlEvent::start_element("world"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }

    assert_eq!(&w.into_inner(), r#"<hello><world /></hello>"#);
}

#[test]
fn writing_empty_elements_without_normalizing() {
    use xml::writer::XmlEvent;

    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .normalize_empty_elements(false)
        .create_writer();
    unwrap_all! {
        w.write(XmlEvent::start_element("hello"));
        w.write(XmlEvent::start_element("world"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }
    

    assert_eq!(&w.into_inner(), r#"<hello><world></world></hello>"#);
}

#[test]
fn writing_empty_elements_without_pad_self_closing() {
    use xml::writer::XmlEvent;

    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .pad_self_closing(false)
        .create_writer();
    unwrap_all! {
        w.write(XmlEvent::start_element("hello"));
        w.write(XmlEvent::start_element("world"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }
    

    assert_eq!(&w.into_inner(), r#"<hello><world/></hello>"#);
}
#[test]
fn writing_empty_elements_pad_self_closing_explicit() {
    use xml::writer::XmlEvent;
        
    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .pad_self_closing(true)
        .create_writer();
    unwrap_all! {
        w.write(XmlEvent::start_element("hello"));
        w.write(XmlEvent::start_element("world"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }
    

    assert_eq!(&w.into_inner(), r#"<hello><world /></hello>"#);
}

#[test]
fn writing_comments_with_indentation() {
    use xml::writer::XmlEvent;

    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .perform_indent(true)
        .create_writer();
    unwrap_all! {
        w.write(XmlEvent::start_element("hello"));
        w.write(XmlEvent::start_element("world"));
        w.write(XmlEvent::comment("  this is a manually padded comment\t"));
        w.write(XmlEvent::comment("this is an unpadded comment"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }
    

    assert_eq!(
        &w.into_inner(),
        "<hello>
  <world>
    <!--  this is a manually padded comment\t-->
    <!-- this is an unpadded comment -->
  </world>
</hello>"
    );
}

#[test]
fn issue_112_overriding_namepace_prefix() {
    use xml::writer::XmlEvent;

    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .create_writer();
    unwrap_all! {
        w.write(XmlEvent::start_element("iq").ns("", "jabber:client").ns("a", "urn:A"));
        w.write(XmlEvent::start_element("bind").ns("", "urn:ietf:params:xml:ns:xmpp-bind"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::start_element("whatever").ns("a", "urn:X"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element())
    }
    

    assert_eq!(
        &w.into_inner(),
        r#"<iq xmlns="jabber:client" xmlns:a="urn:A"><bind xmlns="urn:ietf:params:xml:ns:xmpp-bind" /><whatever xmlns:a="urn:X" /></iq>"#
    );
}

#[test]
fn attribute_escaping() {
    use xml::writer::XmlEvent;

    let mut w = EmitterConfig::new()
        .write_document_declaration(false)
        .perform_indent(true)
        .create_writer();
    unwrap_all! {
        w.write(
            XmlEvent::start_element("hello")
                .attr("testLt", "<")
                .attr("testGt", ">")
        );
        w.write(XmlEvent::end_element());
        w.write(
            XmlEvent::start_element("hello")
                .attr("testQuot", "\"")
                .attr("testApos", "\'")
        );
        w.write(XmlEvent::end_element());
        w.write(
            XmlEvent::start_element("hello")
                .attr("testAmp", "&")
        );
        w.write(XmlEvent::end_element());
        w.write(
            XmlEvent::start_element("hello")
                .attr("testNl", "\n")
                .attr("testCr", "\r")
        );
        w.write(XmlEvent::end_element());
        w.write(
            XmlEvent::start_element("hello")
                .attr("testNl", "\\n")
                .attr("testCr", "\\r")
        );
        w.write(XmlEvent::end_element())
    }

    assert_eq!(
        &w.into_inner(),
        "<hello testLt=\"&lt;\" testGt=\"&gt;\" />
<hello testQuot=\"&quot;\" testApos=\"&apos;\" />
<hello testAmp=\"&amp;\" />
<hello testNl=\"&#xA;\" testCr=\"&#xD;\" />
<hello testNl=\"\\n\" testCr=\"\\r\" />"
    );
}
