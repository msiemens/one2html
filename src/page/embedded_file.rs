use crate::page::Renderer;
use crate::utils::{html_entities, sanitize_output_filename};
use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use onenote_parser::FileSystem;
use onenote_parser::contents::EmbeddedFile;
use onenote_parser::property::embedded_file::FileType;

impl<'a, FS: FileSystem> Renderer<'a, FS> {
    pub(crate) fn render_embedded_file(&mut self, file: &EmbeddedFile) -> Result<String> {
        let filename = self.determine_filename(file.filename())?;

        let target_file = self.output.join(filename.clone());

        let mut reader = file.read();
        self.fs
            .stream_to_file(target_file.to_path(), &mut *reader)
            .wrap_err("Failed to write embedded file")?;

        let file_type = Self::guess_type(file);

        let src = html_entities(&filename);
        let content = match file_type {
            FileType::Audio => format!("<audio controls src=\"{}\"></audio>", src),
            FileType::Video => format!("<video controls src=\"{}\"></video>", src),
            FileType::Unknown => format!("<embed src=\"{}\" />", src),
        };

        Ok(self.render_with_note_tags(file.note_tags(), content))
    }

    fn guess_type(file: &EmbeddedFile) -> FileType {
        match file.file_type() {
            FileType::Audio => return FileType::Audio,
            FileType::Video => return FileType::Video,
            _ => {}
        };

        let filename = file.filename();

        if let Some(mime) = mime_guess::from_path(filename).first() {
            if mime.type_() == "audio" {
                return FileType::Audio;
            }

            if mime.type_() == "video" {
                return FileType::Video;
            }
        }
        FileType::Unknown
    }

    pub(crate) fn determine_filename(&mut self, filename: &str) -> Result<String> {
        let mut i = 0;
        let sanitized = sanitize_output_filename(filename, self.fs)?;
        let mut current_filename = sanitized.clone();

        loop {
            if !self.section.files.contains(&current_filename) {
                self.section.files.insert(current_filename.clone());

                return Ok(current_filename);
            }

            let (base, ext) = match sanitized.rsplit_once('.') {
                Some((base, ext)) if !base.is_empty() && !ext.is_empty() => (base, ext),
                _ => (sanitized.trim_end_matches('.'), "bin"),
            };

            current_filename = format!("{}-{}.{}", base, i, ext);

            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Options;
    use crate::page::Renderer;
    use crate::section;
    use crate::test_utils::StubFs;
    use typed_path::TypedPathBuf;

    fn with_taken(taken: &[&str], filename: &str) -> String {
        let fs = StubFs { windows: false };
        let mut section = section::Renderer::new();
        for name in taken {
            section.files.insert((*name).to_string());
        }
        let output = TypedPathBuf::from_unix("/out");
        let mut renderer = Renderer::new(output, &mut section, Options::default(), fs);
        renderer.determine_filename(filename).unwrap()
    }

    #[test]
    fn returns_input_when_unused() {
        assert_eq!(with_taken(&[], "image.png"), "image.png");
    }

    #[test]
    fn suffixes_on_collision_preserving_extension() {
        assert_eq!(with_taken(&["image.png"], "image.png"), "image-0.png");
    }

    #[test]
    fn keeps_incrementing_past_existing_suffixes() {
        assert_eq!(
            with_taken(&["image.png", "image-0.png", "image-1.png"], "image.png"),
            "image-2.png"
        );
    }

    #[test]
    fn no_extension_falls_back_to_bin() {
        assert_eq!(with_taken(&["README"], "README"), "README-0.bin");
    }

    #[test]
    fn trailing_dot_falls_back_to_bin() {
        assert_eq!(with_taken(&["file."], "file."), "file-0.bin");
    }

    #[test]
    fn leading_dot_treated_as_extensionless() {
        assert_eq!(
            with_taken(&[".gitignore"], ".gitignore"),
            ".gitignore-0.bin"
        );
    }

    #[test]
    fn multiple_dots_split_at_last_dot() {
        assert_eq!(
            with_taken(&["archive.tar.gz"], "archive.tar.gz"),
            "archive.tar-0.gz"
        );
    }
}
