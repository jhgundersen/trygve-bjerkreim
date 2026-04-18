/// Lexer for Trygve Bjerkreim
///
/// Underscore (_) is treated as whitespace — words are separated by spaces or _.
///
/// Produces a flat Vec<Token> from source text.
/// Comments: – – (en-dash pair) to end of line.
/// Strings:  «…» (Norwegian guillemets).

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Word(String),
    Int(i64),
    Float(f64),
    Str(String),
    Comma,
    Dot,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}

impl Token {
    pub fn is_word(&self, w: &str) -> bool {
        matches!(&self.kind, TokenKind::Word(s) if s == w)
    }
    pub fn word_str(&self) -> Option<&str> {
        match &self.kind {
            TokenKind::Word(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    for (lineno, raw_line) in source.lines().enumerate() {
        let line = strip_comment(raw_line);
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            // Whitespace (underscore also treated as whitespace — no _ in identifiers)
            if c.is_whitespace() || c == '_' {
                i += 1;
                continue;
            }

            // String «...»
            if c == '«' {
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '»' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        match chars[i + 1] {
                            'n' => { s.push('\n'); i += 2; }
                            't' => { s.push('\t'); i += 2; }
                            _   => { s.push(chars[i]); i += 1; }
                        }
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                if i >= chars.len() {
                    return Err(format!("Line {}: unterminated string «", lineno + 1));
                }
                tokens.push(Token { kind: TokenKind::Str(s), line: lineno + 1 });
                i += 1; // skip »
                continue;
            }

            // Number
            if c.is_ascii_digit() {
                let start = i;
                let mut has_dot = false;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    if chars[i] == '.' { has_dot = true; }
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                if has_dot {
                    let f: f64 = s.parse().map_err(|e| format!("Line {}: bad float {}: {}", lineno+1, s, e))?;
                    tokens.push(Token { kind: TokenKind::Float(f), line: lineno + 1 });
                } else {
                    let n: i64 = s.parse().map_err(|e| format!("Line {}: bad int {}: {}", lineno+1, s, e))?;
                    tokens.push(Token { kind: TokenKind::Int(n), line: lineno + 1 });
                }
                continue;
            }

            // Single-char punctuation
            let punct = match c {
                ',' => Some(TokenKind::Comma),
                '.' => Some(TokenKind::Dot),
                ':' => Some(TokenKind::Colon),
                '(' => Some(TokenKind::LParen),
                ')' => Some(TokenKind::RParen),
                '[' => Some(TokenKind::LBracket),
                ']' => Some(TokenKind::RBracket),
                _ => None,
            };
            if let Some(k) = punct {
                tokens.push(Token { kind: k, line: lineno + 1 });
                i += 1;
                continue;
            }

            // Word: everything else until whitespace or punctuation
            let start = i;
            while i < chars.len()
                && !chars[i].is_whitespace()
                && !matches!(chars[i], ',' | '.' | ':' | '(' | ')' | '[' | ']' | '«' | '»')
            {
                i += 1;
            }
            if i > start {
                let w: String = chars[start..i].iter().collect();
                tokens.push(Token { kind: TokenKind::Word(w), line: lineno + 1 });
            }
        }
    }

    tokens.push(Token { kind: TokenKind::Eof, line: 0 });
    Ok(tokens)
}

/// Strip – – comment (en-dash or em-dash pair) to end of line.
fn strip_comment(line: &str) -> &str {
    // Look for two consecutive dash-family chars with optional whitespace between
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if is_dash(chars[i]) {
            // Look ahead for optional whitespace then another dash
            let mut j = i + 1;
            while j < n && chars[j] == ' ' {
                j += 1;
            }
            if j < n && is_dash(chars[j]) {
                // Found comment marker — return slice up to i
                let byte_pos = chars[..i].iter().collect::<String>().len();
                return &line[..byte_pos];
            }
        }
        i += 1;
    }
    line
}

fn is_dash(c: char) -> bool {
    c == '–' || c == '—' || c == '-'
}
