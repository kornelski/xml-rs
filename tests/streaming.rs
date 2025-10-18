#![forbid(unsafe_code)]

use std::io::{Cursor, Write};

use xml::EventReader;
use xml::reader::{ParserConfig, XmlEvent};

macro_rules! assert_match {
    ($actual:expr, $( $expected:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        assert_match!($actual, $( $expected )|+ $( if $guard )?, "assert_match failed");
    };
    ($actual:expr, $( $expected:pat_param )|+ $( if $guard: expr )?, $($arg:tt)+) => {
        #[allow(unused)]
        match $actual {
            $( $expected )|+ => {},
            ref actual => panic!("{msg}\nexpect: `{expected}`\nactual: `{actual:?}`",
                msg = $($arg)+, expected = stringify!($( $expected )|+ $( if $guard: expr )?), actual = actual),
        };
    };
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
fn stylesheet_pi_escaping() {
    let source = r#"<?xml version="1.0" standalone="no"?>
        <!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.0//EN"
        "http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd">
        <?xml-stylesheet type="text/css" href="../resources/test.css" ?>
        "#;


    let buf = Cursor::new(source);
    let reader = EventReader::new(buf);

    let mut it = reader.into_iter();

    assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { .. })));
    assert_match!(it.next(), Some(Ok(XmlEvent::Doctype { .. })));
    let pi = it.next();
    assert_match!(pi, Some(Ok(XmlEvent::ProcessingInstruction { ref name, ref data })) if name == "xml-stylesheet" && data.as_deref() == Some(r#"type="text/css" href="../resources/test.css" "#), "{pi:#?}");
}
