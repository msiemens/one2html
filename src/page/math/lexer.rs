use color_eyre::Result;
use onenote_parser::contents::{MathInlineObject, MathObjectType};
use std::collections::VecDeque;

// See https://learn.microsoft.com/en-us/archive/blogs/murrays/officemath for the origin of those
// constants
const TOK_START: &str = "\u{fdd0}";
const TOK_SEP: &str = "\u{fdee}";
const TOK_END: &str = "\u{fdef}";

#[derive(PartialEq, Debug)]
pub(super) enum Token {
    Text(String),
    Start(MathInlineObject),
    Sep(MathObjectType),
    End(MathObjectType),
    Eof,
}

pub(super) struct Lexer {
    tokens: VecDeque<(String, MathInlineObject)>,
}

impl Lexer {
    pub(super) fn new(tokens: Vec<(String, MathInlineObject)>) -> Lexer {
        Lexer {
            tokens: VecDeque::from(tokens),
        }
    }

    pub(super) fn next(&mut self) -> Result<Token> {
        let (text, object) = match self.tokens.pop_front() {
            Some(tok) => tok,
            None => return Ok(Token::Eof),
        };

        match text.as_str() {
            TOK_START => Ok(Token::Start(object)),
            TOK_SEP => Ok(Token::Sep(object.object_type())),
            TOK_END => Ok(Token::End(object.object_type())),
            t if t.starts_with(TOK_START) => {
                let remainder = t[TOK_START.len()..].to_string();
                self.tokens.push_front((remainder, object));

                Ok(Token::Start(object))
            }
            t if t.ends_with(TOK_SEP) => {
                let content = t[..t.len() - TOK_SEP.len()].to_string();
                self.tokens.push_front((TOK_SEP.to_string(), object));

                Ok(Token::Text(content))
            }
            t if t.ends_with(TOK_END) => {
                let content = t[..t.len() - TOK_END.len()].to_string();
                self.tokens.push_front((TOK_END.to_string(), object));

                Ok(Token::Text(content))
            }
            t => Ok(Token::Text(t.to_string())),
        }
    }
}
