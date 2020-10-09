use crate::page::Renderer;
use crate::utils::{px, AttributeSet, StyleSet};
use onenote::{Outline, OutlineElement, OutlineItem};

impl<'a> Renderer<'a> {
    pub(crate) fn render_outline(&mut self, outline: &Outline) -> String {
        let mut styles = StyleSet::new();
        let mut contents = String::new();

        styles.set("margin-left", px(outline.items_level() as f32 * 0.75));

        if outline.is_layout_size_set_by_user() {
            if let Some(width) = outline.layout_max_width() {
                styles.set("max-width", px(width));
            };
        } else {
            styles.set("max-width", px(12.0));
        }

        if outline.offset_horizontal().is_some() || outline.offset_vertical().is_some() {
            styles.set("position", "absolute".to_string());
        }

        if let Some(offset) = outline.offset_horizontal() {
            styles.set("left", px(offset));
        }

        if let Some(offset) = outline.offset_vertical() {
            styles.set("top", px(offset));
        }

        if styles.len() > 0 {
            contents.push_str(&format!("<div style=\"{}\">", styles.to_string()))
        } else {
            contents.push_str("<div>");
        }

        contents.push_str(&self.render_outline_items(outline.items()));
        contents.push_str("</div>");

        contents
    }

    pub(crate) fn render_outline_items(&mut self, items: &[OutlineItem]) -> String {
        self.render_list(flatten_outline_items(items))
    }

    pub(crate) fn render_outline_element(&mut self, element: &OutlineElement) -> String {
        let mut contents = String::new();
        let is_list = self.is_list(element);

        if is_list {
            contents.push_str("<li>");
        }

        self.in_list = is_list;

        contents.extend(
            element
                .contents()
                .iter()
                .map(|content| self.render_content(content)),
        );

        self.in_list = false;

        if !element.children().is_empty() {
            let mut styles = StyleSet::new();
            styles.set("margin-left", px(0.75));

            let mut attrs = AttributeSet::new();
            attrs.set("style", styles.to_string());

            contents.push_str(&format!("<div {}>", attrs.to_string()));

            contents.push_str(&self.render_outline_items(element.children()));

            contents.push_str("</div>");
        }

        if is_list {
            contents.push_str("</li>");
        }

        contents.push('\n');

        contents
    }
}

fn flatten_outline_items<'a>(
    items: &'a [OutlineItem],
) -> Box<dyn Iterator<Item = &'a OutlineElement> + 'a> {
    Box::new(items.iter().flat_map(move |item| match item {
        OutlineItem::Element(element) => Box::new(Some(element).into_iter()),
        OutlineItem::Group(group) => flatten_outline_items(group.outlines()),
    }))
}
