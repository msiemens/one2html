use crate::page::Renderer;
use crate::utils::{px, AttributeSet, StyleSet};
use onenote::{Table, TableCell};

impl<'a> Renderer<'a> {
    pub(crate) fn render_table(&mut self, table: &Table) -> String {
        let mut has_note_tags = false;

        let mut content = String::new();
        if let Some((markup, styles)) = self.render_note_tags(image.note_tags()) {
            content.push_str(&format!("<div style=\"{}\">{}", styles, markup));

            has_note_tags = true;
        }

        let mut styles = StyleSet::new();
        styles.set("border-collapse", "collapse".to_string());

        if table.borders_visible() {
            styles.set("border", "1pt solid #A3A3A3".to_string());
        }

        let mut attributes = AttributeSet::new();
        attributes.set("style", styles.to_string());
        attributes.set("cellspacing", "0".to_string());
        attributes.set("cellpadding", "0".to_string());

        if table.borders_visible() {
            attributes.set("border", "1".to_string());
        }

        content.push_str(&format!("<table {}>", attributes.to_string()));

        let locked_cols = calc_locked_cols(table.cols_locked(), table.cols());

        for row in table.contents() {
            content.push_str("<tr>");

            assert_eq!(row.contents().len(), table.col_widths().len());

            let cells = row
                .contents()
                .iter()
                .zip(table.col_widths().iter().copied())
                .zip(locked_cols.iter().copied())
                .map(|((cell, width), locked)| {
                    if locked {
                        (cell, Some(width))
                    } else {
                        (cell, None)
                    }
                });

            for (cell, width) in cells {
                self.render_table_cell(&mut content, cell, width);
            }

            content.push_str("</tr>");
        }

        content.push_str("</table>");

        if has_note_tags {
            content.push_str("</div>");
        }

        content
    }

    fn render_table_cell(&mut self, contents: &mut String, cell: &TableCell, width: Option<f32>) {
        let mut styles = StyleSet::new();
        styles.set("padding", "2pt".to_string());
        styles.set("vertical-align", "top".to_string());
        styles.set("min-width", px(1.0));

        if let Some(width) = width {
            styles.set("width", px(width));
        }

        if let Some(color) = cell.background_color() {
            styles.set(
                "background",
                format!("rgb({}, {}, {})", color.r(), color.g(), color.b()),
            )
        }

        let mut attrs = AttributeSet::new();
        attrs.set("style", styles.to_string());

        contents.push_str(&format!("<td {}>", attrs.to_string()));

        let mut in_list = false;

        for element in cell.contents() {
            if !in_list && self.is_list(element) {
                contents.push_str("<ul style=\"margin-left: 12px;\">");
                in_list = true;
            }

            if in_list && !self.is_list(element) {
                contents.push_str("</ul>\n");
                in_list = false;
            }

            contents.push_str(&self.render_outline_element(element));
        }

        contents.push_str("</td>");
    }
}

fn calc_locked_cols(data: &[u8], count: u32) -> Vec<bool> {
    if data.is_empty() {
        return vec![false; count as usize];
    }

    (0..count)
        .map(|i| data[i as usize / 8] & (1 << (i % 8)) == 1)
        .collect()
}
