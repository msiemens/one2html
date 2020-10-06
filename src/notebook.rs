use crate::{section, templates};
use onenote::{Color, Notebook};
use palette::rgb::Rgb;
use palette::{Alpha, ConvertFrom, Hsl, Saturate, Shade, Srgb};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub(crate) type RgbColor = Alpha<Rgb<palette::encoding::Srgb, u8>, f32>;

pub(crate) struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(
        &mut self,
        notebook: &Notebook,
        name: &str,
        output_dir: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        if !output_dir.is_dir() {
            fs::create_dir(&output_dir)?;
        }

        let notebook_dir = output_dir.join(name);

        if !notebook_dir.is_dir() {
            fs::create_dir(&notebook_dir)?;
        }

        let mut toc = Vec::new();

        for section in notebook.sections() {
            let mut renderer = section::Renderer::new();
            let path = renderer.render(section, notebook_dir.to_path_buf())?;

            toc.push(templates::notebook::Section {
                name: section.display_name().to_string(),
                path: path
                    .components()
                    .skip(1)
                    .collect::<PathBuf>()
                    .to_string_lossy()
                    .to_string(),
                color: section.color().map(prepare_color),
            });
        }

        let toc_html = templates::notebook::render(name, &toc);
        let toc_file = output_dir.join(format!("{}.html", name));
        fs::write(toc_file, toc_html)?;

        Ok(())
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
