# syntect-tui [![Build Status](https://app.travis-ci.com/chanq-io/syntect-tui.svg?branch=main)](https://app.travis-ci.com/chanq-io/syntect-tui)
A lightweight translation layer between [syntect](https://github.com/trishume/syntect) and
[tui-rs](https://github.com/fdehau/tui-rs) style types. If you're building a CLI app with a UI powered by tui.rs and need syntax highlighting, then you may find this crate useful!

Given the limited scope of this crate I do not have any plans to extend its functionality any further, but am open to contributions. Feel free to fork and submit a pull request.

## Getting Started
`syntect-tui` is [available on crates.io](https://crates.io/crates/syntect-tui). You can install it by adding the following line to your `Cargo.toml`:

```
syntect-tui = "1.0"
```

## Docs
For more usage information read the [docs](https://docs.rs/syntect-tui/latest/syntect_tui/)

## Example Code
Building upon [syntect's simple example](https://github.com/trishume/syntect#example-code), here's a
snippet that parses some rust code, highlights it using syntect and converts it into
[tui::text::Spans](https://docs.rs/tui/latest/tui/text/struct.Spans.html) ready for rendering in a tui appliction:
```
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::LinesWithEndings;
use syntui::into_span;

let ps = SyntaxSet::load_defaults_newlines();
let ts = ThemeSet::load_defaults();
let syntax = ps.find_syntax_by_extension("rs").unwrap();
let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
let s = "pub struct Wow { hi: u64 }\nfn blah() -> u64 {}";
for line in LinesWithEndings::from(s) { // LinesWithEndings enables use of newlines mode
    let line_spans: Vec<tui::text::Span> =
        h.highlight_line(line, &ps)
         .unwrap()
         .into_iter()
         .filter_map(|segment| into_span(segment).ok())
         .collect();
    let spans = tui::text::Spans::from(line_spans);
    print!("{:?}", spans);
}

```

## Licence & Acknowledgements
Thanks to [trishume](https://github.com/trishume) and [fdehau](https://github.com/fdehau/) for building `sytect` & `tui`! All code is released under the MIT License.
