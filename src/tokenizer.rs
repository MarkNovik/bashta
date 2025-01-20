use crate::Location;
use std::iter::Peekable;
use std::str::CharIndices;

pub fn tokenize<'a>(source: &'a str, path: Option<&'a str>) -> Tokenizer<'a> {
    Tokenizer::new(source, path)
}

#[derive(Debug, Copy, Clone)]
pub struct Token<'a> {
    pub content: &'a str,
    pub location: Location<'a>,
}

#[derive(Clone)]
pub struct Tokenizer<'a> {
    source: &'a str,
    indices: Peekable<CharIndices<'a>>,
    path: Option<&'a str>,
    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(source: &'a str, path: Option<&'a str>) -> Self {
        Self {
            source,
            indices: source.char_indices().peekable(),
            path,
            line: 1,
            column: 1,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((i, c)) = self.indices.next() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
                continue;
            } else if c.is_whitespace() {
                self.column += 1;
                continue;
            }
            let mut s = String::new();
            s.push(c);
            let mut offset = 1;
            while let Some((_, c)) = self.indices.next_if(|&(_, c)| !c.is_whitespace()) {
                s.push(c);
                offset += 1;
            }
            let column = self.column;
            self.column += offset;
            let last_byte = self.indices.peek().map(|&(b, _)| b).unwrap_or(self.source.len());
            return Some(Token {
                location: Location {
                    path: self.path,
                    line: self.line,
                    column,
                },
                content: &self.source[i..last_byte],
            });
        }
        None
    }
}