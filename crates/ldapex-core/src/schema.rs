//! Minimal parser for RFC 4512 schema definitions.
//!
//! We do **not** aim to be a conformant parser: the goal is to extract
//! the bits the UI needs (primary `NAME`, kind, `MUST`, `MAY`, `SUP`)
//! from `attributeTypes` and `objectClasses` entries of the
//! subschema. Unknown tokens are skipped.

use std::collections::HashMap;

use crate::types::{AttributeTypeDef, ObjectClassDef, ObjectClassKind, ResolvedClass};

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

/// Parse an attributeTypes definition. Returns `None` if it lacks a
/// name. Captures the primary NAME, OID, aliases, SUP, syntax,
/// EQUALITY/ORDERING/SUBSTR matching rules, SINGLE-VALUE,
/// NO-USER-MODIFICATION and USAGE — i.e. everything the schema
/// explorer needs to be informative.
#[must_use]
pub fn parse_attribute_type(def: &str) -> Option<AttributeTypeDef> {
    let tokens = tokenize(strip_outer_parens(def)?);
    // The first token in an RFC 4512 definition is the OID.
    let mut it = tokens.into_iter().peekable();
    let oid = match it.peek() {
        Some(Token::Word(w)) => {
            let v = w.clone();
            it.next();
            v
        }
        _ => String::new(),
    };

    let mut name: Option<String> = None;
    let mut aliases: Vec<String> = Vec::new();
    let mut sup: Option<String> = None;
    let mut syntax: Option<String> = None;
    let mut equality: Option<String> = None;
    let mut ordering: Option<String> = None;
    let mut substring: Option<String> = None;
    let mut single_valued = false;
    let mut no_user_modification = false;
    let mut usage: Option<String> = None;

    while let Some(tok) = it.next() {
        let Token::Word(w) = tok else { continue };
        match w.to_ascii_uppercase().as_str() {
            "NAME" => {
                let names = read_descrs(&mut it);
                if !names.is_empty() {
                    name = Some(names[0].clone());
                    aliases = names.into_iter().skip(1).collect();
                }
            }
            "SUP" => sup = read_oids(&mut it).into_iter().next(),
            "SYNTAX" => {
                // SYNTAX numericoid {len}? — the curly len suffix is
                // already part of the next word.
                if let Some(Token::Word(w)) = it.next() {
                    syntax = Some(w);
                }
            }
            "EQUALITY" => equality = read_oids(&mut it).into_iter().next(),
            "ORDERING" => ordering = read_oids(&mut it).into_iter().next(),
            "SUBSTR" | "SUBSTRING" => substring = read_oids(&mut it).into_iter().next(),
            "SINGLE-VALUE" => single_valued = true,
            "NO-USER-MODIFICATION" => no_user_modification = true,
            "USAGE" => {
                if let Some(Token::Word(w)) = it.next() {
                    usage = Some(w);
                }
            }
            _ => {}
        }
    }

    Some(AttributeTypeDef {
        name: name?,
        oid,
        aliases,
        sup,
        syntax,
        equality,
        ordering,
        substring,
        single_valued,
        no_user_modification,
        usage,
    })
}

/// Resolve the effective MUST / MAY attribute lists for an
/// objectClass, walking the SUP chain. Cycles are broken; an unknown
/// SUP is silently skipped.
#[must_use]
pub fn resolve_must_may(target: &str, classes: &[ObjectClassDef]) -> Option<ResolvedClass> {
    let by_name: HashMap<String, &ObjectClassDef> = classes
        .iter()
        .map(|c| (c.name.to_ascii_lowercase(), c))
        .collect();
    let root = by_name.get(&target.to_ascii_lowercase())?;

    let mut must: Vec<String> = Vec::new();
    let mut may: Vec<String> = Vec::new();
    let mut chain: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut stack: Vec<&ObjectClassDef> = vec![*root];

    while let Some(oc) = stack.pop() {
        if !seen.insert(oc.name.to_ascii_lowercase()) {
            continue;
        }
        chain.push(oc.name.clone());
        for m in &oc.must {
            push_unique_ci(&mut must, m);
        }
        for m in &oc.may {
            push_unique_ci(&mut may, m);
        }
        for parent in &oc.sup {
            if let Some(p) = by_name.get(&parent.to_ascii_lowercase()) {
                stack.push(*p);
            }
        }
    }

    // The starting class itself is in `chain[0]`; the rest is the
    // ancestry. Drop duplicates between MUST and MAY (RFC 4512 says a
    // MUST always wins).
    may.retain(|a| !must.iter().any(|m| m.eq_ignore_ascii_case(a)));

    Some(ResolvedClass {
        name: root.name.clone(),
        kind: root.kind,
        sup_chain: chain.into_iter().skip(1).collect(),
        must,
        may,
    })
}

