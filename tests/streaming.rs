#![forbid(unsafe_code)]

use std::io::{Cursor, Write};

use xml::reader::{ParserConfig, XmlEvent};
use xml::EventReader;

mod util {
    mod assert_match;
}

fn write_and_reset_position<W>(c: &mut Cursor<W>, data: &[u8]) where Cursor<W>: Write {
    let p = c.position();
    c.write_all(data).unwrap();
    c.set_position(p);
}

#[test]
fn reading_streamed_content() {
    let buf = Cursor::new(b"<root>".to_vec());
    let reader = EventReader::new(buf);

    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "root");

    write_and_reset_position(it.source_mut(), b"<child-1>content</child-1>");
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-1");
    assert_match!(it.next(), Some(Ok(XmlEvent::Characters(ref c))) if c == "content");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "child-1");

    write_and_reset_position(it.source_mut(), b"<child-2/>");
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-2");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "child-2");

    write_and_reset_position(it.source_mut(), b"<child-3/>");
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-3");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "child-3");
    // doesn't seem to work because of how tags parsing is done
//    write_and_reset_position(it.source_mut(), b"some text");
   // assert_match!(it.next(), Some(Ok(XmlEvent::Characters(ref c))) if c == "some text");

    write_and_reset_position(it.source_mut(), b"</root>");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "root");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndDocument)));
    assert_match!(it.next(), None);
}

#[test]
fn reading_streamed_content2() {
    let buf = Cursor::new(b"<root>".to_vec());
    let mut config = ParserConfig::new();
    config.ignore_end_of_stream = true;
    let readerb = EventReader::new_with_config(buf, config);

    let mut reader = readerb.into_iter();

    assert_match!(reader.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(reader.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "root");

    write_and_reset_position(reader.source_mut(), b"<child-1>content</child-1>");
    assert_match!(reader.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-1");
    assert_match!(reader.next(), Some(Ok(XmlEvent::Characters(ref c))) if c == "content");
    assert_match!(reader.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "child-1");

    write_and_reset_position(reader.source_mut(), b"<child-2>content</child-2>");

    assert_match!(reader.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-2");
    assert_match!(reader.next(), Some(Ok(XmlEvent::Characters(ref c))) if c == "content");
    assert_match!(reader.next(), Some(Ok(XmlEvent::EndElement { ref name })) if name.local_name == "child-2");
    assert_match!(reader.next(), Some(Err(_)));
    write_and_reset_position(reader.source_mut(), b"<child-3></child-3>");
    assert_match!(reader.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-3");
    write_and_reset_position(reader.source_mut(), b"<child-4 type='get'");
    match reader.next() {
        None | Some(Ok(_)) => {
            panic!("At this point, parser must not detect something.");
        },
        Some(Err(_)) => {},
    }
    write_and_reset_position(reader.source_mut(), b" />");
    assert_match!(reader.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "child-4");
}

#[test]
fn late_ns_binding() {
    let reader = EventReader::new(Cursor::new(r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:â="urn:x-test:U+00E2">
        <ê:test id="test" xmlns:ê="urn:x-test:U+00EA" â:âAttr="âValue"/>
        </html>
    "#));

    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    let XmlEvent::StartElement { name, attributes, namespace } = it.next().unwrap().unwrap() else { panic!() };
    assert_eq!("urn:x-test:U+00E2", namespace.0["â"]);
    assert!(attributes.is_empty());
    assert_eq!(("http://www.w3.org/1999/xhtml", "html"), name);
}

#[test]
fn stylesheet_pi_escaping() {
    let source = r#"<?xml version="1.0" standalone="no"?>
        <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.0//EN"
        "http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd">
        <?xml-stylesheet type="text/css" href="../resources/test.css" ?>
        <root>
        &custom;
        </root>
        "#;

    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);

    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { .. })));
    it.add_entities([("custom", "okay")]).unwrap();
    let pi = it.next();
    assert_match!(pi, Some(Ok(XmlEvent::ProcessingInstruction { ref name, ref data })) if name == "xml-stylesheet" && data.as_deref() == Some(r#"type="text/css" href="../resources/test.css" "#), "{pi:#?}");
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { .. })));
    assert!(it.add_entities([("too", "late")]).is_err());
    assert_match!(it.next(), Some(Ok(XmlEvent::Characters(c))) if c.trim() == "okay");
}


#[test]
fn unicode_attribute() {
    let source = r#"<xml xmlns:â="_"><b:t â:a="_" xmlns:b="_"/></xml>"#;

    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);

    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { name, attributes: _ , namespace: _ })) if name.local_name == "xml");
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { name, attributes , namespace: _ }))
        if name.prefix == Some(String::from("b"))
        && name.local_name == "t"
        && attributes[0].name.prefix == Some(String::from("â"))
        && attributes[0].name.local_name == "a" );
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name }))
        if name.prefix == Some(String::from("b"))
        && name.local_name == "t");
    assert_match!(it.next(), Some(Ok(XmlEvent::EndElement { ref name }))
        if name.local_name == "xml");
}

#[test]
fn no_double_colon_in_tag_name() {
    let source = r#"<root::element/>"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert!(format!("{:?}", it.next()).contains("pos: 1:7, kind: Syntax(\"Unexpected token inside qualified name: :\")"));
}

#[test]
fn no_double_prefix() {
    let source = r#"<root><a:b:c/></root>"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::StartElement { ref name, .. })) if name.local_name == "root");
    assert!(format!("{:?}", it.next()).contains("pos: 1:11, kind: Syntax(\"Unexpected token inside qualified name: :\")"));
}


#[test]
fn no_double_colon_in_attr_name() {
    let source = r#"<root a::c="_"/>"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert!(format!("{:?}", it.next()).contains("pos: 1:9, kind: Syntax(\"Unexpected token inside qualified name: :\")"));
}

#[test]
fn doctype_public_sytem() {
    let source = r#"<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { syntax })));
    let d = it.doctype_ids().unwrap();
    assert_eq!(d.name(), "svg");
    assert_eq!(d.public_id(), Some("-//W3C//DTD SVG 1.1//EN"));
    assert_eq!(d.system_id(), Some("http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd"));
}

#[test]
fn doctype_system_only() {
    let source = r#"<!DOCTYPE svg SYSTEM "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { syntax })));
    let d = it.doctype_ids().unwrap();
    assert_eq!(d.name(), "svg");
    assert_eq!(d.public_id(), None);
    assert_eq!(d.system_id(), Some("http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd"));
}

#[test]
fn doctype_name_only_with_space() {
    let source = r#"<!DOCTYPE svg >"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { syntax })));
    let d = it.doctype_ids().unwrap();
    assert_eq!(d.name(), "svg");
    assert_eq!(d.public_id(), None);
    assert_eq!(d.system_id(), None);
}

#[test]
fn doctype_name_only_name_closing_tag() {
    let source = r#"<!DOCTYPE svg>"#;
    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);
    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { syntax })));
    let d = it.doctype_ids().unwrap();
    assert_eq!(d.name(), "svg");
    assert_eq!(d.public_id(), None);
    assert_eq!(d.system_id(), None);
}
