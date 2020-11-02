use crate::page::Renderer;
use crate::utils::{px, AttributeSet, StyleSet};
use color_eyre::eyre::WrapErr;
use color_eyre::Result;
use onenote_parser::contents::Image;
use std::fs;

impl<'a> Renderer<'a> {
    pub(crate) fn render_image(&mut self, image: &Image) -> Result<String> {
        let mut content = String::new();

        if let Some(data) = image.data() {
            let filename = self.determine_image_filename(image)?;
            fs::write(self.output.join(filename.clone()), data)
                .wrap_err("Failed to write image")?;

            let mut attrs = AttributeSet::new();
            let mut styles = StyleSet::new();

            attrs.set("src", filename);

            if let Some(text) = image.alt_text() {
                attrs.set("alt", text.to_string().replace('"', "&quot;"));
            }

            if let Some(width) = image.layout_max_width() {
                styles.set("max-width", px(width));
            }

            if let Some(height) = image.layout_max_height() {
                styles.set("max-height", px(height));
            }

            if styles.len() > 0 {
                attrs.set("style", styles.to_string());
            }

            content.push_str(&format!("<img {} />", attrs.to_string()));
        }

        Ok(self.render_with_note_tags(image.note_tags(), content))
    }

    fn determine_image_filename(&mut self, image: &Image) -> Result<String> {
        if let Some(name) = image.image_filename() {
            return self.determine_filename(name);
        }

        if let Some(ext) = image.extension() {
            let mut i = 0;

            loop {
                let filename = format!("image{}{}", i, ext);

                if !self.section.files.contains(&filename) {
                    self.section.files.insert(filename.clone());

                    return Ok(filename);
                }

                i += 1;
            }
        }

        unimplemented!()
    }
}
