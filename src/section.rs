use crate::{page, templates};
use color_eyre::eyre::Result;
use onenote_parser::section::Section;
use std::cmp::min;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct Renderer {
    pub(crate) files: HashSet<String>,
    pub(crate) pages: HashSet<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            files: Default::default(),
            pages: Default::default(),
        }
    }

    pub fn render(&mut self, section: &Section, output_dir: &Path) -> Result<PathBuf> {
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
                let file_name = self.determine_page_filename(&file_name)?;

                let output_file = section_dir.join(file_name + ".html");

                let mut renderer = page::Renderer::new(section_dir.clone(), self);
                let output = renderer.render_page(page)?;

                fs::write(&output_file, output)?;

                toc.push((
                    title,
                    output_file
                        .strip_prefix(&output_dir)?
                        .to_string_lossy()
                        .to_string(),
                    page.level(),
                ))
            }
        }

        let toc_html = templates::section::render(section.display_name(), toc)?;
        let toc_file = output_dir.join(format!("{}.html", section.display_name()));
        fs::write(toc_file, toc_html)?;

        Ok(section_dir)
    }

    pub(crate) fn determine_page_filename(&mut self, filename: &str) -> Result<String> {
        let mut i = 0;
        let mut current_filename = filename.to_string();

        loop {
            if !self.pages.contains(&current_filename) {
                self.pages.insert(current_filename.clone());

                return Ok(current_filename);
            }

            i += 1;

            current_filename = format!("{}_{}", filename, i);
        }
    }
}
