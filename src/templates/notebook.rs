use crate::notebook::RgbColor;
use askama::Template;

#[derive(Template)]
#[template(path = "notebook.html")]
struct NotebookTemplate<'a> {
    name: &'a str,
    sections: &'a [Section],
}

#[derive(Debug)]
pub(crate) struct Section {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) color: Option<RgbColor>,
}

pub(crate) fn render(name: &str, sections: &[Section]) -> String {
    let template = NotebookTemplate { name, sections };

    template.render().expect("failed to render template")
}

mod filters {
    pub(crate) use crate::templates::urlencode;
}
