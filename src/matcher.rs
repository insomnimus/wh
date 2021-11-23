use std::fmt;

use globset::GlobMatcher;

pub enum Matcher<'a> {
	Glob(GlobMatcher),
	Str(&'a str),
}

pub struct MatchState<'a> {
	pub matcher: Matcher<'a>,
	pub count: usize,
}

impl<'a> fmt::Display for Matcher<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Str(s) => f.write_str(s),
			Self::Glob(g) => f.write_str(g.glob().glob()),
		}
	}
}
