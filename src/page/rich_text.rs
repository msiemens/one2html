use crate::page::Renderer;
use crate::utils::{AttributeSet, StyleSet, px};
use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::WrapErr;
use itertools::Itertools;
use log::warn;
use once_cell::sync::Lazy;
use onenote_parser::contents::{EmbeddedObject, MathInlineObject, RichText};
use onenote_parser::property::common::ColorRef;
use onenote_parser::property::rich_text::{ParagraphAlignment, ParagraphStyling};
use regex::{Captures, Regex};
use std::iter::repeat;

impl<'a> Renderer<'a> {
    pub(crate) fn render_rich_text(&mut self, text: &RichText) -> Result<String> {
        let mut content = String::new();
        let mut attrs = AttributeSet::new();
        let mut style = self.parse_paragraph_styles(text);

        if let Some((note_tag_html, note_tag_styles)) = self.render_note_tags(text.note_tags()) {
            content.push_str(&note_tag_html);
            style.extend(note_tag_styles);
        }

        content.push_str(&self.parse_content(text)?);

        if content.starts_with("http://") || content.starts_with("https://") {
            content = format!("<a href=\"{}\">{}</a>", content, content);
        }

        if style.len() > 0 {
            attrs.set("style", style.to_string());
        }

        match text.paragraph_style().style_id() {
            Some(t) if !self.in_list && is_tag(t) => {
                Ok(format!("<{} {}>{}</{}>", t, attrs, content, t))
            }
            _ if style.len() > 0 => Ok(format!("<span style=\"{}\">{}</span>", style, content)),
            _ => Ok(content),
        }
    }

    fn parse_content(&mut self, data: &RichText) -> Result<String> {
        if !data.embedded_objects().is_empty() {
            return Ok(data
                .embedded_objects()
                .iter()
                .map(|object| match object {
                    EmbeddedObject::Ink(container) => {
                        self.render_ink(container.ink(), container.bounding_box(), true)
                    }
                    EmbeddedObject::InkSpace(space) => {
                        format!("<span class=\"ink-space\" style=\"padding-left: {}; padding-top: {};\"></span>",
                                px(space.width()), px(space.height()))
                    }
                    EmbeddedObject::InkLineBreak => {
                        "<span class=\"ink-linebreak\"><br></span>".to_string()
                    }
                })
                .collect_vec()
                .join(""));
        }

        let indices = data.text_run_indices();
        let styles = data.text_run_formatting();

        if indices.len() > styles.len() {
            warn!(
                "Some text runs have no corresponding styles: {:?} vs {:?}",
                indices, styles
            );
        }

        let mut text = data.text().to_string();

        if text.is_empty() {
            text = "&nbsp;".to_string();
        }

        let parts = if !indices.is_empty() {
            self.split_by_indices(indices, text)?
        } else {
            vec![text]
        };

        // Render text run styles
        let content = self.render_text_run_styles(styles, parts)?;

        // Render math groups
        let content = self.render_math_text_runs(data, styles, content)?;

        Ok(fix_newlines(content))
    }

    fn render_math_text_runs(
        &mut self,
        data: &RichText,
        styles: &[ParagraphStyling],
        content: Vec<String>,
    ) -> Result<String> {
        let math_groups = content
            .into_iter()
            .zip(styles.iter())
            .chunk_by(|(_text, style)| style.math_formatting());

        let mut math_object_offset = 0;

        let contents = (&math_groups)
            .into_iter()
            .map(|(is_math, group)| {
                let group_parts = group.collect_vec();
                let text = group_parts.iter().map(|(text, _)| text).join("");

                if !is_math {
                    return Ok(text);
                }

                let inline_objects = data.math_inline_objects();

                if math_object_offset >= inline_objects.len() {
                    let segment = (text, MathInlineObject::default());
                    return self.render_math(vec![segment]);
                }

                let count = group_parts.len();
                let objects = inline_objects[math_object_offset..math_object_offset + count]
                    .iter()
                    .copied();
                let segments = group_parts
                    .into_iter()
                    .map(|(text, _)| text)
                    .zip(objects)
                    .collect_vec();

                let text = self.render_math(segments)?;
                math_object_offset += count;

                Ok(text)
            })
            .collect::<Result<Vec<_>>>()?
            .join("");

        Ok(contents)
    }

    fn render_text_run_styles(
        &mut self,
        styles: &[ParagraphStyling],
        parts: Vec<String>,
    ) -> Result<Vec<String>> {
        let mut in_hyperlink = false;

        parts
            .into_iter()
            .rev()
            .zip(styles.iter().map(Some).chain(repeat(None)))
            .map(|(text, style)| {
                let style = match style {
                    Some(style) => style,
                    None => return Ok(text),
                };

                if style.hyperlink() {
                    let text = self.render_hyperlink(text, style, in_hyperlink);
                    in_hyperlink = true;

                    return text;
                }

                in_hyperlink = false;

                let style = self.parse_style(style);

                if style.len() > 0 {
                    Ok(format!("<span style=\"{}\">{}</span>", style, text))
                } else {
                    Ok(text)
                }
            })
            .collect::<Result<Vec<String>>>()
    }

