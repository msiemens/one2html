//! Plain-Markdown rendering, as an alternative to the HTML site rendered by
//! `section::Renderer`/`notebook::Renderer`.
//!
//! Intended for text-mode previewing (e.g. a Midnight Commander F3 viewer)
//! rather than as a full-fidelity export: formatting is reduced to bold/
//! italic/strikethrough and hyperlinks, images/embedded files/ink become a
//! placeholder line instead of being extracted to disk, and outline nesting
//! becomes indentation. Real page structure (page order, section/page
//! nesting, table layout, hyperlink targets) is preserved from the parsed
//! object model, not re-guessed from text.

use color_eyre::Result;
use color_eyre::eyre::eyre;
use onenote_parser::contents::{
    Content, List, NoteTag, Outline, OutlineElement, OutlineItem, RichText, Table,
};
use onenote_parser::notebook::Notebook;
use onenote_parser::page::{Page, PageContent};
use onenote_parser::property::rich_text::ParagraphStyling;
use onenote_parser::section::{Section, SectionEntry};
use std::iter::repeat;

const FORMAT_NUMBERED_LIST: char = '\u{fffd}';
const HYPERLINK_MARKER: &str = "\u{fddf}HYPERLINK \"";

pub fn render_notebook(notebook: &Notebook, name: &str) -> Result<String> {
    let mut out = String::new();
    out.push_str(&heading(1, name));

    for entry in notebook.entries() {
        render_entry(entry, 2, &mut out)?;
    }

    Ok(out)
}

pub fn render_section_standalone(section: &Section) -> Result<String> {
    let mut out = String::new();
    out.push_str(&heading(1, section.display_name()));
    render_section_pages(section, 2, &mut out)?;
    Ok(out)
}

fn render_entry(entry: &SectionEntry, depth: usize, out: &mut String) -> Result<()> {
    match entry {
        SectionEntry::Section(section) => {
            out.push_str(&heading(depth, section.display_name()));
            render_section_pages(section, depth + 1, out)?;
        }
        SectionEntry::SectionGroup(group) => {
            out.push_str(&heading(depth, group.display_name()));
            for child in group.entries() {
                render_entry(child, depth + 1, out)?;
            }
        }
    }
    Ok(())
}

fn render_section_pages(section: &Section, depth: usize, out: &mut String) -> Result<()> {
    for series in section.page_series() {
        for page in series.pages() {
            render_page(page, depth, out)?;
        }
    }
    Ok(())
}

fn render_page(page: &Page, depth: usize, out: &mut String) -> Result<()> {
    let title = page.title_text().unwrap_or("Untitled Page");
    let page_depth = depth + (page.level().max(0) as usize);
    out.push_str(&heading(page_depth, title));

    render_page_contents(page.contents(), out)?;
    out.push('\n');
    Ok(())
}

fn heading(depth: usize, text: &str) -> String {
    let level = depth.clamp(1, 6);
    let text = text.trim();
    let text = if text.is_empty() { "Untitled" } else { text };
    format!("{} {}\n\n", "#".repeat(level), text)
}

fn indent(depth: usize) -> String {
    "    ".repeat(depth)
}

fn render_page_contents(contents: &[PageContent], out: &mut String) -> Result<()> {
    for content in contents {
        match content {
            PageContent::Outline(outline) => render_outline(outline, 0, out)?,
            PageContent::Image(image) => {
                out.push_str(&image_placeholder(image.alt_text()));
                out.push_str("\n\n");
            }
            PageContent::EmbeddedFile(file) => {
                out.push_str(&embedded_placeholder(file.filename()));
                out.push_str("\n\n");
            }
            PageContent::Ink(_) | PageContent::Unknown => {}
        }
    }
    Ok(())
}

fn render_outline(outline: &Outline, depth: usize, out: &mut String) -> Result<()> {
    render_outline_items(outline.items(), depth, out)
}

