//! # syntect-tui
//!
//! `syntect-tui` is a lightweight toolset for converting from text stylised by
//! [syntect](https://docs.rs/syntect/latest/syntect) into stylised text renderable in
//! [tui](https://docs.rs/tui/0.10.0/tui) applications.
//!
//! Contributions welcome! Feel free to fork and submit a pull request.
use custom_error::custom_error;

custom_error! {
    #[derive(PartialEq)]
    pub SyntectTuiError
    UnknownFontStyle { bits: u8 } = "Unable to convert syntect::FontStyle into tui::Modifier: unsupported bits ({bits}) value.",
}

/// Converts a line segment highlighed using [syntect::easy::HighlightLines::highlight_line](https://docs.rs/syntect/latest/syntect/easy/struct.HighlightLines.html#method.highlight_line) into a [tui::text::Span](https://docs.rs/tui/0.10.0/tui/text/struct.Span.html).
///
/// Syntect colours are RGBA while Tui colours are RGB, so colour conversion is lossy. However, if a Syntect colour's alpha value is `0`, then we preserve this to some degree by returning a value of `None` for that colour (i.e. its colourless).
///
/// # Examples
/// Basic usage:
/// ```
/// let input_text = "hello";
/// let input_style = syntect::highlighting::Style {
///     foreground: syntect::highlighting::Color { r: 255, g: 0, b: 0, a: 255 },
///     background: syntect::highlighting::Color { r: 0, g: 0, b: 0, a: 0 },
///     font_style: syntect::highlighting::FontStyle::BOLD
/// };
/// let expected_style = tui::style::Style {
///     fg: Some(tui::style::Color::Rgb(255, 0, 0)),
///     bg: None,
///     add_modifier: tui::style::Modifier::BOLD,
///     sub_modifier: tui::style::Modifier::empty()
/// };
/// let expected_span = tui::text::Span::styled(input_text, expected_style);
/// let actual_span = syntect_tui::into_span((input_style, input_text)).unwrap();
/// assert_eq!(expected_span, actual_span);
/// ```
///
/// Here's a more complex example that builds upon syntect's own example for `HighlightLines`:
/// ```
/// use syntect::easy::HighlightLines;
/// use syntect::parsing::SyntaxSet;
/// use syntect::highlighting::{ThemeSet, Style};
/// use syntect::util::LinesWithEndings;
/// use syntect_tui::into_span;
///
/// let ps = SyntaxSet::load_defaults_newlines();
/// let ts = ThemeSet::load_defaults();
/// let syntax = ps.find_syntax_by_extension("rs").unwrap();
/// let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
/// let s = "pub struct Wow { hi: u64 }\nfn blah() -> u64 {}";
/// for line in LinesWithEndings::from(s) { // LinesWithEndings enables use of newlines mode
///     let line_spans: Vec<tui::text::Span> =
///         h.highlight_line(line, &ps)
///          .unwrap()
///          .into_iter()
///          .filter_map(|segment| into_span(segment).ok())
///          .collect();
///     let spans = tui::text::Spans::from(line_spans);
///     print!("{:?}", spans);
/// }
///
/// ```
///
/// # Errors
/// Can return `SyntectTuiError::UnknownFontStyle` if the input [FontStyle](https://docs.rs/syntect/latest/syntect/highlighting/struct.FontStyle.html) is not supported.
///
/// All explicit compositions of `BOLD`, `ITALIC` & `UNDERLINE` are supported, however, implicit bitflag coercions are not. For example, even though `FontStyle::from_bits(3)` is coerced to `Some(FontStyle::BOLD | FontStyle::ITALIC)`, we ignore this result as it would be a pain to handle all implicit coercions.
pub fn into_span<'a>(
    (style, content): (syntect::highlighting::Style, &'a str),
) -> Result<tui::text::Span<'a>, SyntectTuiError> {
    Ok(tui::text::Span::styled(
        String::from(content),
        translate_style(style)?,
    ))
}

