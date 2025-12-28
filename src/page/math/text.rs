use color_eyre::Result;
use color_eyre::eyre::eyre;
use finl_unicode::categories::CharacterCategories;
use log::warn;
use phf::phf_map;

static RANGES: phf::Map<&'static str, (char, char)> = phf_map! {
    "bold" => ('\u{1d400}', '\u{1d433}'),
    "italic" => ('\u{1d434}', '\u{1d467}'),
    "bold-italic" => ('\u{1d468}', '\u{1d49b}'),
    "script" => ('\u{1d49c}', '\u{1d4cf}'),
    "bold-script" => ('\u{1d4d0}', '\u{1d503}'),
    "fraktur" => ('\u{1d504}', '\u{1d537}'),
    "double" => ('\u{1d538}', '\u{1d56b}'),
    "fraktur-bold" => ('\u{1d56c}', '\u{1d59f}'),
    "sans" => ('\u{1d5a0}', '\u{1d5d3}'),
    "sans-bold" => ('\u{1d5d4}', '\u{1d607}'),
    "sans-italic" => ('\u{1d608}', '\u{1d63b}'),
    "sans-bold-italic" => ('\u{1d63c}', '\u{1d66f}'),
    "monospace" => ('\u{1d670}', '\u{1d6a3}'),
    "greek-bold" => ('\u{1d6a8}', '\u{1d6e1}'),
    "greek-italic" => ('\u{1d6e2}', '\u{1d71b}'),
    "greek-bold-italic" => ('\u{1d71c}', '\u{1d755}'),
    "greek-sans-bold" => ('\u{1d756}', '\u{1d78f}'),
    "greek-sans-bold-italic" => ('\u{1d790}', '\u{17c9}'),
    "operator" => ('\u{2200}', '\u{22ff}'),
    "arrow" => ('\u{2190}', '\u{21ff}'),
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub(crate) enum TextType {
    Bold,
    BoldItalic,
    BoldScript,
    Double,
    // DoubleItalic,
    DoubleOperator,
    Fraktur,
    FrakturBold,
    Identifier,
    // Italic,
    Mono,
    Normal,
    Numeric,
    Operator,
    Raw,
    Sans,
    SansBold,
    SansBoldItalic,
    SansItalic,
    Script,
    Space,
}

impl TextType {
    pub(crate) fn from_char(c: &char) -> Result<TextType> {
        if *c == '&' {
            return Ok(TextType::Raw);
        }

        if c.is_ascii_punctuation() {
            return Ok(TextType::Operator);
        }

        if c.is_ascii_alphabetic() {
            return Ok(TextType::Normal);
        }

        if c.is_separator_space() {
            return Ok(TextType::Space);
        }

        if c.is_numeric() {
            return Ok(TextType::Numeric);
        }

        if c.is_symbol_math() {
            return Ok(TextType::Operator);
        }

        if c.is_format() {
            if *c == '\u{2061}' {
                return Ok(TextType::Operator);
            }

            return Ok(TextType::Raw);
        }

        if *c == 'ⅅ' || *c == 'ⅆ' {
            return Ok(TextType::DoubleOperator);
        }

        for (name, range) in RANGES.entries() {
            if *c >= range.0 && *c <= range.1 {
                return TextType::from_name(name);
            }
        }

        let unicode_block = unicode_blocks::find_unicode_block(*c);

        if let Some(block) = unicode_block {
            if block == unicode_blocks::GREEK_AND_COPTIC
                || block == unicode_blocks::LETTERLIKE_SYMBOLS
                || block == unicode_blocks::MATHEMATICAL_ALPHANUMERIC_SYMBOLS
            {
                return Ok(TextType::Identifier);
            }

            if block == unicode_blocks::LATIN_1_SUPPLEMENT {
                return Ok(TextType::Normal);
            }

            if block == unicode_blocks::GENERAL_PUNCTUATION {
                return Ok(TextType::Operator);
            }
        }

        warn!(
            "Math feature not implemented: unknown text classification for {}. Please provide a sample at https://github.com/msiemens/one2html/issues.",
            c
        );
        Ok(TextType::Identifier)
    }

    fn from_name(name: &str) -> Result<TextType> {
        let text_type = match name {
            "operator" => TextType::Operator,
            "arrow" => TextType::Operator,
            "bold" => TextType::Bold,
            "italic" => TextType::Identifier,
            "bold-italic" => TextType::BoldItalic,
            "script" => TextType::Script,
            "bold-script" => TextType::BoldScript,
            "fraktur" => TextType::Fraktur,
            "double" => TextType::Double,
            "fraktur-bold" => TextType::FrakturBold,
            "sans" => TextType::Sans,
            "sans-bold" => TextType::SansBold,
            "sans-italic" => TextType::SansItalic,
            "sans-bold-italic" => TextType::SansBoldItalic,
            "monospace" => TextType::Mono,
            "greek-bold" => TextType::Bold,
            "greek-italic" => TextType::Identifier,
            "greek-bold-italic" => TextType::BoldItalic,
            "greek-sans-bold" => TextType::SansBold,
            "greek-sans-bold-italic" => TextType::SansBoldItalic,
            _ => return Err(eyre!("Unsupported TextType: {}", name)),
        };

        Ok(text_type)
    }
}
