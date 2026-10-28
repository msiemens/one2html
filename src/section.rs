use crate::utils::{sanitize_output_filename, sanitize_path};
use crate::{Options, page, templates};
use color_eyre::eyre::Result;
use onenote_parser::FileSystem;
use onenote_parser::section::Section;
use std::collections::HashSet;
use typed_path::{TypedPath, TypedPathBuf};

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

    pub fn render(
        &mut self,
        section: &Section,
        output_dir: TypedPath,
        options: Options,
        fs: impl FileSystem,
    ) -> Result<TypedPathBuf> {
        let section_dir = output_dir.join(sanitize_path(section.display_name(), fs)?);

        fs.make_dir(section_dir.to_path())?;

        let mut entries: Vec<templates::section::Entry> = Vec::new();
        let mut fallback_title_index = 0;

        for page_series in section.page_series() {
            for page in page_series.pages() {
                let title = page
                    .title_text()
                    .map(|s| s.to_string())
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| {
                        fallback_title_index += 1;

                        format!("Untitled Page {}", fallback_title_index)
                    });

                let file_name = title.trim().replace("/", "_");
                let file_name = self.determine_page_filename(&file_name, fs)?;
                let file_name = sanitize_path(&(file_name + ".html"), fs)?;

                let output_file = section_dir.join(file_name);

                let mut renderer = page::Renderer::new(section_dir.clone(), self, options, fs);
                let output = renderer.render_page(page)?;

                fs.write_file(output_file.to_path(), output.as_bytes())?;

                entries.push(templates::section::Entry {
                    name: title,
                    path: output_file
                        .strip_prefix(output_dir)?
                        .to_string_lossy()
                        .to_string(),
                    level: page.level(),
                    is_warnings: false,
                });
            }
        }

        let warnings = section.report().warnings();
        if options.warnings && !warnings.is_empty() {
            let stem = self.determine_page_filename("Warnings", fs)?;
            let filename = sanitize_path(&(stem + ".html"), fs)?;
            let warnings_path = section_dir.join(&filename);

            let warning_entries: Vec<templates::warnings::Entry> = warnings
                .iter()
                .map(|w| templates::warnings::Entry {
                    page: w
                        .page()
                        .map(|(_, title)| title)
                        .unwrap_or("(section-level)"),
                    message: w.message(),
                })
                .collect();

            let html = templates::warnings::render(section.display_name(), &warning_entries)?;
            fs.write_file(warnings_path.to_path(), html.as_bytes())?;

            entries.push(templates::section::Entry {
                name: "\u{26A0} Conversion Warnings".to_string(),
                path: warnings_path
                    .strip_prefix(output_dir)?
                    .to_string_lossy()
                    .to_string(),
                level: 0,
                is_warnings: true,
            });
        }

        let toc_html = templates::section::render(section.display_name(), &entries)?;
        let toc_name = sanitize_output_filename(section.display_name(), fs)? + ".html";
        let toc_file = output_dir.join(toc_name);

        fs.write_file(toc_file.to_path(), toc_html.as_bytes())?;

        Ok(section_dir)
    }

    pub(crate) fn determine_page_filename(
        &mut self,
        filename: &str,
        fs: impl FileSystem,
    ) -> Result<String> {
        let sanitized = sanitize_path(filename, fs)?;
        let mut current_filename = sanitized.clone();
        let mut i = 0;

        loop {
            if !self.pages.contains(&current_filename) {
                self.pages.insert(current_filename.clone());

                return Ok(current_filename);
            }

            i += 1;

            current_filename = format!("{}_{}", sanitized, i);
        }
    }
}
