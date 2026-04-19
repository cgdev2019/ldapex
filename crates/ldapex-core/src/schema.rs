//! Minimal parser for RFC 4512 schema definitions.
//!
//! We do **not** aim to be a conformant parser: the goal is to extract
//! the bits the UI needs (primary `NAME`, kind, `MUST`, `MAY`, `SUP`)
//! from `attributeTypes` and `objectClasses` entries of the
//! subschema. Unknown tokens are skipped.

use crate::types::{ObjectClassDef, ObjectClassKind};

/// Extract the primary name (first `NAME 'xxx'` form, or the first name
/// from `NAME ( 'a' 'b' )`). Returns `None` if the definition has no
/// `NAME`.
#[must_use]
pub fn attribute_name(def: &str) -> Option<String> {
    let tokens = tokenize(strip_outer_parens(def)?);
    let mut it = tokens.into_iter().peekable();
    while let Some(tok) = it.next() {
        if matches!(&tok, Token::Word(w) if w.eq_ignore_ascii_case("NAME")) {
            return first_qdescr(&mut it);
        }
    }
    None
}

/// Parse an objectClass definition. Returns `None` if it lacks a name.
#[must_use]
pub fn parse_object_class(def: &str) -> Option<ObjectClassDef> {
    let tokens = tokenize(strip_outer_parens(def)?);
    let mut it = tokens.into_iter().peekable();
    let mut name: Option<String> = None;
    let mut kind = ObjectClassKind::Structural;
    let mut sup: Vec<String> = Vec::new();
    let mut must: Vec<String> = Vec::new();
    let mut may: Vec<String> = Vec::new();

    while let Some(tok) = it.next() {
        let Token::Word(w) = tok else { continue };
        match w.to_ascii_uppercase().as_str() {
            "NAME" => name = first_qdescr(&mut it),
            "SUP" => sup = read_oids(&mut it),
            "MUST" => must = read_oids(&mut it),
            "MAY" => may = read_oids(&mut it),
            "STRUCTURAL" => kind = ObjectClassKind::Structural,
            "AUXILIARY" => kind = ObjectClassKind::Auxiliary,
            "ABSTRACT" => kind = ObjectClassKind::Abstract,
            _ => {}
        }
    }

    Some(ObjectClassDef {
        name: name?,
        kind,
        sup,
        must,
        may,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    LParen,
    RParen,
    Dollar,
    Word(String),
    Quoted(String),
}

fn strip_outer_parens(s: &str) -> Option<&str> {
    let t = s.trim();
    let t = t.strip_prefix('(')?.trim();
    let t = t.strip_suffix(')')?.trim();
    Some(t)
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '(' => {
                out.push(Token::LParen);
                chars.next();
            }
            ')' => {
                out.push(Token::RParen);
                chars.next();
            }
            '$' => {
                out.push(Token::Dollar);
                chars.next();
            }
            '\'' => {
                chars.next();
                let mut buf = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '\'' {
                        chars.next();
                        break;
                    }
                    buf.push(ch);
                    chars.next();
                }
                out.push(Token::Quoted(buf));
            }
            _ => {
                let mut buf = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || matches!(ch, '(' | ')' | '$' | '\'') {
                        break;
                    }
                    buf.push(ch);
                    chars.next();
                }
                if !buf.is_empty() {
                    out.push(Token::Word(buf));
                }
            }
        }
    }
    out
}

/// Read one quoted descriptor, potentially wrapped in parentheses. In
/// `NAME ( 'a' 'b' )` we only keep the first alias.
fn first_qdescr<I>(it: &mut std::iter::Peekable<I>) -> Option<String>
where
    I: Iterator<Item = Token>,
{
    match it.next()? {
        Token::Quoted(s) => Some(s),
        Token::LParen => {
            let mut first: Option<String> = None;
            for tok in it.by_ref() {
                match tok {
                    Token::Quoted(s) if first.is_none() => first = Some(s),
                    Token::RParen => break,
                    _ => {}
                }
            }
            first
        }
        _ => None,
    }
}

/// Read either `word` or `( word $ word $ word )`. Returns the list of
/// words (without quotes).
fn read_oids<I>(it: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = Token>,
{
    match it.next() {
        Some(Token::Word(w) | Token::Quoted(w)) => vec![w],
        Some(Token::LParen) => {
            let mut out = Vec::new();
            for tok in it.by_ref() {
                match tok {
                    Token::Word(w) | Token::Quoted(w) => out.push(w),
                    Token::Dollar | Token::LParen => {}
                    Token::RParen => break,
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_name_extracts_primary() {
        let def = "( 2.5.4.3 NAME 'cn' SUP name )";
        assert_eq!(attribute_name(def).as_deref(), Some("cn"));

        let def_alias = "( 2.5.4.3 NAME ( 'cn' 'commonName' ) SUP name )";
        assert_eq!(attribute_name(def_alias).as_deref(), Some("cn"));

        assert!(attribute_name("bad").is_none());
    }

    #[test]
    fn object_class_parses_person() {
        let def = "( 2.5.6.6 NAME 'person' SUP top STRUCTURAL \
                   MUST ( sn $ cn ) \
                   MAY ( userPassword $ telephoneNumber $ seeAlso $ description ) )";
        let parsed = parse_object_class(def).expect("person parsed");
        assert_eq!(parsed.name, "person");
        assert_eq!(parsed.kind, ObjectClassKind::Structural);
        assert_eq!(parsed.sup, vec!["top".to_string()]);
        assert_eq!(parsed.must, vec!["sn".to_string(), "cn".to_string()]);
        assert!(parsed.may.contains(&"description".to_string()));
    }

    #[test]
    fn object_class_parses_auxiliary_without_may() {
        let def = "( 1.2.3 NAME 'extra' AUXILIARY MUST cn )";
        let parsed = parse_object_class(def).expect("extra parsed");
        assert_eq!(parsed.name, "extra");
        assert_eq!(parsed.kind, ObjectClassKind::Auxiliary);
        assert_eq!(parsed.must, vec!["cn".to_string()]);
        assert!(parsed.may.is_empty());
    }
}
