use crate::page::Renderer;
use onenote::{EmbeddedFile, FileType};
use std::fs;
use std::path::PathBuf;

impl<'a> Renderer<'a> {
    pub(crate) fn render_embedded_file(&mut self, file: &EmbeddedFile) -> String {
        let filename = self.determine_filename(file.filename());
        fs::write(self.output.join(filename.clone()), file.data())
            .expect("failed to write embedded file");

        let file_type = Self::guess_type(file);

        match file_type {
            FileType::Audio => format!("<audio controls src=\"{}\"></audio>", filename),
            FileType::Video => format!("<video controls src=\"{}\"></video>", filename),
            FileType::Unknown => format!("<embed src=\"{}\" />", filename),
        }
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

    pub(crate) fn determine_filename(&mut self, filename: &str) -> String {
        let mut i = 0;
        let mut current_filename = filename.to_string();

        loop {
            if !self.section.files.contains(&current_filename) {
                self.section.files.insert(current_filename.clone());

                return current_filename;
            }

            let path = PathBuf::from(filename);
            let ext = path
                .extension()
                .expect("embedded file has no extension")
                .to_str()
                .expect("embedded file name is non utf-8");
            let base = path
                .as_os_str()
                .to_str()
                .expect("embedded file name is non utf-8")
                .strip_suffix(ext)
                .unwrap()
                .trim_matches('.');

            current_filename = format!("{}-{}.{}", base, i, ext);

            i += 1;
        }
    }
}