/// Converts a
/// [syntect::highlighting::Style](https://docs.rs/syntect/latest/syntect/highlighting/struct.Style.html)
/// into a [tui::style::Style](https://docs.rs/tui/0.10.0/tui/style/struct.Style.html).
///
/// Syntect colours are RGBA while Tui colours are RGB, so colour conversion is lossy. However, if a Syntect colour's alpha value is `0`, then we preserve this to some degree by returning a value of `None` for that colour (i.e. its colourless).
///
/// # Examples
/// Basic usage:
/// ```
/// let input = syntect::highlighting::Style {
///     foreground: syntect::highlighting::Color { r: 255, g: 0, b: 0, a: 255 },
///     background: syntect::highlighting::Color { r: 0, g: 0, b: 0, a: 0 },
///     font_style: syntect::highlighting::FontStyle::BOLD
/// };
/// let expected = tui::style::Style {
///     fg: Some(tui::style::Color::Rgb(255, 0, 0)),
///     bg: None,
///     add_modifier: tui::style::Modifier::BOLD,
///     sub_modifier: tui::style::Modifier::empty()
/// };
/// let actual = syntect_tui::translate_style(input).unwrap();
/// assert_eq!(expected, actual);
/// ```
/// # Errors
/// Can return `SyntectTuiError::UnknownFontStyle` if the input [FontStyle](https://docs.rs/syntect/latest/syntect/highlighting/struct.FontStyle.html) is not supported.
///
/// All explicit compositions of `BOLD`, `ITALIC` & `UNDERLINE` are supported, however, implicit bitflag coercions are not. For example, even though `FontStyle::from_bits(3)` is coerced to `Some(FontStyle::BOLD | FontStyle::ITALIC)`, we ignore this result as it would be a pain to handle all implicit coercions.
pub fn translate_style(
    syntect_style: syntect::highlighting::Style,
) -> Result<tui::style::Style, SyntectTuiError> {
    Ok(tui::style::Style {
        fg: translate_colour(syntect_style.foreground),
        bg: translate_colour(syntect_style.background),
        add_modifier: translate_font_style(syntect_style.font_style)?,
        sub_modifier: tui::style::Modifier::empty(),
    })
}

/// Converts a
/// [syntect::highlighting::Color](https://docs.rs/syntect/latest/syntect/highlighting/struct.Color.html)
/// into a [tui::style::Color](https://docs.rs/tui/0.10.0/tui/style/enum.Color.html).
///
///
/// # Examples
/// Basic usage:
/// ```
/// let input = syntect::highlighting::Color { r: 255, g: 0, b: 0, a: 255 };
/// let expected = Some(tui::style::Color::Rgb(255, 0, 0));
/// let actual = syntect_tui::translate_colour(input);
/// assert_eq!(expected, actual);
/// ```
/// Syntect colours are RGBA while Tui colours are RGB, so colour conversion is lossy. However, if a Syntect colour's alpha value is `0`, then we preserve this to some degree by returning a value of `None` for that colour (i.e. colourless):
/// ```
/// assert_eq!(
///     None,
///     syntect_tui::translate_colour(syntect::highlighting::Color { r: 255, g: 0, b: 0, a: 0 })
/// );
/// ```
pub fn translate_colour(syntect_color: syntect::highlighting::Color) -> Option<tui::style::Color> {
    match syntect_color {
        syntect::highlighting::Color { r, g, b, a } if a > 0 => {
            Some(tui::style::Color::Rgb(r, g, b))
        }
        _ => None,
    }
}

