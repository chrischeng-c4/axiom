//! Identifier casing and collision-safe naming for generated TypeScript.

use std::collections::BTreeSet;

/// Split an arbitrary spec key into word tokens on non-alphanumeric
/// boundaries and camelCase humps.
fn words(input: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            if ch.is_uppercase() && prev_lower && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(ch);
            prev_lower = ch.is_lowercase() || ch.is_numeric();
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
            prev_lower = false;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
        None => String::new(),
    }
}

/// PascalCase a spec key (`pet_category` -> `PetCategory`). Falls back to a
/// leading underscore when the result would not start with a letter.
pub fn to_pascal(input: &str) -> String {
    let mut s: String = words(input).iter().map(|w| capitalize(w)).collect();
    if s.is_empty() {
        s = "Anonymous".to_string();
    }
    if !s.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
        s.insert(0, '_');
    }
    s
}

/// camelCase a spec key (`PetCategory` -> `petCategory`).
pub fn to_camel(input: &str) -> String {
    let pascal = to_pascal(input);
    let mut chars = pascal.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => pascal,
    }
}

/// snake_case a spec key (`PetCategory` -> `pet_category`). Used for Python and
/// Rust function/parameter identifiers. Falls back to a leading underscore when
/// the result would not start with a letter or underscore.
pub fn to_snake(input: &str) -> String {
    let s = words(input)
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_");
    if s.is_empty() {
        "field".to_string()
    } else if !s
        .chars()
        .next()
        .map(|c| c.is_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        format!("_{s}")
    } else {
        s
    }
}

/// True when `s` is a valid bare TypeScript object-property / identifier.
pub fn is_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Object-literal property key: bare when a valid identifier, else quoted.
pub fn prop_key(key: &str) -> String {
    if is_ident(key) {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Member access against a `params` object: dotted for identifiers, bracketed
/// (string-keyed) otherwise.
pub fn param_access(name: &str) -> String {
    if is_ident(name) {
        format!("params.{name}")
    } else {
        format!(
            "params[\"{}\"]",
            name.replace('\\', "\\\\").replace('"', "\\\"")
        )
    }
}

/// Allocates unique names, suffixing `_2`, `_3`, ... on collision.
#[derive(Debug, Default)]
pub struct NameRegistry {
    used: BTreeSet<String>,
}

impl NameRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn unique(&mut self, base: &str) -> String {
        let base = if base.is_empty() { "anonymous" } else { base };
        if self.used.insert(base.to_string()) {
            return base.to_string();
        }
        let mut n = 2;
        loop {
            let candidate = format!("{base}_{n}");
            if self.used.insert(candidate.clone()) {
                return candidate;
            }
            n += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_and_camel() {
        assert_eq!(to_pascal("pet_category"), "PetCategory");
        assert_eq!(to_pascal("pet-category"), "PetCategory");
        assert_eq!(to_camel("PetCategory"), "petCategory");
        assert_eq!(to_camel("list_pets"), "listPets");
    }

    #[test]
    fn pascal_handles_leading_non_alpha() {
        assert_eq!(to_pascal("123abc"), "_123abc");
    }

    #[test]
    fn registry_suffixes_on_collision() {
        let mut reg = NameRegistry::new();
        assert_eq!(reg.unique("Pet"), "Pet");
        assert_eq!(reg.unique("Pet"), "Pet_2");
        assert_eq!(reg.unique("Pet"), "Pet_3");
    }

    #[test]
    fn ident_detection() {
        assert!(is_ident("petId"));
        assert!(!is_ident("X-Request-Id"));
        assert_eq!(prop_key("X-Request-Id"), "\"X-Request-Id\"");
        assert_eq!(prop_key("petId"), "petId");
    }
}
