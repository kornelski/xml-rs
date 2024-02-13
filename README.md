# xml-no-std, an `xml-rs` fork for `no_std`

[![crates.io][crates-io-img]](https://lib.rs/crates/xml-no-std)
[![docs][docs-img]](https://docs.rs/xml-no-std/)

[Documentation](https://docs.rs/xml-no-std/)

[crates-io-img]: https://img.shields.io/crates/v/xml-no-std.svg
[docs-img]: https://img.shields.io/badge/docs-latest%20release-6495ed.svg

`xml-no-std` is a `no_std` fork of the popular XML library [`xml-rs`](https://github.com/kornelski/xml-rs)
for the [Rust](https://www.rust-lang.org/) programming language. The crate sacrifices streaming capabilities 
and performance for `no_std` compliance (`alloc` is still needed).

All credit goes to [netvl](https://github.com/netvl) and [kornelski](https://github.com/kornelski). 
Thank you for the great work :green_heart:

### Motivation

`xml-no-std` was created in order to support [XML encoding rules](https://www.itu.int/en/ITU-T/asn1/Pages/xer.aspx) 
for the [`librasn` ASN.1 framework](https://github.com/librasn). From the various encoding rules for ASN.1, XML 
encoding rules are usually not chosen for performance-critical use cases. Therefore, the performance losses are tolerable.

### Trade-offs

In order to be compliant with [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) environments, 
`xml-no-std` operates on `Iterator<Item = &u8>` for reading and `alloc::string::String` for writing instead of
`std::io::Read` and `std::io::Write`.
Stream reading is therefore not supported.

As far as performance is concerned, the changes `xml-no-std` makes hit hard when XML documents with 
many attributes in its elements are read. `xml-no-std` uses a `alloc::collections::BTreeSet` for 
storing XML Attributes, which is suboptimal for elements with many attributes. There's definitely
room for improvement here, so contributions are very welcome.

Some ballpark figures from my own dev machine:

| Bench          | `xml-rs`                    | `xml-no-std`                    |
| -------------- | --------------------------- | ------------------------------- |
| read           | 43,255 ns/iter (+/- 1,498)  | 57,263 ns/iter (+/- 1,121)      |
| read_lots_attr | 426,440 ns/iter (+/- 3,932) | 6,122,947 ns/iter (+/- 609,079) |
| write          | 7,405 ns/iter (+/- 31)      | 17,303 ns/iter (+/- 134)        |

## Building and using

xml-no-std uses [Cargo](https://crates.io), so add it with `cargo add xml-no-std` or modify `Cargo.toml`:

```toml
[dependencies]
xml-no-std = "0.8.16"
```

The package exposes a single crate called `xml-no-std`.

## Reading XML documents

[`xml::reader::EventReader`](EventReader) requires an [`Iterator`](https://doc.rust-lang.org/core/iter/trait.Iterator.html) 
over `&u8` items to read from. 

[EventReader]: https://docs.rs/xml-rs/latest/xml/reader/struct.EventReader.html

`EventReader` implements `IntoIterator` trait, so you can use it in a `for` loop directly:

```rust
use std::fs::File;
use std::io::BufReader;

use xml_no_std::reader::{EventReader, XmlEvent};

fn main() -> std::io::Result<()> {
    let mut input = String::new();
    let file = File::open("file.xml")?.read_to_string(&mut input);

    let parser = EventReader::new(input.as_bytes().iter());
    let mut depth = 0;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                println!("{:spaces$}+{name}", "", spaces = depth * 2);
                depth += 1;
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                println!("{:spaces$}-{name}", "", spaces = depth * 2);
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            // There's more: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html
            _ => {}
        }
    }

    Ok(())
}
```

Document parsing can end normally or with an error. Regardless of exact cause, the parsing
process will be stopped, and the iterator will terminate normally.

You can also have finer control over when to pull the next event from the parser using its own
`next()` method:

```rust,ignore
match parser.next() {
    ...
}
```

Upon the end of the document or an error, the parser will remember the last event and will always
return it in the result of `next()` call afterwards. If iterator is used, then it will yield
error or end-of-document event once and will produce `None` afterwards.

It is also possible to tweak parsing process a little using [`xml::reader::ParserConfig`][ParserConfig] structure.
See its documentation for more information and examples.

[ParserConfig]: https://docs.rs/xml-rs/latest/xml/reader/struct.ParserConfig.html

## Parsing untrusted inputs

The parser is written in safe Rust subset, so by Rust's guarantees the worst that it can do is to cause a panic.
You can use `ParserConfig` to set limits on maximum lenghts of names, attributes, text, entities, etc.

## Writing XML documents

xml-rs also provides a streaming writer much like StAX event writer. With it you can write an
XML document to any `Write` implementor.

```rust,no_run
use std::io;
use xml::writer::{EmitterConfig, XmlEvent};

/// A simple demo syntax where "+foo" makes `<foo>`, "-foo" makes `</foo>`
fn make_event_from_line(line: &str) -> XmlEvent {
    let line = line.trim();
    if let Some(name) = line.strip_prefix("+") {
        XmlEvent::start_element(name).into()
    } else if line.starts_with("-") {
        XmlEvent::end_element().into()
    } else {
        XmlEvent::characters(line).into()
    }
}

fn main() -> io::Result<()> {
    let input = io::stdin();
    let out = io::stdout();
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer();

    let mut line = String::new();
    loop {
        line.clear();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            break; // EOF
        }

        let event = make_event_from_line(&line);
        if let Err(e) = writer.write(event) {
            panic!("Write error: {e}")
        }
    }
    out.write_all(writer.into_inner().as_bytes())
}
```

The code example above also demonstrates how to create a writer out of its configuration.
Similar thing also works with `EventReader`.

The library provides an XML event building DSL which helps to construct complex events,
e.g. ones having namespace definitions. Some examples:

```rust,ignore
// <a:hello a:param="value" xmlns:a="urn:some:document">
XmlEvent::start_element("a:hello").attr("a:param", "value").ns("a", "urn:some:document")

// <hello b:config="name" xmlns="urn:default:uri">
XmlEvent::start_element("hello").attr("b:config", "value").default_ns("urn:defaul:uri")

// <![CDATA[some unescaped text]]>
XmlEvent::cdata("some unescaped text")
```

Of course, one can create `XmlEvent` enum variants directly instead of using the builder DSL.
There are more examples in [`xml::writer::XmlEvent`][XmlEvent] documentation.

[XmlEvent]: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html

The writer has multiple configuration options; see `EmitterConfig` documentation for more
information.

[EmitterConfig]: https://docs.rs/xml-rs/latest/xml/writer/struct.EmitterConfig.html

## Bug reports

Please report issues concerning core XML reading and writing at: <https://github.com/kornelski/xml-rs/issues>.
Please report issues concerning the no-std fork at: <https://github.com/6d7a/xml-no-std/issues>.
