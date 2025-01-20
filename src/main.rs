mod tokenizer;
mod parser;

use parser::parse_tokens;
use std::fmt::{Display, Formatter};
use thiserror::Error;
use tokenizer::{tokenize, Token};

macro_rules! print_unsigned_impl {
    ($typ:ty, $idx:expr, $location:expr, $stack:expr) => {
        match $stack.data.last_chunk() { 
            Some(ch) => {
                println!("{num}", num = <$typ>::from_be_bytes(*ch));
                $stack.data.truncate(ch.len());
                Ok($idx + 1)
            }
            None => Err(StackError {
                typ: StackErrorType::InsufficientElements { expected: size_of::<$typ>(), found: $stack.data.len() },
                location: $location,
            })
        }
    };
}

macro_rules! u8_binop_impl {
    ($op:tt, $stack:expr, $idx:expr, $location:expr) => {
        match $stack.data.last_chunk() {
            Some([a, b]) => {
                let res = a $op b;
                $stack.data.truncate($stack.data.len() - 2);
                $stack.data.push(res);
                Ok($idx + 1)
            }
            None => Err(StackError {
                typ: StackErrorType::InsufficientElements { expected: 2, found: $stack.data.len() },
                location: $location
            })
        }
    };
}

#[derive(Debug, Default)]
struct Stack {
    data: Vec<u8>,
    ret: Vec<usize>,
}

const DEFAULT_SOURCE: &str = include_str!("./гол.баш");

fn main() -> anyhow::Result<()> {
    let tokens = tokenize(DEFAULT_SOURCE, Some("./гол.баш")); //.on_each(|t| println!("{t:?}"));
    let operations = parse_tokens(tokens).collect::<Result<Vec<_>, _>>()?;
    //println!("{:#?}", operations);
    let mut stack = Stack::default();
    let mut idx = 0;
    while idx < operations.len() {
        let (operation, loc) = &operations[idx];
        idx = operation.op(&mut stack, idx, loc)?;
    }

    Ok(())
}


#[derive(Debug)]
pub enum StackErrorType {
    InsufficientElements {
        expected: usize,
        found: usize,
    },
    NowhereToReturn,
    NonZeroExit { code: u8 },
}

impl Display for StackErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StackErrorType::InsufficientElements { expected, found } => {
                write!(f, "Insufficient elements on the stack, expected {expected}, but found {found}")
            }
            StackErrorType::NowhereToReturn => write!(f, "Nowhere to return"),
            StackErrorType::NonZeroExit { code } => write!(f, "Exited with nonzero exit code {code}"),
        }
    }
}

#[derive(Debug, Error)]
#[error("{location}: Error: {typ}")]
pub struct StackError<'a> {
    typ: StackErrorType,
    location: Location<'a>,
}

#[derive(Debug)]
pub enum ParseErrorType<'a> {
    UnknownSymbol,
    UndefinedLabel(&'a str),
}

impl Display for ParseErrorType<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorType::UnknownSymbol => {
                write!(f, "Unknown symbol")
            }
            ParseErrorType::UndefinedLabel(lbl) => {
                write!(f, "Undefined label `{lbl}`")
            }
        }
    }
}

#[derive(Debug, Error)]
#[error("{loc}: Error: {typ}, caused by `{sym}`", loc = .cause.location, sym = .cause.content)]
pub struct ParseError<'a> {
    typ: ParseErrorType<'a>,
    cause: Token<'a>,
}

#[derive(Debug, Copy, Clone)]
pub struct Location<'a> {
    path: Option<&'a str>,
    line: usize,
    column: usize,
}

impl Display for Location<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{path}:")?
        };
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug)]
pub enum Operation {
    PushByte(u8),
    PushLong(u64),
    Call,
    Return,
    AddU8,
    SubU8,
    MulU8,
    DivU8,
    PrintAscii,
    PrintU8,
    PrintU16,
    PrintU32,
    PrintU64,
    Debug,
    Noop,
    Exit
}

impl Operation {
    fn op<'a>(&self, stack: &mut Stack, idx: usize, &location: &Location<'a>) -> Result<usize, StackError<'a>> {
        match self {
            &Operation::PushByte(b) => {
                stack.data.push(b);
                Ok(idx + 1)
            }
            Operation::PushLong(ptr) => {
                stack.data.extend(ptr.to_ne_bytes());
                Ok(idx + 1)
            }
            Operation::Call => {
                let Some(ptr) = stack.data.last_chunk().map(|c| usize::from_ne_bytes(*c)) else {
                    return Err(StackError {
                        typ: StackErrorType::InsufficientElements { expected: 8, found: stack.data.len() },
                        location,
                    })
                };
                stack.data.truncate(stack.data.len() - size_of::<u64>());
                stack.ret.push(idx + 1);
                Ok(ptr)
            }
            Operation::Return => {
                stack.ret.pop().ok_or(StackError {
                    typ: StackErrorType::NowhereToReturn,
                    location,
                })
            }
            Operation::AddU8 => u8_binop_impl!(+, stack, idx, location),
            Operation::SubU8 => u8_binop_impl!(-, stack, idx, location),
            Operation::MulU8 => u8_binop_impl!(*, stack, idx, location),
            Operation::DivU8 => u8_binop_impl!(/, stack, idx, location),
            Operation::PrintAscii => {
                let Some(c) = stack.data.pop() else {
                    return Err(
                        StackError {
                            typ: StackErrorType::InsufficientElements { expected: 1, found: 0 },
                            location,
                        }
                    )
                };
                println!("{}", char::from(c));
                Ok(idx + 1)
            }
            Operation::PrintU8 => print_unsigned_impl!(u8, idx, location, stack),
            Operation::PrintU16 => print_unsigned_impl!(u16, idx, location, stack),
            Operation::PrintU32 => print_unsigned_impl!(u32, idx, location, stack),
            Operation::PrintU64 => print_unsigned_impl!(u64, idx, location, stack),
            Operation::Debug => {
                println!("{:#?}", stack);
                Ok(idx + 1)
            }
            Operation::Noop => Ok(idx + 1),
            Operation::Exit => {
                match stack.data.pop() {
                    Some(0) | None => Ok(usize::MAX),
                    Some(code) => Err(StackError { typ: StackErrorType::NonZeroExit { code }, location }) 
                }
            },
        }
    }
}