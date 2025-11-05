#![forbid(unsafe_code)]

use std::io::Cursor;

use xml::common::XmlVersion;
use xml::reader::XmlEvent;
use xml::EventReader;

mod assert_match;

#[test]
fn accepted_xml_versions() {
    let accepted_versions_enum = [
        XmlVersion::Version10,
        XmlVersion::Version11,
        XmlVersion::Version1x("1.2".into()),
        XmlVersion::Version1x("1.7".into()),
        XmlVersion::Version1x("1.1075".into()),
        XmlVersion::Version1x("1.000".into()),
    ];

    for (i, version) in accepted_versions_enum.iter().enumerate() {
        let source = format!(r#"<?xml version="{version}"?>"#, version = version.as_str());

        let buf = Cursor::new(source);
        let reader = EventReader::new(buf);
        let mut it = reader.into_iter();

        assert_match!(it.next(), Some(Ok(XmlEvent::StartDocument { version: v, .. })) if v == accepted_versions_enum[i]);
    }
}

#[test]
fn rejected_xml_versions() {
    let rejected_versions = ["1", "1.", "2.0", "1.0.0", "10", "1.0-", "100", "17.0"];

    for version in rejected_versions {
        let source = format!(r#"<?xml version="{version}"?>"#, version = version);

        let buf = Cursor::new(source);
        let reader = EventReader::new(buf);
        let mut it = reader.into_iter();

        assert!(format!("{:?}", it.next()).contains("Invalid XML version"));
    }
}
