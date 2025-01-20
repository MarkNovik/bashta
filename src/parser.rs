use std::collections::HashMap;
use crate::{Location, Operation, ParseError, ParseErrorType};
use crate::tokenizer::Token;

pub fn parse_tokens<'a, I: Iterator<Item=Token<'a>> + Clone>(tokens: I) -> Parser<'a, I> { Parser::new(tokens) }

pub struct Parser<'a, I: Iterator<Item=Token<'a>>> {
    tokens: I,
    labels: HashMap<&'a str, u64>,
}

impl<'a, I: Iterator<Item=Token<'a>> + Clone> Parser<'a, I> {
    fn new(tokens: I) -> Self {
        let labels = tokens.clone().enumerate()
            .filter_map(|(i, t)| t.content.strip_suffix(':')
                .map(|lbl| (lbl, i as u64))).collect();
        Self { tokens, labels }
    }
}

impl<'a, I: Iterator<Item=Token<'a>>> Iterator for Parser<'a, I> {
    type Item = Result<(Operation, Location<'a>), ParseError<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let t = self.tokens.next()?;
        Some(if let Ok(num) = t.content.parse::<u8>() {
            Ok((Operation::PushByte(num), t.location))
        } else if t.content == "put-char" {
            Ok((Operation::PrintAscii, t.location))
        } else if t.content == "div-u8" {
            Ok((Operation::DivU8, t.location))
        } else if t.content == "mul-u8" {
            Ok((Operation::MulU8, t.location))
        } else if t.content == "sub-u8" {
            Ok((Operation::SubU8, t.location))
        } else if t.content == "ass-u8" {
            Ok((Operation::AddU8, t.location))
        } else if t.content == "debug" {
            Ok((Operation::Debug, t.location))
        } else if t.content == "print-u8" {
            Ok((Operation::PrintU8, t.location))
        } else if t.content == "print-u16" {
            Ok((Operation::PrintU16, t.location))
        } else if t.content == "print-u32" {
            Ok((Operation::PrintU32, t.location))
        } else if t.content == "print-u64" {
            Ok((Operation::PrintU64, t.location))
        } else if let Some(label) = t.content.strip_prefix(';') {
            self.labels.get(label)
                .map(|ptr| (Operation::PushLong(*ptr), t.location))
                .ok_or(ParseError {
                    typ: ParseErrorType::UndefinedLabel(label),
                    cause: t,
                })
        } else if t.content.strip_suffix(':').is_some() {
            Ok((Operation::Noop, t.location))
        } else if t.content == "exit" {
            Ok((Operation::Exit, t.location))
        } else if t.content == "call" {
            Ok((Operation::Call, t.location))
        } else if t.content == "return" {
            Ok((Operation::Return, t.location))
        } else {
            Err(ParseError {
                typ: ParseErrorType::UnknownSymbol,
                cause: t,
            })
        })
    }
}