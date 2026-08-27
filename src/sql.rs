#[derive(Debug)]
pub(crate) struct Split {
    pub statements: Vec<String>,
    pub remainder: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scan {
    Normal,
    SingleQuote,
    DoubleQuote,
    Backtick,
    Bracket,
    LineComment,
    BlockComment,
}

#[must_use]
pub(crate) fn split_statements(input: &str) -> Split {
    let mut statements: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut scan = Scan::Normal;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match scan {
            Scan::Normal => match c {
                '\'' => {
                    scan = Scan::SingleQuote;
                    current.push(c);
                }
                '"' => {
                    scan = Scan::DoubleQuote;
                    current.push(c);
                }
                '`' => {
                    scan = Scan::Backtick;
                    current.push(c);
                }
                '[' => {
                    scan = Scan::Bracket;
                    current.push(c);
                }
                '-' if chars.peek() == Some(&'-') => {
                    chars.next();
                    scan = Scan::LineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    chars.next();
                    scan = Scan::BlockComment;
                }
                ';' => {
                    let statement = current.trim().to_string();
                    if !statement.is_empty() {
                        statements.push(statement);
                    }
                    current.clear();
                }
                _ => current.push(c),
            },
            Scan::SingleQuote => {
                current.push(c);
                if c == '\'' {
                    scan = Scan::Normal;
                }
            }
            Scan::DoubleQuote => {
                current.push(c);
                if c == '"' {
                    scan = Scan::Normal;
                }
            }
            Scan::Backtick => {
                current.push(c);
                if c == '`' {
                    scan = Scan::Normal;
                }
            }
            Scan::Bracket => {
                current.push(c);
                if c == ']' {
                    scan = Scan::Normal;
                }
            }
            Scan::LineComment => {
                if c == '\n' {
                    scan = Scan::Normal;
                    current.push(c);
                }
            }
            Scan::BlockComment => {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    scan = Scan::Normal;
                    current.push(' ');
                }
            }
        }
    }

    Split {
        statements,
        remainder: current,
    }
}
