use crate::{page, templates};
use onenote::Section;
use std::cmp::min;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub(crate) struct Renderer {
    pub(crate) files: HashSet<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            files: Default::default(),
        }
    }

    pub fn render(
        &mut self,
        section: &Section,
        output_dir: PathBuf,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let section_dir = output_dir.join(section.display_name());

        if !section_dir.is_dir() {
            fs::create_dir(&section_dir)?;
        }

        let mut toc = Vec::new();
        let mut fallback_title_index = 0;

        for page_series in section.page_series() {
            for page in page_series.pages() {
                let title = page.title_text().map(|s| s.to_string()).unwrap_or_else(|| {
                    fallback_title_index += 1;

                    format!("Untitled Page {}", fallback_title_index)
                });

                let file_name = (&title[0..(min(title.len(), 250))])
                    .trim()
                    .replace("/", "_");

                let output_file = section_dir.join(file_name + ".html");

                let mut renderer = page::Renderer::new(section_dir.clone(), self);
                let output = renderer.render_page(page);

                fs::write(dbg!(&output_file), output)?;

                toc.push((
                    title,
                    output_file
                        .components()
                        .skip(2)
                        .collect::<PathBuf>()
                        .to_string_lossy()
                        .to_string(),
                    page.level(),
                ))
            }
        }

        let toc_html = templates::section::render(section.display_name(), toc);
        let toc_file = output_dir.join(format!("{}.html", section.display_name()));
        fs::write(toc_file, toc_html)?;

        Ok(section_dir)
    }
}
