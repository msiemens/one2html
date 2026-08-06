use crate::page::Renderer;
use crate::utils::sanitize_output_filename;
use crate::utils::{AttributeSet, StyleSet, detect_png, px};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use onenote_parser::FileSystem;
use onenote_parser::contents::Image;
use std::io::{Cursor, Read};

impl<'a, FS: FileSystem> Renderer<'a, FS> {
    pub(crate) fn render_image(&mut self, image: &Image) -> Result<String> {
        let mut content = String::new();

        if let Some(mut reader) = image.read() {
            // Read enough of a prefix that `determine_image_filename` can run
            // file-type detection without materialising the whole image.
            let image_start_bytes = read_file_start(&mut reader)?;
            let filename = self.determine_image_filename(
                image.image_filename(),
                image.extension(),
                &image_start_bytes,
            )?;

            let target_file = self.output.join(filename.clone());

            // Re-prepend the sniffed bytes so the file we write out is complete.
            let mut reader = Cursor::new(image_start_bytes).chain(reader);
            self.fs
                .stream_to_file(target_file.to_path(), &mut reader)
                .wrap_err("Failed to write image")?;

            let mut attrs = AttributeSet::new();
            let mut styles = StyleSet::new();

            attrs.set("src", filename);

            if let Some(text) = image.alt_text() {
                attrs.set("alt", text.to_string());
            }

            if let Some(width) = image.layout_max_width() {
                styles.set("max-width", px(width));
            }

            if let Some(height) = image.layout_max_height() {
                styles.set("max-height", px(height));
            }

            if image.offset_horizontal().is_some() || image.offset_vertical().is_some() {
                styles.set("position", "absolute".to_string());
            }

            if let Some(offset) = image.offset_horizontal() {
                styles.set("left", px(offset));
            }

            if let Some(offset) = image.offset_vertical() {
                styles.set("top", px(offset));
            }

            if styles.len() > 0 {
                attrs.set("style", styles.to_string());
            }

            content.push_str(&format!("<img {} />", attrs));
        }

        Ok(self.render_with_note_tags(image.note_tags(), content))
    }

    fn determine_image_filename(
        &mut self,
        image_filename: Option<&str>,
        extension: Option<&str>,
        initial_bytes: &[u8],
    ) -> Result<String> {
        if let Some(name) = image_filename {
            // Workaround: PDF printout pages are PNG images, but have an image_filename
            // with extension .PDF. Add a PNG extension to these files so that they are
            // imported properly.
            let is_pdf = std::path::Path::new(name)
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false);
            let owned;
            let name: &str = if is_pdf && detect_png(initial_bytes) {
                owned = format!("{name}.png");
                &owned
            } else {
                name
            };

            let sanitized = sanitize_output_filename(name, self.fs)?;
            return self.determine_filename(&sanitized);
        }

        // The extension is read verbatim out of the OneNote file, so it is fully
        // attacker controlled and must not reach `output.join()` unsanitized —
        // separators in it would otherwise escape the output directory. If
        // nothing survives sanitization, fall through to the extensionless name
        // rather than failing the whole conversion.
        if let Some(ext) = extension.and_then(|ext| sanitize_output_filename(ext, self.fs).ok()) {
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

        let mut i = 0;

        loop {
            let filename = format!("image{}", i);

            if !self.section.files.contains(&filename) {
                self.section.files.insert(filename.clone());

                return Ok(filename);
            }

            i += 1;
        }
    }
}

fn read_file_start(reader: &mut Box<dyn Read>) -> Result<Vec<u8>> {
    const SIZE: usize = 1024;
    let mut sub_reader = reader.by_ref().take(SIZE as u64);
    let mut bytes = Vec::with_capacity(SIZE);
    sub_reader
        .read_to_end(&mut bytes)
        .wrap_err("Failed to read image prefix")?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::Options;
    use crate::page::Renderer;
    use crate::section;
    use crate::test_utils::StubFs;
    use typed_path::TypedPathBuf;

    const PNG_HEADER: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A];

    fn determine(
        windows: bool,
        taken: &[&str],
        image_filename: Option<&str>,
        extension: Option<&str>,
        initial_bytes: &[u8],
    ) -> String {
        let fs = StubFs { windows };
        let mut section = section::Renderer::new();
        for name in taken {
            section.files.insert((*name).to_string());
        }
        let output = TypedPathBuf::from_unix("/out");
        let mut renderer = Renderer::new(output, &mut section, Options::default(), fs);
        renderer
            .determine_image_filename(image_filename, extension, initial_bytes)
            .unwrap()
    }

    fn from_extension(ext: &str) -> String {
        determine(false, &[], None, Some(ext), &[])
    }

    #[test]
    fn extension_is_appended_when_harmless() {
        assert_eq!(from_extension(".png"), "image0.png");
    }

    #[test]
    fn extension_without_leading_dot_is_preserved() {
        assert_eq!(from_extension("png"), "image0png");
    }

    #[test]
    fn extension_collisions_still_increment() {
        assert_eq!(
            determine(
                false,
                &["image0.png", "image1.png"],
                None,
                Some(".png"),
                &[]
            ),
            "image2.png"
        );
    }

    // GHSA-86gx-fw5h-h2gp: a crafted PictureFileExtension must not be able to
    // walk out of the output directory.
    #[test]
    fn extension_cannot_traverse_with_unix_separators() {
        let name = from_extension("../../../pwned.txt");
        assert!(!name.contains('/'), "{name} still contains a separator");
        assert!(!name.contains(".."), "{name} still contains a traversal");
    }

    #[test]
    fn extension_cannot_traverse_with_windows_separators() {
        let name = determine(true, &[], None, Some(r"..\..\pwned.txt"), &[]);
        assert!(!name.contains('\\'), "{name} still contains a separator");
        assert!(!name.contains(".."), "{name} still contains a traversal");
    }

    #[test]
    fn extension_that_sanitizes_to_nothing_falls_back_to_no_extension() {
        // Must not abort the conversion: malformed input should degrade, not fail.
        assert_eq!(from_extension(".."), "image0");
        assert_eq!(from_extension("/"), "image0");
        assert_eq!(from_extension(""), "image0");
    }

    #[test]
    fn missing_extension_falls_back_to_no_extension() {
        assert_eq!(determine(false, &[], None, None, &[]), "image0");
        assert_eq!(determine(false, &["image0"], None, None, &[]), "image1");
    }

    #[test]
    fn image_filename_cannot_traverse() {
        let name = determine(false, &[], Some("../../pwned.txt"), None, &[]);
        assert_eq!(name, "pwned.txt");
    }

    #[test]
    fn image_filename_takes_precedence_over_extension() {
        assert_eq!(
            determine(false, &[], Some("photo.jpg"), Some(".png"), &[]),
            "photo.jpg"
        );
    }

    #[test]
    fn pdf_printout_with_png_payload_gains_png_extension() {
        assert_eq!(
            determine(false, &[], Some("printout.PDF"), None, PNG_HEADER),
            "printout.PDF.png"
        );
    }

    #[test]
    fn pdf_without_png_payload_keeps_its_extension() {
        assert_eq!(
            determine(false, &[], Some("printout.PDF"), None, b"%PDF-1.7"),
            "printout.PDF"
        );
    }
}
