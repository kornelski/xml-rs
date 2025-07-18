## Version 1.0.0

* Added `Doctype` event
* Marked structs as `#[non_exhaustive]`
* Merged `ParserConfig2` back into `ParserConfig`
* Added option to the writer to pass through XML markup unmodified
* `xml-analyze` binary has been moved to examples
* Writer escapes `--` in comments and `]]>` in CDATA

## Version 0.8.27

* Added detection of invalid `<?` in attributes

## Version 0.8.26

* Fixed buffering of files with a broken UTF-16 encoding

## Version 0.8.25

* `TryFrom` for converting from reader to writer events, to make `.as_writer_event()` more discoverable.

## Version 0.8.24

* Fixed reporting of line/column position of CDATA when trimming whitespace

## Version 0.8.23

* StartDocument event will consistently use uppercase "UTF-8" name for encoding when the document did not declare it expicitly, but beware that documents can still use lowercase encoding names, so you must always use case-insensitive comparisons.

## Version 0.8.22

* Ability to retrieve the whole DOCTYPE. For backwards compatibility, it's a getter on the reader, not an event.

## Version 0.8.21

* Added `EventWriter::inner_ref`
* ~15% performance improvement

## Version 0.8.20

* Fixed escaping of literal `]]>` in CDATA

## Version 0.8.19

* Fixed whitespace event when parsing DOCTYPE with internal subset

## Version 0.8.18

* Option to tolerate invalid entities and chars

## Version 0.8.17

* Added configuration for document size/complexity limits.

## Version 0.8.16

* Fixed error line numbers when parsing CDATA as characters

## Version 0.8.15

* Improved speed of parsing elements with huge number of arguments

## Version 0.8.14

* Fixed error line numbers when ignoring comments

## Version 0.8.13

* Backward-compatibility fix

## Version 0.8.12

* Improved conformance of parsing invalid codepoints, XML prolog
* Reduced number of allocations

## Version 0.8.11

* Improved conformance of PI
* Forbidden invalid multiple root elements, unless an option allowing them is enabled.

## Version 0.8.10

* Improved parsing conformance
* Internal error handling improvements

## Version 0.8.9

* Added support for UTF-16 and ASCII
* Fixed CDATA parsing
* Added PE entities parsing

## Version 0.8.8

* Added recursive entity expansion (with length protection)
* Expanded parsing of DTD

## Version 0.8.7

* Basic parsing of DTD internal subset
* Speed improvements

## Version 0.8.6

* Fixed parsing of incorrectly nested comments and processing instructions

## Version 0.8.5

* Updated source code to edition 2018 and fixed/updated some Rust idioms.

## Version 0.8.4

* Fixed recognition of `?>`, `]]>` and `/>` tokens as characters.
* Fixed writer output operations to use `write_all` to ensure that the data
  is written fully.
* The document declaration is now written before any characters automatically.

## Version 0.8.3

* Added a new parser option, `ignore_root_level_whitespace`, which makes the parser
  skip emitting whitespace events outside of the root element when set to `true`.
  This helps with certain tasks like canonicalization.

## Version 0.8.2

* Added a new parser option, `replace_unknown_entity_references`, which allows to ignore
  invalid Unicode code points and replace them with a Unicode "replacement character"
  during parsing. This can be helpful to deal with e.g. UTF-16 surrogate pairs.
* Added a new emitter option, `pad_self_closing`, which determines the style of the self-closing
  elements when they are emitted: `<a />` (`true`) vs `<a/>` (`false`).

## Version 0.8.1

* Fixed various issues with tests introduced by updates in Rust.
* Adjusted the lexer to ignore contents of the `<!DOCTYPE>` tag.
* Removed unnecessary unsafety in tests.
* Added tests for doc comments in the readme file.
* Switched to GitHub Actions from Travis CI.

## Version 0.8.0

* Same as 0.7.1, with 0.7.1 being yanked because of the incorrect semver bump.

## Version 0.7.1

* Removed dependency on bitflags.
* Added the `XmlWriter::inner_mut()` method.
* Fixed some rustdoc warnings.

## Version 0.7.0

* Same as 0.6.2, with 0.6.2 being yanked because of the incompatible bump of minimum required version of rustc.

## Version 0.6.2

* Bumped `bitflags` to 1.0.

## Version 0.6.1

* Fixed the writer to escape some special characters when writing attribute values.

## Version 0.6.0

* Changed the target type of extra entities from `char` to `String`. This is an incompatible
  change.

## Version 0.5.0

* Added support for ignoring EOF errors in order to read documents from streams incrementally.
* Bumped `bitflags` to 0.9.

## Version 0.4.1

* Added missing `Debug` implementation to `xml::writer::XmlEvent`.

## Version 0.4.0

* Bumped version number, since changes introduced in 0.3.7 break backwards compatibility.

## Version 0.3.8

* Fixed a problem introduced in 0.3.7 with entities in attributes causing parsing errors.

## Version 0.3.7

* Fixed the problem with parsing non-whitespace character entities as whitespace (issue #140).
* Added support for configuring custom entities in the parser configuration.

## Version 0.3.6

* Added an `Error` implementation for `EmitterError`.
* Fixed escaping of strings with multi-byte code points.

## Version 0.3.5

* Added `Debug` implementation for `XmlVersion`.
* Fixed some failing tests.

## Version 0.3.3

* Updated `bitflags` to 0.7.

## Version 0.3.2

* Added `From<io::Error>` for `xml::reader::Error`, which improves usability of working with parsing errors.

## Version 0.3.1

* Bumped `bitflags` dependency to 0.4, some internal warning fixes.

## Version 0.3.0

* Changed error handling in `EventReader` - now I/O errors are properly bubbled up from the lexer.

## Version 0.2.4

* Fixed #112 - incorrect handling of namespace redefinitions when writing a document.

## Version 0.2.3

* Added `into_inner()` methods to `EventReader` and `EventWriter`.

## Version 0.2.2

* Using `join` instead of the deprecated `connect`.
* Added a simple XML analyzer program which demonstrates library usage and can be used to check XML documents for well-formedness.
* Fixed incorrect handling of unqualified attribute names (#107).
* Added this changelog.

## Version 0.2.1

* Fixed #105 - incorrect handling of double dashes.

## Version 0.2.0

* Major update, includes proper document writing support and significant architecture changes.
