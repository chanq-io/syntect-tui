use custom_error::custom_error;
use syntect::highlighting::{Color as SyntectColour, FontStyle, Style as SyntectStyle};
use tui::style::{Color as TuiColour, Modifier, Style as TuiStyle};
use tui::text::Span;

custom_error! {
    #[derive(PartialEq)]
    pub SynTuiError
    UnknownFontStyle { bits: u8 } = "Unable to convert syntect::FontStyle into tui::Modifier: unsupported bits ({bits}) value.",
}

pub fn into_span<'a>((style, content): (SyntectStyle, &'a str)) -> Result<Span<'a>, SynTuiError> {
    Ok(Span::styled(String::from(content), translate_style(style)?))
}

fn translate_style(syntect_style: SyntectStyle) -> Result<TuiStyle, SynTuiError> {
    Ok(TuiStyle {
        fg: translate_colour(syntect_style.foreground),
        bg: translate_colour(syntect_style.background),
        add_modifier: translate_font_style(syntect_style.font_style)?,
        sub_modifier: Modifier::empty(),
    })
}

fn translate_colour(syntect_color: SyntectColour) -> Option<TuiColour> {
    match syntect_color {
        SyntectColour { r, g, b, a } if a > 0 => Some(TuiColour::Rgb(r, g, b)),
        _ => None,
    }
}

fn translate_font_style(syntect_font_style: FontStyle) -> Result<Modifier, SynTuiError> {
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
        unknown => Err(SynTuiError::UnknownFontStyle {
            bits: unknown.bits(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::LinesWithEndings;

    use super::*;

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
            style: TuiStyle {
                fg: Some(TuiColour::Rgb(r, g, b)),
                bg: Some(TuiColour::Rgb(g, b, r)),
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
        let expected = Ok(TuiStyle::default()
            .fg(TuiColour::Rgb(r, g, b))
            .bg(TuiColour::Rgb(g, b, r))
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
        let expected = Err(SynTuiError::UnknownFontStyle { bits: 254 });
        let actual = translate_style(input);
        assert_eq!(expected, actual);
    }

    #[rstest]
    #[case::with_alpha(
        fake_syntect_colour(012, 123, 234, 128),
        Some(TuiColour::Rgb(012, 123, 234))
    )]
    #[case::without_alpha(fake_syntect_colour(012, 123, 234, 0), None)]
    fn check_translate_colour(#[case] input: SyntectColour, #[case] expected: Option<TuiColour>) {
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
        Err(SynTuiError::UnknownFontStyle { bits: 254 })
    )]
    fn check_translate_font_style(
        #[case] input: FontStyle,
        #[case] expected: Result<Modifier, SynTuiError>,
    ) {
        let actual = translate_font_style(input);
        assert_eq!(expected, actual);
    }

    //#[test]
    //fn sandbox() {
    //    let ps = SyntaxSet::load_defaults_newlines();
    //    let ts = ThemeSet::load_defaults();

    //    let syntax = ps.find_syntax_by_extension("rs").unwrap();
    //    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    //    let s = "pub struct Wow { hi: u64 }\nfn blah() -> u64 {}";
    //    for line in LinesWithEndings::from(s) {
    //        // LinesWithEndings enables use of newlines mode
    //        let ranges: Vec<(SyntectStyle, &str)> = h.highlight_line(line, &ps).unwrap();
    //        println!("{:#?}", ranges);
    //    }
    //    assert!(false);
    //}
}
