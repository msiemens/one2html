use crate::page::Renderer;
use onenote::Content;

impl<'a> Renderer<'a> {
    pub(crate) fn render_content(&mut self, content: &Content) -> String {
        match content {
            Content::RichText(text) => self.render_rich_text(text),
            Content::Image(image) => self.render_image(image),
            Content::EmbeddedFile(file) => self.render_embedded_file(file),
            Content::Table(table) => self.render_table(table),
            Content::Unknown => String::new(),
        }
    }
}