/// Converts a
/// [syntect::highlighting::FontStyle](https://docs.rs/syntect/latest/syntect/highlighting/struct.FontStyle.html)
/// into a [tui::style::Modifier](https://docs.rs/tui/0.10.0/tui/style/struct.Modifier.html).
///
///
/// # Examples
/// Basic usage:
/// ```
/// let input = syntect::highlighting::FontStyle::BOLD | syntect::highlighting::FontStyle::ITALIC;
/// let expected = tui::style::Modifier::BOLD | tui::style::Modifier::ITALIC;
/// let actual = syntect_tui::translate_font_style(input).unwrap();
/// assert_eq!(expected, actual);
/// ```
/// # Errors
/// Can return `SyntectTuiError::UnknownFontStyle` if the input [FontStyle](https://docs.rs/syntect/latest/syntect/highlighting/struct.FontStyle.html) is not supported.
///
/// All explicit compositions of `BOLD`, `ITALIC` & `UNDERLINE` are supported, however, implicit bitflag coercions are not. For example, even though `FontStyle::from_bits(3)` is coerced to `Some(FontStyle::BOLD | FontStyle::ITALIC)`, we ignore this result as it would be a pain to handle all implicit coercions.
pub fn translate_font_style(
    syntect_font_style: syntect::highlighting::FontStyle,
) -> Result<tui::style::Modifier, SyntectTuiError> {
    use syntect::highlighting::FontStyle;
    use tui::style::Modifier;
    match syntect_font_style {
        x if x == FontStyle::empty() => Ok(Modifier::empty()),
        x if x == FontStyle::BOLD => Ok(Modifier::BOLD),
        x if x == FontStyle::ITALIC => Ok(Modifier::ITALIC),
        x if x == FontStyle::UNDERLINE => Ok(Modifier::UNDERLINED),
        x if x == FontStyle::BOLD | FontStyle::ITALIC => Ok(Modifier::BOLD | Modifier::ITALIC),
        x if x == FontStyle::BOLD | FontStyle::UNDERLINE => {
            Ok(Modifier::BOLD | Modifier::UNDERLINED)
        }
        x if x == FontStyle::ITALIC | FontStyle::UNDERLINE => {
            Ok(Modifier::ITALIC | Modifier::UNDERLINED)
        }
        x if x == FontStyle::BOLD | FontStyle::ITALIC | FontStyle::UNDERLINE => {
            Ok(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
        }
        unknown => Err(SyntectTuiError::UnknownFontStyle {
            bits: unknown.bits(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use syntect::highlighting::{Color as SyntectColour, FontStyle, Style as SyntectStyle};
    use tui::style::Modifier;
    use tui::text::Span;

    fn fake_syntect_colour(r: u8, g: u8, b: u8, a: u8) -> SyntectColour {
        SyntectColour { r, g, b, a }
    }

    #[test]
    fn can_convert_to_span() {
        let (r, g, b) = (012_u8, 123_u8, 234_u8);
        let style = SyntectStyle {
            font_style: FontStyle::UNDERLINE,
            foreground: fake_syntect_colour(r, g, b, 128),
            background: fake_syntect_colour(g, b, r, 128),
        };
        let content = "syntax";
        let expected = Ok(Span {
            content: std::borrow::Cow::Owned(String::from(content)),
            style: tui::style::Style {
                fg: Some(tui::style::Color::Rgb(r, g, b)),
                bg: Some(tui::style::Color::Rgb(g, b, r)),
                add_modifier: Modifier::UNDERLINED,
                sub_modifier: Modifier::empty(),
            },
        });
        let actual = into_span((style, content));
        assert_eq!(expected, actual);
    }

    #[test]
    fn translate_style_ok() {
        let (r, g, b) = (012_u8, 123_u8, 234_u8);
        let input = SyntectStyle {
            font_style: FontStyle::UNDERLINE,
            foreground: fake_syntect_colour(r, g, b, 128),
            background: fake_syntect_colour(g, b, r, 128),
        };
        let expected = Ok(tui::style::Style::default()
            .fg(tui::style::Color::Rgb(r, g, b))
            .bg(tui::style::Color::Rgb(g, b, r))
            .add_modifier(Modifier::UNDERLINED));
        let actual = translate_style(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn translate_style_err() {
        let colour = fake_syntect_colour(012, 123, 234, 128);
        let input = SyntectStyle {
            font_style: unsafe { FontStyle::from_bits_unchecked(254) },
            foreground: colour.to_owned(),
            background: colour,
        };
        let expected = Err(SyntectTuiError::UnknownFontStyle { bits: 254 });
        let actual = translate_style(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::with_alpha(
        fake_syntect_colour(012, 123, 234, 128),
        Some(tui::style::Color::Rgb(012, 123, 234))
    )]
    #[case::without_alpha(fake_syntect_colour(012, 123, 234, 0), None)]
    fn check_translate_colour(
        #[case] input: SyntectColour,
        #[case] expected: Option<tui::style::Color>,
    ) {
        assert_eq!(expected, translate_colour(input));
    }

    #[rstest]
    #[case::empty(FontStyle::empty(), Ok(Modifier::empty()))]
    #[case::bold(FontStyle::BOLD, Ok(Modifier::BOLD))]
    #[case::italic(FontStyle::ITALIC, Ok(Modifier::ITALIC))]
    #[case::underline(FontStyle::UNDERLINE, Ok(Modifier::UNDERLINED))]
    #[case::bold_italic(FontStyle::BOLD | FontStyle::ITALIC, Ok(Modifier::BOLD | Modifier::ITALIC))]
    #[case::bold_underline(FontStyle::BOLD | FontStyle::UNDERLINE, Ok(Modifier::BOLD | Modifier::UNDERLINED))]
    #[case::italic_underline(FontStyle::ITALIC | FontStyle::UNDERLINE, Ok(Modifier::ITALIC | Modifier::UNDERLINED))]
    #[case::bold_italic_underline(
        FontStyle::BOLD | FontStyle::ITALIC | FontStyle::UNDERLINE,
        Ok(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED)
    )]
    #[case::err(
        unsafe { FontStyle::from_bits_unchecked(254) } ,
        Err(SyntectTuiError::UnknownFontStyle { bits: 254 })
    )]
    fn check_translate_font_style(
        #[case] input: FontStyle,
        #[case] expected: Result<Modifier, SyntectTuiError>,
    ) {
        let actual = translate_font_style(input);
        assert_eq!(expected, actual);
    }
}
