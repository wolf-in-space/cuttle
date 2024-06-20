use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Lines {
    pub lines: Vec<String>,
}

impl From<String> for Lines {
    fn from(line: String) -> Self {
        Self { lines: vec![line] }
    }
}

impl From<&str> for Lines {
    fn from(line: &str) -> Self {
        Self {
            lines: vec![line.to_string()],
        }
    }
}

impl From<Vec<String>> for Lines {
    fn from(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

impl From<Vec<&str>> for Lines {
    fn from(lines: Vec<&str>) -> Self {
        Self {
            lines: lines.into_iter().map(ToString::to_string).collect(),
        }
    }
}

impl From<Vec<Lines>> for Lines {
    fn from(lines: Vec<Lines>) -> Self {
        lines
            .into_iter()
            .fold(Lines::new(), |accu, lines| accu.merge(lines))
    }
}

impl<const I: usize> From<[Lines; I]> for Lines {
    fn from(lines: [Lines; I]) -> Self {
        lines
            .into_iter()
            .fold(Lines::new(), |accu, lines| accu.merge(lines))
    }
}

impl FromIterator<Lines> for Lines {
    fn from_iter<T: IntoIterator<Item = Lines>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Lines::new(), |prev, new| prev.merge(new))
    }
}

impl Display for Lines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.clone().into_file_str())
    }
}

#[macro_export]
macro_rules! linefy {
    ($($text:tt)*) => {
        Lines::from_linefy(stringify!($($text)*))
    };
}

#[macro_export]
macro_rules! line_f {
    ($($text:tt)*) => {
        Lines::from(format!($($text)*))
    };
}

impl Lines {
    pub const fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn from_linefy(string: impl Into<String>) -> Self {
        Self {
            lines: string
                .into()
                .replace('\n', " ")
                .split_whitespace()
                .join(" ")
                .split_inclusive(|char| ['{', '}', ';'].contains(&char))
                .map(|s| s.trim_start().to_string())
                .collect(),
        }
    }

    pub fn into_file_str(self) -> String {
        self.lines
            .into_iter()
            .fold((String::new(), 0), |(mut result, mut indent), mut line| {
                let ind = " ".repeat(indent);
                let mut add_to_result = |s: &str| {
                    result += &ind;
                    result += s;
                    result += "\n";
                };
                if line.ends_with('}') {
                    line = line.strip_suffix('}').unwrap().into();
                    line.split_inclusive(',')
                        .map(str::trim_start)
                        .filter(|s| !s.is_empty())
                        .for_each(add_to_result);
                    indent -= 4;
                    result += "}\n \n";
                } else {
                    add_to_result(&line)
                };

                if line.ends_with('{') {
                    indent += 4;
                };

                (result, indent)
            })
            .0
    }

    pub fn add(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    pub fn merge(mut self, other: impl Into<Self>) -> Self {
        self.lines.extend(other.into().lines);
        self
    }

    pub fn block(self, other: impl Into<Self>) -> Self {
        self.add("{").merge(other).add("}")
    }
}
