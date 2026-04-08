/// All tokens of the Holy language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Testament, Revealing, Scripture, Sin, Covenant, Salm, Upon, Receiving, Reveals,
    Let, There, Be, Of, Become, Hail, Praying, Reveal, Whether, Otherwise,
    So, Litany, For, Confess, Answer, Absolve, As, Transgress, Manifest,
    From, Its, Discern, Amen, Forsake, Ascend, Bearing,
    // Grouping / context markers
    After, Thus,
    // Word operators
    Plus, Minus, Times, Over, Remainder, Negate,
    Is, Not, Greater, Lesser, Than, No,
    Blessed, Forsaken, And,
    // Types
    Void, Atom, Fractional, Word, Dogma,
    // Punctuation and indentation
    Comma, Indent, Dedent,
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),
    // Identifier
    Ident(String),
    Eof,
}

/// Token with source position.
#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub line:  usize, // 1-indexed
    pub col:   usize, // 1-indexed
}

impl Spanned {
    fn new(token: Token, line: usize, col: usize) -> Self {
        Spanned { token, line, col }
    }
}

/// Human-readable token name used in error messages.
pub fn token_name(t: &Token) -> String {
    match t {
        Token::Ident(n)   => format!("'{}'", n),
        Token::IntLit(n)  => format!("{}", n),
        Token::FloatLit(f)=> format!("{}", f),
        Token::StrLit(s)  => format!("\"{}\"", s),
        Token::Eof        => "end of file".into(),
        Token::Indent     => "block start (indent)".into(),
        Token::Dedent     => "block end (dedent)".into(),
        Token::Comma      => "','".into(),
        Token::Testament  => "'testament'".into(),
        Token::Revealing  => "'revealing'".into(),
        Token::Scripture  => "'scripture'".into(),
        Token::Sin        => "'sin'".into(),
        Token::Covenant   => "'covenant'".into(),
        Token::Salm       => "'salm'".into(),
        Token::Upon       => "'upon'".into(),
        Token::Receiving  => "'receiving'".into(),
        Token::Reveals    => "'reveals'".into(),
        Token::Let        => "'let'".into(),
        Token::There      => "'there'".into(),
        Token::Be         => "'be'".into(),
        Token::Of         => "'of'".into(),
        Token::Become     => "'become'".into(),
        Token::Hail       => "'hail'".into(),
        Token::Praying    => "'praying'".into(),
        Token::Reveal     => "'reveal'".into(),
        Token::Whether    => "'whether'".into(),
        Token::Otherwise  => "'otherwise'".into(),
        Token::So         => "'so'".into(),
        Token::Litany     => "'litany'".into(),
        Token::For        => "'for'".into(),
        Token::Confess    => "'confess'".into(),
        Token::Answer     => "'answer'".into(),
        Token::Absolve    => "'absolve'".into(),
        Token::As         => "'as'".into(),
        Token::Transgress => "'transgress'".into(),
        Token::Manifest   => "'manifest'".into(),
        Token::From       => "'from'".into(),
        Token::Its        => "'its'".into(),
        Token::Discern    => "'discern'".into(),
        Token::Amen       => "'amen'".into(),
        Token::Forsake    => "'forsake'".into(),
        Token::Ascend     => "'ascend'".into(),
        Token::Bearing    => "'bearing'".into(),
        Token::After      => "'after'".into(),
        Token::Thus       => "'thus'".into(),
        Token::Plus       => "'plus'".into(),
        Token::Minus      => "'minus'".into(),
        Token::Times      => "'times'".into(),
        Token::Over       => "'over'".into(),
        Token::Remainder  => "'remainder'".into(),
        Token::Negate     => "'negate'".into(),
        Token::Is         => "'is'".into(),
        Token::Not        => "'not'".into(),
        Token::Greater    => "'greater'".into(),
        Token::Lesser     => "'lesser'".into(),
        Token::Than       => "'than'".into(),
        Token::No         => "'no'".into(),
        Token::Blessed    => "'blessed'".into(),
        Token::Forsaken   => "'forsaken'".into(),
        Token::And        => "'and'".into(),
        Token::Void       => "'void'".into(),
        Token::Atom       => "'atom'".into(),
        Token::Fractional => "'fractional'".into(),
        Token::Word       => "'word'".into(),
        Token::Dogma      => "'dogma'".into(),
    }
}

/// Converts source code into a list of positioned tokens.
/// Indentation is tracked and emitted as `Indent`/`Dedent` tokens.
pub fn tokenize(source: &str) -> Vec<Spanned> {
    let mut result = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];

    for (line_idx, line) in source.lines().enumerate() {
        let line_num = line_idx + 1;

        let indent_count = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let content = line.trim();

        if content.is_empty() || content.starts_with("--") {
            continue;
        }

        let current = *indent_stack.last().unwrap();

        if indent_count > current {
            indent_stack.push(indent_count);
            result.push(Spanned::new(Token::Indent, line_num, indent_count + 1));
        } else if indent_count < current {
            while indent_stack.len() > 1 && *indent_stack.last().unwrap() > indent_count {
                indent_stack.pop();
                result.push(Spanned::new(Token::Dedent, line_num, indent_count + 1));
            }
        }

        tokenize_line(line, line_num, &mut result);
    }

    // Close any remaining open blocks
    while indent_stack.len() > 1 {
        indent_stack.pop();
        result.push(Spanned::new(Token::Dedent, 0, 0));
    }

    result.push(Spanned::new(Token::Eof, 0, 0));
    result
}