fn render_outline_items(items: &[OutlineItem], depth: usize, out: &mut String) -> Result<()> {
    // Numbering restarts at each nesting level and at each group boundary --
    // a simplification of the real per-list restart/continuation rules,
    // which aren't meaningful in a plain-text preview anyway.
    let mut number = 1u32;
    for item in items {
        match item {
            OutlineItem::Element(element) => {
                render_outline_element(element, depth, out, &mut number)?;
            }
            OutlineItem::Group(group) => {
                render_outline_items(group.outlines(), depth, out)?;
            }
        }
    }
    Ok(())
}

fn render_outline_element(
    element: &OutlineElement,
    depth: usize,
    out: &mut String,
    number: &mut u32,
) -> Result<()> {
    let is_numbered = element
        .list_contents()
        .first()
        .map(is_numbered_list)
        .unwrap_or(false);

    let marker = if is_numbered {
        let n = *number;
        *number += 1;
        format!("{}. ", n)
    } else {
        "- ".to_string()
    };

    let tag_prefix = note_tag_prefix(element.contents());
    let lines = render_element_contents(element.contents())?;
    let is_explicit_list_item = !element.list_contents().is_empty();
    let all_blank = lines.iter().all(|l| l.trim().is_empty());

    if all_blank && !is_explicit_list_item && tag_prefix.is_empty() {
        // A plain empty paragraph is source spacing between real bullets,
        // not a real (numbered/bulleted) list item -- render it as
        // whitespace rather than a bare "- " marker.
        out.push('\n');
    } else if lines.is_empty() {
        out.push_str(&format!("{}{}\n", indent(depth), marker.trim_end()));
    } else {
        out.push_str(&format!(
            "{}{}{}{}\n",
            indent(depth),
            marker,
            tag_prefix,
            lines[0]
        ));
        for line in &lines[1..] {
            out.push_str(&format!("{}{}\n", indent(depth + 1), line));
        }
    }

    let children = element.children();
    if !children.is_empty() {
        render_outline_items(children, depth + 1, out)?;
    }

    Ok(())
}

fn is_numbered_list(list: &List) -> bool {
    list.list_format().first() == Some(&FORMAT_NUMBERED_LIST)
}

/// A leading `**[Label]** ` (or `**[x] Label:** ` for a completed action
/// item) taken from the first note tag found on this outline element's rich
/// text, if any.
fn note_tag_prefix(contents: &[Content]) -> String {
    for content in contents {
        let Content::RichText(text) = content else {
            continue;
        };
        for tag in text.note_tags() {
            if let Some(prefix) = format_note_tag(tag) {
                return prefix;
            }
        }
    }
    String::new()
}

fn format_note_tag(tag: &NoteTag) -> Option<String> {
    let def = tag.definition()?;
    let label = def.label();
    if label.is_empty() {
        return None;
    }
    if tag.item_status().completed() {
        Some(format!("**[x] {}:** ", label))
    } else {
        Some(format!("**[ ] {}:** ", label))
    }
}

/// One rendered "line" per `Content` entry -- most commonly a single
/// RichText paragraph, but a Table renders as a multi-line block and
/// Image/EmbeddedFile become a placeholder line.
fn render_element_contents(contents: &[Content]) -> Result<Vec<String>> {
    let mut lines = Vec::new();
    for content in contents {
        match content {
            Content::RichText(text) => lines.push(render_rich_text(text)?),
            Content::Table(table) => lines.push(render_table(table)?),
            Content::Image(image) => lines.push(image_placeholder(image.alt_text())),
            Content::EmbeddedFile(file) => lines.push(embedded_placeholder(file.filename())),
            Content::Ink(_) | Content::Unknown => {}
        }
    }
    Ok(lines)
}

fn render_rich_text(text: &RichText) -> Result<String> {
    if !text.embedded_objects().is_empty() {
        // Ink / math-space / line-break embedded objects have no plain-text
        // representation worth emitting here.
        return Ok(String::new());
    }

    let indices = text.text_run_indices();
    let styles = text.text_run_formatting();
    let full_text = text.text().to_string();

    let parts = if !indices.is_empty() {
        split_by_indices(indices, full_text)?
    } else {
        vec![full_text]
    };

    let rendered = render_text_run_styles(styles, parts).join("");
    Ok(fix_newlines(rendered))
}

