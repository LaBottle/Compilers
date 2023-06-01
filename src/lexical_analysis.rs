use std::{
    fmt::{self},
    fs,
};

pub fn lexical_analysis(path: &str) -> Vec<Word> {
    let s = fs::read_to_string(path).unwrap();
    let s = s.as_bytes();
    const KEYWORDS: [&str; 18] = [
        "program",
        "var",
        "integer",
        "float",
        "procedure",
        "begin",
        "end",
        "read",
        "write",
        "if",
        "then",
        "else",
        "fi",
        "while",
        "do",
        "endwh",
        "and",
        "or",
    ];

    let mut r = 1usize;
    let mut c = 0;
    let mut output: Vec<Word> = Vec::new();
    let mut i = 0;
    while i < s.len() {
        c += 1;
        if s[i] == b'\r' || s[i] == b'\t' || s[i] == b' ' {
            i += 1;
            continue;
        } else if s[i] == b'\n' {
            r += 1;
            c = 0;
        } else if is_letter_u8(s[i]) {
            let mut ty = Type::Identifier;
            let mut word = (s[i] as char).to_string();
            let mut j = i + 1;
            while j != s.len() && (is_letter_u8(s[j]) || is_number_u8(s[j])) {
                word.push(s[j] as char);
                j += 1;
            }
            if KEYWORDS.contains(&word.as_str()) {
                ty = Type::Keyword;
            }
            j -= 1;

            output.push(Word::new(ty, word, r, c));
            c += j - i;
            i = j;
        } else if is_number_u8(s[i]) {
            let mut ty = Type::Integer;
            let mut word = (s[i] as char).to_string();
            let mut j = i + 1;
            while j != s.len() && is_number_u8(s[j]) {
                word.push(s[j] as char);
                j += 1;
            }
            if j != s.len() && s[j] == b'.' {
                ty = Type::FloatPoint;
                word.push('.');
                j += 1;
                while j != s.len() && is_number_u8(s[j]) {
                    word.push(s[j] as char);
                    j += 1;
                }
            }
            j -= 1;

            output.push(Word::new(ty, word, r, c));
            c += j - i;
            i = j;
        } else if s[i] == b'\"' {
            let ty = Type::String;
            let mut word = (s[i] as char).to_string();
            let mut j = i + 1;
            while j != s.len() && s[j] != b'\"' {
                word.push(s[j] as char);
                j += 1;
            }
            word.push(s[j] as char);

            output.push(Word::new(ty, word, r, c));
            c += j - i;
            i = j;
        } else if s[i] == b'\'' {
            let ty = Type::Character;
            let mut word = (s[i] as char).to_string();
            let mut j = i + 1;
            //if s[j] == b'\\'  {
            //    word.push(s[j] as char);
            //    j += 1;
            //}
            word.push(s[j] as char);
            j += 1;
            word.push(s[j] as char);

            output.push(Word::new(ty, word, r, c));
            c += j - i;
            i = j;
        } else if is_separator_u8(s[i]) {
            let ty = Type::Separator;
            let word = (s[i] as char).to_string();

            output.push(Word::new(ty, word, r, c));
        } else if is_operator_u8(s[i]) {
            let ty = Type::Operator;
            let mut word = (s[i] as char).to_string();
            let mut j = i + 1;
            if j != s.len() && s[j] == b'=' {
                if s[i] == b'=' || s[i] == b'<' || s[i] == b'>' {
                    word.push(s[j] as char);
                    j += 1;
                }
            } else if j != s.len() && s[j] == b'>' {
                if s[i] == b'<' {
                    word.push(s[j] as char);
                    j += 1;
                }
            } else if j != s.len() && s[j] == b'/' {
                // Single-Line Comment
                if s[i] == b'/' {
                    i += 2;
                    while i < s.len() && s[i] != b'\n' {
                        i += 1;
                    }

                    i += 1;
                    r += 1;
                    c = 0;
                    continue;
                }
            } else if s[j] == b'*' {
                // Multi-Line Comment
                if s[i] == b'/' {
                    i += 2;
                    while i < s.len() && s[i] != b'/' || s[i - 1] != b'*' {
                        i += 1;
                        if s[i] != b'\n' {
                            c += 1;
                        } else {
                            r += 1;
                            c = 1;
                        }
                    }

                    i += 1;
                    continue;
                }
            }

            j -= 1;
            output.push(Word::new(ty, word, r, c));
            c += j - i;
            i = j;
        } else {
            output.push(Word::new(Type::Error, "_Error".to_string(), r, c));
        }
        i += 1;
    }

    output
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Type {
    Error,
    Keyword = 1,
    Identifier,
    Integer,
    FloatPoint,
    String,
    Character,
    Separator,
    Operator,
}

impl Copy for Type {}

// impl ToString for Type{
//     fn to_string(&self) -> String {
//         match self {
//             Type::Error => "Type::Error".to_string(),
//             Type::Keyword => "Type::Keyword".to_string(),
//             Type::Identifier => "Type::Identifier".to_string(),
//             Type::Integer => "Type::Integer".to_string(),
//             Type::FloatPoint => "Type::FloatPoint".to_string(),
//             Type::String => "Type::String".to_string(),
//             Type::Character => "Type::Character".to_string(),
//             Type::Separator => "Type::Separator".to_string(),
//             Type::Operator => "Type::Operator".to_string(),
//         }
//     }
// }

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Error => "Type::Error",
                Type::Keyword => "Type::Keyword",
                Type::Identifier => "Type::Identifier",
                Type::Integer => "Type::Integer",
                Type::FloatPoint => "Type::FloatPoint",
                Type::String => "Type::String",
                Type::Character => "Type::Character",
                Type::Separator => "Type::Separator",
                Type::Operator => "Type::Operator",
            }
        )
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Word {
    ty: Type,
    val: String,
    row: usize,
    col: usize,
}
impl Word {
    pub fn new(ty: Type, val: String, row: usize, col: usize) -> Self {
        Word { ty, val, row, col }
    }
}

fn is_letter_u8(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'$'
}

fn is_number_u8(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn is_separator_u8(c: u8) -> bool {
    const SEPARATOR: [u8; 6] = [b'(', b')', b'{', b'}', b';', b','];
    SEPARATOR.contains(&c)
}

fn is_operator_u8(c: u8) -> bool {
    const OPERATOR: [u8; 7] = [b'+', b'-', b'*', b'/', b'=', b'<', b'>'];
    OPERATOR.contains(&c)
}
