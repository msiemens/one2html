mod ast;
mod lexer;
mod parser;
mod render;
mod text;

use crate::page::Renderer;
use color_eyre::Result;
use onenote_parser::contents::MathInlineObject;
use parser::Parser;
use render::render_equation;

impl<'a> Renderer<'a> {
    pub(crate) fn render_math(&self, segments: Vec<(String, MathInlineObject)>) -> Result<String> {
        let mut parser = Parser::new(segments)?;
        let equation = parser.parse()?;

        let markup = render_equation(equation)?;

        Ok(markup)
    }
}
