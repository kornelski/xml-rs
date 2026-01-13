#![forbid(unsafe_code)]

use std::io::Cursor;

use xml::common::XmlVersion;
use xml::reader::XmlEvent;
use xml::EventReader;

mod util {
    mod assert_match;
}

#[test]
fn tolerated_versions() {
    for version in ["1.2", "1.7", "1.1075", "1.000"] {
        let source = format!(r#"<?xml version="{version}"?>"#);

        let buf = Cursor::new(source);
        let reader = EventReader::new(buf);
        let mut it = reader.into_iter();

        assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { version: XmlVersion::Version10, .. })));
    }
}

#[test]
fn rejected_xml_versions() {
    let rejected_versions = ["1", "1.", "2.0", "1.0.0", "10", "1.0-", "100", "17.0"];

    for version in rejected_versions {
        let source = format!(r#"<?xml version="{version}"?>"#);

        let buf = Cursor::new(source);
        let reader = EventReader::new(buf);
        let mut it = reader.into_iter();

        assert!(format!("{:?}", it.next()).contains("Invalid XML version"));
    }
}