fn push_unique_ci(out: &mut Vec<String>, item: &str) {
    if !out.iter().any(|x| x.eq_ignore_ascii_case(item)) {
        out.push(item.to_string());
    }
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

/// Read every quoted descriptor in either `'a'` or `( 'a' 'b' 'c' )`.
fn read_descrs<I>(it: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = Token>,
{
    match it.next() {
        Some(Token::Quoted(s)) => vec![s],
        Some(Token::LParen) => {
            let mut out = Vec::new();
            for tok in it.by_ref() {
                match tok {
                    Token::Quoted(s) => out.push(s),
                    Token::RParen => break,
                    _ => {}
                }
            }
            out
        }
        _ => Vec::new(),
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

    #[test]
    fn attribute_type_parses_full_definition() {
        let def = "( 2.5.4.3 NAME ( 'cn' 'commonName' ) DESC 'RFC2256: common name(s)' \
                   SUP name SYNTAX 1.3.6.1.4.1.1466.115.121.1.15{32768} \
                   EQUALITY caseIgnoreMatch SUBSTR caseIgnoreSubstringsMatch \
                   USAGE userApplications )";
        let parsed = parse_attribute_type(def).expect("cn parsed");
        assert_eq!(parsed.name, "cn");
        assert_eq!(parsed.oid, "2.5.4.3");
        assert_eq!(parsed.aliases, vec!["commonName"]);
        assert_eq!(parsed.sup.as_deref(), Some("name"));
        assert!(parsed.syntax.unwrap().starts_with("1.3.6.1.4.1.1466"));
        assert_eq!(parsed.equality.as_deref(), Some("caseIgnoreMatch"));
        assert_eq!(
            parsed.substring.as_deref(),
            Some("caseIgnoreSubstringsMatch")
        );
        assert!(!parsed.single_valued);
        assert_eq!(parsed.usage.as_deref(), Some("userApplications"));
    }

    #[test]
    fn attribute_type_picks_up_single_value_flag() {
        let def = "( 2.5.4.13 NAME 'description' SYNTAX 1.3.6.1.4.1.1466.115.121.1.15 \
                   SINGLE-VALUE NO-USER-MODIFICATION )";
        let parsed = parse_attribute_type(def).expect("description parsed");
        assert!(parsed.single_valued);
        assert!(parsed.no_user_modification);
    }

    fn oc(name: &str, sup: &[&str], must: &[&str], may: &[&str]) -> ObjectClassDef {
        ObjectClassDef {
            name: name.into(),
            kind: ObjectClassKind::Structural,
            sup: sup.iter().map(|s| (*s).into()).collect(),
            must: must.iter().map(|s| (*s).into()).collect(),
            may: may.iter().map(|s| (*s).into()).collect(),
        }
    }

    #[test]
    fn resolve_must_may_walks_inheritance() {
        let classes = vec![
            oc("top", &[], &["objectClass"], &[]),
            oc(
                "person",
                &["top"],
                &["sn", "cn"],
                &["userPassword", "description"],
            ),
            oc(
                "organizationalPerson",
                &["person"],
                &[],
                &["title", "ou", "telephoneNumber"],
            ),
            oc(
                "inetOrgPerson",
                &["organizationalPerson"],
                &[],
                &["mail", "uid", "displayName", "description"],
            ),
        ];

        let resolved = resolve_must_may("inetOrgPerson", &classes).expect("resolved");
        assert_eq!(resolved.name, "inetOrgPerson");
        // MUST collected from ancestors.
        for must in ["objectClass", "sn", "cn"] {
            assert!(
                resolved.must.iter().any(|x| x.eq_ignore_ascii_case(must)),
                "missing MUST {must} in {:?}",
                resolved.must
            );
        }
        // MAY merged (description deduped, not in MUST).
        for may in ["userPassword", "description", "ou", "mail", "displayName"] {
            assert!(
                resolved.may.iter().any(|x| x.eq_ignore_ascii_case(may)),
                "missing MAY {may} in {:?}",
                resolved.may
            );
        }
        // SUP chain in lookup order.
        assert!(resolved
            .sup_chain
            .iter()
            .any(|x| x == "organizationalPerson"));
        assert!(resolved.sup_chain.iter().any(|x| x == "person"));
        assert!(resolved.sup_chain.iter().any(|x| x == "top"));
    }

    #[test]
    fn resolve_must_may_handles_unknown_root() {
        let classes = vec![oc("top", &[], &[], &[])];
        assert!(resolve_must_may("does-not-exist", &classes).is_none());
    }

    #[test]
    fn resolve_must_may_breaks_cycles() {
        let classes = vec![oc("a", &["b"], &["x"], &[]), oc("b", &["a"], &[], &["y"])];
        let resolved = resolve_must_may("a", &classes).expect("resolved");
        assert!(resolved.must.iter().any(|m| m == "x"));
        assert!(resolved.may.iter().any(|m| m == "y"));
    }
}
