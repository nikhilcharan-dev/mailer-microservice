use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeSet;

/// Built-in Handlebars helpers that should not be treated as variables.
const BUILTINS: &[&str] = &[
    "if", "unless", "each", "with", "lookup", "log", "else", "this",
];

/// Extract variable identifiers from a Handlebars template.
/// Walks `{{var}}`, `{{{var}}}`, `{{#if var}}`, `{{#each var}}`, helper args.
/// This is a regex-based approximation that handles the common cases without
/// pulling in a full parser; helpers built into Handlebars are skipped.
static TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    // Match {{...}} or {{{...}}} blocks
    Regex::new(r"\{\{\{?\s*([^}]+?)\s*\}?\}\}").unwrap()
});

static IDENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());

pub fn extract_variables(template_strs: &[&str]) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for s in template_strs {
        for caps in TOKEN_RE.captures_iter(s) {
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("").trim();
            // Skip comments {{! ... }} and partials {{> ... }}
            if inner.starts_with('!') || inner.starts_with('>') {
                continue;
            }
            // Strip leading block markers: #if foo  /  /each  /  ^unless
            let cleaned = inner
                .trim_start_matches('#')
                .trim_start_matches('/')
                .trim_start_matches('^')
                .trim();
            // Tokenise to grab identifiers — pick the first one that isn't a builtin
            // helper name, then keep ALL further identifiers (helper arguments).
            let mut tokens: Vec<&str> = IDENT_RE
                .find_iter(cleaned)
                .map(|m| m.as_str())
                .collect();
            if tokens.is_empty() {
                continue;
            }
            // If the first token is a builtin helper (#if, each, ...), skip it
            // so the remaining args become the variables.
            if BUILTINS.contains(&tokens[0]) {
                tokens.remove(0);
            }
            for t in tokens {
                // Skip literals like "true" / "false" / numeric-looking
                if t == "true" || t == "false" || t.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    continue;
                }
                // Drop the path segments — we store the root identifier
                let root = t.split('.').next().unwrap_or(t);
                set.insert(root.to_string());
            }
        }
    }
    set.into_iter().collect()
}

pub fn validate_template(template: &str) -> Result<()> {
    let hbs = Handlebars::new();
    hbs.render_template(template, &serde_json::json!({}))
        .map_err(|e| anyhow!("template parse error: {e}"))?;
    Ok(())
}

pub fn render(template: &str, vars: &serde_json::Value) -> Result<String> {
    let hbs = Handlebars::new();
    Ok(hbs.render_template(template, vars)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_simple_variables() {
        let vars = extract_variables(&["Your code is {{otp}}, {{name}}!"]);
        assert_eq!(vars, vec!["name".to_string(), "otp".to_string()]);
    }

    #[test]
    fn extracts_triple_stache() {
        let vars = extract_variables(&["<a href=\"{{{resetLink}}}\">click</a>"]);
        assert_eq!(vars, vec!["resetLink".to_string()]);
    }

    #[test]
    fn skips_builtin_helpers() {
        let vars = extract_variables(&["{{#if actionLink}}go{{/if}} {{name}}"]);
        assert_eq!(vars, vec!["actionLink".to_string(), "name".to_string()]);
    }

    #[test]
    fn extracts_from_multiple_sources() {
        let vars = extract_variables(&["Hi {{name}}", "Subject: {{title}}"]);
        assert_eq!(vars, vec!["name".to_string(), "title".to_string()]);
    }
}