fn tokenize_line(line: &str, line_num: usize, result: &mut Vec<Spanned>) {
    let mut chars = line.chars().peekable();
    let mut col = 1usize;

    while let Some(&c) = chars.peek() {
        let tok_col = col;
        match c {
            ' ' | '\t' => { chars.next(); col += 1; }

            '-' => {
                chars.next(); col += 1;
                if chars.peek() == Some(&'-') {
                    break; // line comment
                }
                // Negative numeric literal: -1, -3.14
                if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    let mut num = String::from("-");
                    let mut has_dot = false;
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() { num.push(c); chars.next(); col += 1; }
                        else if c == '.' && !has_dot { has_dot = true; num.push(c); chars.next(); col += 1; }
                        else { break; }
                    }
                    if has_dot {
                        result.push(Spanned::new(Token::FloatLit(num.parse().unwrap_or(0.0)), line_num, tok_col));
                    } else {
                        result.push(Spanned::new(Token::IntLit(num.parse().unwrap_or(0)), line_num, tok_col));
                    }
                }
            }

            ',' => { chars.next(); col += 1; result.push(Spanned::new(Token::Comma, line_num, tok_col)); }

            '"' => {
                chars.next(); col += 1;
                let mut s = String::new();
                for c in chars.by_ref() {
                    col += 1;
                    if c == '"' { break; }
                    s.push(c);
                }
                result.push(Spanned::new(Token::StrLit(s), line_num, tok_col));
            }

            '0'..='9' => {
                let mut num = String::new();
                let mut has_dot = false;
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() { num.push(c); chars.next(); col += 1; }
                    else if c == '.' && !has_dot { has_dot = true; num.push(c); chars.next(); col += 1; }
                    else { break; }
                }
                if has_dot {
                    result.push(Spanned::new(Token::FloatLit(num.parse().unwrap_or(0.0)), line_num, tok_col));
                } else {
                    result.push(Spanned::new(Token::IntLit(num.parse().unwrap_or(0)), line_num, tok_col));
                }
            }

            c if c.is_alphabetic() || c == '_' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' { word.push(c); chars.next(); col += 1; }
                    else { break; }
                }
                result.push(Spanned::new(keyword_or_ident(word), line_num, tok_col));
            }

            _ => { chars.next(); col += 1; }
        }
    }
}

fn keyword_or_ident(word: String) -> Token {
    match word.as_str() {
        "testament"  => Token::Testament,
        "revealing"  => Token::Revealing,
        "scripture"  => Token::Scripture,
        "sin"        => Token::Sin,
        "covenant"   => Token::Covenant,
        "salm"       => Token::Salm,
        "upon"       => Token::Upon,
        "receiving"  => Token::Receiving,
        "reveals"    => Token::Reveals,
        "let"        => Token::Let,
        "there"      => Token::There,
        "be"         => Token::Be,
        "of"         => Token::Of,
        "become"     => Token::Become,
        "hail"       => Token::Hail,
        "praying"    => Token::Praying,
        "reveal"     => Token::Reveal,
        "whether"    => Token::Whether,
        "otherwise"  => Token::Otherwise,
        "so"         => Token::So,
        "litany"     => Token::Litany,
        "for"        => Token::For,
        "confess"    => Token::Confess,
        "answer"     => Token::Answer,
        "absolve"    => Token::Absolve,
        "as"         => Token::As,
        "transgress" => Token::Transgress,
        "manifest"   => Token::Manifest,
        "from"       => Token::From,
        "its"        => Token::Its,
        "discern"    => Token::Discern,
        "amen"       => Token::Amen,
        "forsake"    => Token::Forsake,
        "ascend"     => Token::Ascend,
        "bearing"    => Token::Bearing,
        "after"      => Token::After,
        "thus"       => Token::Thus,
        "plus"       => Token::Plus,
        "minus"      => Token::Minus,
        "times"      => Token::Times,
        "over"       => Token::Over,
        "remainder"  => Token::Remainder,
        "negate"     => Token::Negate,
        "is"         => Token::Is,
        "not"        => Token::Not,
        "greater"    => Token::Greater,
        "lesser"     => Token::Lesser,
        "than"       => Token::Than,
        "no"         => Token::No,
        "blessed"    => Token::Blessed,
        "forsaken"   => Token::Forsaken,
        "and"        => Token::And,
        "void"       => Token::Void,
        "atom"       => Token::Atom,
        "fractional" => Token::Fractional,
        "word"       => Token::Word,
        "dogma"      => Token::Dogma,
        _            => Token::Ident(word),
    }
}