fn split_by_indices(indices: &[u32], text: String) -> Result<Vec<String>> {
    let mut parts = vec![];
    let mut text = text.encode_utf16().collect::<Vec<u16>>();

    for i in indices.iter().copied().rev() {
        let i = (i as usize).min(text.len());
        let part = text.split_off(i);
        parts.push(part);
    }

    if !indices.is_empty() {
        parts.push(text);
    }

    parts
        .into_iter()
        .map(|t| String::from_utf16(&t).map_err(|e| eyre!("Failed to parse rich text contents: {e}")))
        .collect()
}

/// See onenote.rs `page/rich_text.rs`'s `render_text_run_styles` for the
/// full explanation of the hidden-marker-run + display-run hyperlink
/// encoding this mirrors (simplified: markdown link syntax instead of
/// `<a href>`, `**`/`*`/`~~` instead of CSS, everything else dropped).
fn render_text_run_styles(styles: &[ParagraphStyling], parts: Vec<String>) -> Vec<String> {
    let mut pending_marker = String::new();

    parts
        .into_iter()
        .rev()
        .zip(styles.iter().map(Some).chain(repeat(None)))
        .map(|(text, style)| -> String {
            let style = match style {
                Some(style) => style,
                None => return text,
            };

            if style.hidden() {
                pending_marker.push_str(&text);
                return String::new();
            }

            let pending_url = extract_hyperlink_url(&pending_marker);
            pending_marker.clear();

            if style.hyperlink_protected() {
                return match pending_url {
                    Some(url) if url != text.trim() => format!("[{}]({})", text.trim(), url),
                    Some(url) => url,
                    None => text,
                };
            }

            if !style.math_formatting()
                && (text.starts_with("http://") || text.starts_with("https://"))
            {
                return text;
            }

            apply_emphasis(style, text)
        })
        .collect()
}

fn extract_hyperlink_url(text: &str) -> Option<String> {
    text.strip_prefix(HYPERLINK_MARKER)
        .and_then(|s| s.strip_suffix('"'))
        .map(str::to_owned)
}

fn apply_emphasis(style: &ParagraphStyling, text: String) -> String {
    if text.trim().is_empty() || style.math_formatting() {
        return text;
    }
    let mut text = text;
    if style.strikethrough() {
        text = format!("~~{}~~", text);
    }
    if style.italic() {
        text = format!("*{}*", text);
    }
    if style.bold() {
        text = format!("**{}**", text);
    }
    text
}

/// Keep embedded newlines (e.g. from OCR "recognized text" on a pasted
/// screenshot) inside the same list item as a Markdown hard line break,
/// rather than letting a bare `\n` terminate the surrounding list.
fn fix_newlines(text: String) -> String {
    text.replace('\u{000b}', "  \n")
        .replace("\r\n", "  \n")
        .replace(['\n', '\r'], "  \n")
}

fn render_table(table: &Table) -> Result<String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in table.contents() {
        let mut cells = Vec::new();
        for cell in row.contents() {
            let mut cell_lines = Vec::new();
            for element in cell.contents() {
                cell_lines.extend(render_element_contents(element.contents())?);
            }
            let cell_text = cell_lines.join("; ").replace('|', "\\|").replace('\n', " ");
            cells.push(if cell_text.trim().is_empty() {
                " ".to_string()
            } else {
                cell_text
            });
        }
        rows.push(cells);
    }

    if rows.is_empty() {
        return Ok(String::new());
    }

    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut out = String::new();
    for (i, row) in rows.iter().enumerate() {
        let mut cells = row.clone();
        cells.resize(col_count, String::new());
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
        if i == 0 {
            out.push('|');
            out.push_str(&"---|".repeat(col_count));
            out.push('\n');
        }
    }
    Ok(out.trim_end().to_string())
}

fn image_placeholder(alt: Option<&str>) -> String {
    match alt {
        Some(text) if !text.trim().is_empty() => {
            format!("*[Image: {}]*", text.trim().replace('\n', " "))
        }
        _ => "*[Image]*".to_string(),
    }
}

fn embedded_placeholder(filename: &str) -> String {
    format!("*[Embedded file: {}]*", filename)
}
