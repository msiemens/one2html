use crate::page::Renderer;
use color_eyre::Result;
use console::style;
use onenote_parser::contents::Content;

impl<'a> Renderer<'a> {
    pub(crate) fn render_content(&mut self, content: &Content) -> Result<String> {
        match content {
            Content::RichText(text) => self.render_rich_text(text),
            Content::Image(image) => self.render_image(image),
            Content::EmbeddedFile(file) => self.render_embedded_file(file),
            Content::Table(table) => self.render_table(table),
            Content::Unknown => {
                eprintln!("{} Page with unknown content", style("Warning:").yellow());
                Ok(String::new())
            }
        }
    }
}
