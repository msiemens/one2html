use crate::templates::notebook::Toc;
use crate::{section, templates};
use color_eyre::eyre::{eyre, Result};
use onenote_parser::{Color, Notebook, NotebookEntry, Section};
use palette::rgb::Rgb;
use palette::{Alpha, ConvertFrom, Hsl, Saturate, Shade, Srgb};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) type RgbColor = Alpha<Rgb<palette::encoding::Srgb, u8>, f32>;

pub(crate) struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(&mut self, notebook: &Notebook, name: &str, output_dir: &Path) -> Result<()> {
        if !output_dir.is_dir() {
            fs::create_dir(&output_dir)?;
        }

        let notebook_dir = output_dir.join(name);

        if !notebook_dir.is_dir() {
            fs::create_dir(&notebook_dir)?;
        }

        let mut toc = Vec::new();

        for entry in notebook.entries() {
            match entry {
                NotebookEntry::Section(section) => {
                    toc.push(Toc::Section(self.render_section(
                        section,
                        &notebook_dir,
                        output_dir,
                    )?));
                }
                NotebookEntry::SectionGroup(group) => {
                    let group_dir = dbg!(notebook_dir.join(group.display_name()));
                    if !group_dir.is_dir() {
                        fs::create_dir(&group_dir)?;
                    }

                    let mut entries = Vec::new();

                    for entry in group.entries() {
                        if let NotebookEntry::Section(section) = entry {
                            entries.push(self.render_section(section, &group_dir, &output_dir)?);
                        } else {
                            return Err(eyre!("Nested section groups are not yet supported"));
                        }
                    }

                    toc.push(templates::notebook::Toc::SectionGroup(
                        group.display_name().to_string(),
                        entries,
                    ))
                }
            }
        }

        let toc_html = templates::notebook::render(name, &toc)?;
        let toc_file = output_dir.join(format!("{}.html", name));
        fs::write(toc_file, toc_html)?;

        Ok(())
    }

    fn render_section(
        &mut self,
        section: &Section,
        notebook_dir: &Path,
        base_dir: &Path,
    ) -> Result<templates::notebook::Section> {
        let mut renderer = section::Renderer::new();
        let path = renderer.render(section, dbg!(notebook_dir).to_path_buf())?;

        Ok(templates::notebook::Section {
            name: section.display_name().to_string(),
            path: path.strip_prefix(base_dir)?.to_string_lossy().to_string(),
            color: section.color().map(prepare_color),
        })
    }
}

fn prepare_color(color: Color) -> RgbColor {
    Alpha {
        alpha: color.alpha() as f32 / 255.0,
        color: Srgb::convert_from(
            Hsl::convert_from(Srgb::new(
                color.r() as f32 / 255.0,
                color.g() as f32 / 255.0,
                color.b() as f32 / 255.0,
            ))
            .darken(0.2)
            .saturate(1.0),
        )
        .into_format(),
    }
}