    fn split_by_indices(&self, indices: &[u32], text: String) -> Result<Vec<String>> {
        // Split text into parts specified by indices
        let mut parts = vec![];

        let mut text = text.encode_utf16().collect::<Vec<u16>>();

        for i in indices.iter().copied().rev() {
            let part = text[i as usize..].to_vec();
            text = text[0..i as usize].to_vec();

            parts.push(part);
        }

        if !indices.is_empty() {
            parts.push(text);
        }

        parts
            .into_iter()
            .map(|text| String::from_utf16(&text).wrap_err("Failed to parse rich text contents"))
            .collect::<Result<Vec<_>>>()
    }

    fn render_hyperlink(
        &self,
        text: String,
        style: &ParagraphStyling,
        in_hyperlink: bool,
    ) -> Result<String> {
        const HYPERLINK_MARKER: &str = "\u{fddf}HYPERLINK \"";

        let style = self.parse_style(style);

        if text.starts_with(HYPERLINK_MARKER) {
            let url = text
                .strip_prefix(HYPERLINK_MARKER)
                .wrap_err("Hyperlink has no start marker")?
                .strip_suffix('"')
                .wrap_err("Hyperlink has no end marker")?;

            Ok(format!("<a href=\"{}\" style=\"{}\">", url, style))
        } else if in_hyperlink {
            Ok(text + "</a>")
        } else {
            Ok(format!(
                "<a href=\"{}\" style=\"{}\">{}</a>",
                text, style, text
            ))
        }
    }

    fn parse_paragraph_styles(&self, text: &RichText) -> StyleSet {
        if !text.embedded_objects().is_empty() {
            assert_eq!(
                text.text(),
                "",
                "paragraph with text and embedded objects is not supported"
            );

            return StyleSet::new();
        }

        let mut styles = self.parse_style(text.paragraph_style());

        if let [style] = text.text_run_formatting() {
            styles.extend(self.parse_style(style))
        }

        if text.paragraph_space_before() > 0.0 {
            styles.set("padding-top", px(text.paragraph_space_before()))
        }

        if text.paragraph_space_after() > 0.0 {
            styles.set("padding-bottom", px(text.paragraph_space_after()))
        }

        if let Some(line_spacing) = text.paragraph_line_spacing_exact()
            && line_spacing > 0.0
        {
            dbg!(text);
            unimplemented!();
        }

        match text.paragraph_alignment() {
            ParagraphAlignment::Center => styles.set("text-align", "center".to_string()),
            ParagraphAlignment::Right => styles.set("text-align", "right".to_string()),
            _ => {}
        }

        styles
    }

    fn parse_style(&self, style: &ParagraphStyling) -> StyleSet {
        let mut styles = StyleSet::new();

        if style.math_formatting() {
            return styles;
        }

        if style.bold() {
            styles.set("font-weight", "bold".to_string());
        }

        if style.italic() {
            styles.set("font-style", "italic".to_string());
        }

        if style.underline() {
            styles.set("text-decoration", "underline".to_string());
        }

        if style.superscript() {
            styles.set("vertical-align", "super".to_string());
        }

        if style.subscript() {
            styles.set("vertical-align", "sub".to_string());
        }

        if style.strikethrough() {
            styles.set("text-decoration", "line-through".to_string());
        }

        if let Some(font) = style.font() {
            styles.set("font-family", font.to_string());
        }

        if let Some(size) = style.font_size() {
            styles.set("font-size", ((size as f32) / 2.0).to_string() + "pt");
        }

        if let Some(ColorRef::Manual { r, g, b }) = style.font_color() {
            styles.set("color", format!("rgb({},{},{})", r, g, b));
        }

        if let Some(ColorRef::Manual { r, g, b }) = style.highlight() {
            styles.set("background-color", format!("rgb({},{},{})", r, g, b));
        }

        if style.paragraph_alignment().is_some() {
            unimplemented!()
        }

        if let Some(space) = style.paragraph_space_before()
            && space != 0.0
        {
            unimplemented!()
        }

        if let Some(space) = style.paragraph_space_after()
            && space != 0.0
        {
            unimplemented!()
        }

        if let Some(space) = style.paragraph_line_spacing_exact() {
            if space != 0.0 {
                unimplemented!()
            }

            if let Some(size) = style.font_size() {
                styles.set(
                    "line-height",
                    format!("{}px", (size as f32 * 1.2 / 72.0 * 48.0).ceil()),
                )
            }
        }

        // if style.math_formatting() {
        //     // FIXME: Handle math formatting
        //     // See https://docs.microsoft.com/en-us/windows/win32/api/richedit/ns-richedit-gettextex
        //     // for unicode chars used
        //     unimplemented!()
        // }

        styles
    }
}

fn is_tag(tag: &str) -> bool {
    !matches!(tag, "PageDateTime" | "PageTitle")
}

fn fix_newlines(text: String) -> String {
    static REGEX_LEADING_SPACES: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"<br>(\s+)").expect("failed to compile regex"));

    let text = text
        .replace("\u{000b}", "<br>")
        .replace("\n", "<br>")
        .replace("\r", "<br>");

    REGEX_LEADING_SPACES
        .replace_all(&text, |captures: &Captures| {
            "<br>".to_string() + &"&nbsp;".repeat(captures[1].len())
        })
        .to_string()
}
