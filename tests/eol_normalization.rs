use xml::reader::{EventReader, XmlEvent};

#[test]
fn test_eol_normalization_in_node_content() {
    let xml = "<a>\r\n\t<b>x</b></a>";
    let mut reader = EventReader::from_str(xml);

    loop {
        match reader.next() {
            Ok(XmlEvent::StartDocument { .. }) => continue,
            Ok(XmlEvent::StartElement { name, .. }) => {
                assert_eq!(name.local_name, "a");
                break;
            }
            _ => panic!("Expected start of <a>"),
        }
    }

    // This is the part we want to confirm: \r\n normalized to \n
    assert_eq!(Ok(XmlEvent::Whitespace("\n\t".to_string())), reader.next());

    match reader.next() {
        Ok(XmlEvent::StartElement { name, .. }) => assert_eq!(name.local_name, "b"),
        _ => panic!("Expected start of <b>"),
    }
    match reader.next() {
        Ok(XmlEvent::Characters(data)) => assert_eq!(data, "x"),
        _ => panic!("Expected characters 'x'"),
    }
    match reader.next() {
        Ok(XmlEvent::EndElement { name }) => assert_eq!(name.local_name, "b"),
        _ => panic!("Expected end of <b>"),
    }
    match reader.next() {
        Ok(XmlEvent::EndElement { name }) => assert_eq!(name.local_name, "a"),
        _ => panic!("Expected end of <a>"),
    }
}

#[test]
fn test_eol_normalization_with_intervening_tag() {
    let xml = "<a>\r<b>\n</b></a>";
    let mut reader = EventReader::from_str(xml);

    loop {
        match reader.next() {
            Ok(XmlEvent::StartDocument { .. }) => continue,
            Ok(XmlEvent::StartElement { name, .. }) => {
                assert_eq!(name.local_name, "a");
                break;
            }
            _ => panic!("Expected start of <a>"),
        }
    }

    // \r should be normalized to \n
    assert_eq!(Ok(XmlEvent::Whitespace("\n".to_string())), reader.next());

    match reader.next() {
        Ok(XmlEvent::StartElement { name, .. }) => assert_eq!(name.local_name, "b"),
        _ => panic!("Expected start of <b>"),
    }

    // \n should remain \n
    assert_eq!(Ok(XmlEvent::Whitespace("\n".to_string())), reader.next());

    match reader.next() {
        Ok(XmlEvent::EndElement { name }) => assert_eq!(name.local_name, "b"),
        _ => panic!("Expected end of <b>"),
    }
    match reader.next() {
        Ok(XmlEvent::EndElement { name }) => assert_eq!(name.local_name, "a"),
        _ => panic!("Expected end of <a>"),
    }
}

#[test]
fn test_eol_normalization_complex() {
    let test_cases = [
        ("\r\r\n", "\n\n"),
        ("\r\n\n", "\n\n"),
        ("\r\r\n\n", "\n\n\n"),
        ("\r\n\r\n", "\n\n"),
        ("\r\r\r", "\n\n\n"),
    ];

    for (input, expected) in test_cases {
        let xml = format!("<a>{}</a>", input);
        let mut reader = EventReader::from_str(&xml);

        loop {
            match reader.next() {
                Ok(XmlEvent::StartDocument { .. }) => continue,
                Ok(XmlEvent::StartElement { name, .. }) => {
                    assert_eq!(name.local_name, "a");
                    break;
                }
                _ => panic!("Expected start of <a> for input {:?}", input),
            }
        }

        match reader.next() {
            Ok(XmlEvent::Whitespace(data)) => {
                assert_eq!(data, expected, "Failed for input {:?}", input)
            }
            Ok(XmlEvent::Characters(data)) => {
                assert_eq!(data, expected, "Failed for input {:?}", input)
            }
            other => panic!(
                "Expected whitespace or characters for input {:?}, got {:?}",
                input, other
            ),
        }
    }
}
