//! Parses the output of running `(alias; declare -f) on Bash.
//!
//! This is a pretty naive implementation but it should be enough for the use
//! case.

use std::fmt::{
	self,
};

use nom::{
	branch::alt,
	bytes::complete::{
		is_a,
		is_not,
		tag,
	},
	character::complete::{
		multispace0,
		space0,
		space1,
	},
	combinator::{
		iterator,
		map,
		opt,
	},
	error::{
		ErrorKind,
		ParseError,
	},
	sequence::{
		pair,
		preceded,
		tuple,
	},
	Parser,
};

pub enum Item<'a> {
	Alias(Alias<'a>),
	Function(Function<'a>),
}

pub struct Alias<'a> {
	pub lhs: &'a str,
	pub rhs: &'a str,
}

pub struct Function<'a> {
	pub name: &'a str,
	pub body: &'a str,
}

impl<'a> fmt::Display for Alias<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "alias {}={}", self.lhs, self.rhs)
	}
}

impl<'a> fmt::Display for Function<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}() {{\n{}\n}}", self.name, self.body)
	}
}

impl<'a> fmt::Display for Item<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Alias(x) => x.fmt(f),
			Self::Function(x) => x.fmt(f),
		}
	}
}

#[derive(Debug)]
struct Dummy;

type IResult<I, O, E = Dummy> = Result<(I, O), nom::Err<E>>;

macro_rules! err {
	[] => (::std::result::Result::Err(::nom::Err::Error(Dummy)));
}

impl<T> ParseError<T> for Dummy {
	fn from_error_kind(_: T, _: ErrorKind) -> Self {
		Self
	}

	fn append(_: T, _: ErrorKind, _: Self) -> Self {
		Self
	}
}

fn terminated_word(until: char) -> impl Fn(&str) -> IResult<&str, &str> {
	move |input| {
		for (i, c) in input.char_indices() {
			if c == until {
				if i == 0 {
					return err!();
				}
				return Ok((&input[i + 1..], &input[..i]));
			}
			if c.is_ascii_whitespace() {
				return err!();
			}
		}

		err!()
	}
}

fn consume_line(input: &str) -> IResult<&str, &str> {
	if let Some(i) = input.find('\n') {
		return Ok((&input[i + 1..], &input[..i + 1]));
	}
	if input.is_empty() {
		err!()
	} else {
		Ok(("", input))
	}
}

fn parse_alias(input: &str) -> IResult<&str, (&str, &str)> {
	preceded(
		pair(tag("alias"), space1),
		pair(terminated_word('='), consume_line.map(|s| s.trim_end())),
	)(input)
}

fn parse_function(input: &str) -> IResult<&str, (&str, &str)> {
	let not_allowed = "<>[]{}()\t\r\n\0 \\*?$~;#`'\"";
	tuple((
		opt(tuple((tag("function"), space1))),
		is_not(not_allowed),
		tuple((space0, tag("()"), multispace0, tag("{"), space0, is_a("\n"))),
		until_alone_rbrace,
		// tag("}"),
	))
	.map(|t| (t.1, t.3))
	.parse(input)
}

fn until_alone_rbrace(input: &str) -> IResult<&str, &str> {
	let mut i = 0;
	while let Some(idx) = input[i..].find("\n}") {
		let remaining = &input[i + idx + 2..];
		match remaining.split_once('\n') {
			Some((ln, rest)) => {
				if ln.trim().is_empty() {
					return Ok((rest, &input[..i + idx]));
				}
			}
			None => {
				if remaining.trim_start().is_empty() {
					return Ok(("", &input[..i + idx]));
				} else {
					return err!();
				}
			}
		}

		i += idx + 2;
	}

	err!()
}

pub fn parse(input: &str) -> Vec<Item> {
	iterator(
		input,
		alt((
			map(parse_function, |(name, body)| {
				Some(Item::Function(Function {
					name,
					body: body.trim_matches('\n'),
				}))
			}),
			map(parse_alias, |(lhs, rhs)| {
				Some(Item::Alias(Alias {
					lhs,
					rhs: rhs.trim_end(),
				}))
			}),
			map(consume_line, |_| None),
		)),
	)
	.flatten()
	// .inspect(|x| println!("{x}"))
	.collect()
}
