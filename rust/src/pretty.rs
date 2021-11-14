use crate::{Rule, Rules, Term, Terms};

/// Pretty prints a term into a string
#[must_use]
pub fn term(term: Term) -> String {
    match term {
        Term::Word(s) => s,
        Term::Quote(q) => {
            format!("({})", terms(q).trim())
        }
    }
}

/// Pretty prints a sequence of terms into a string
#[must_use]
pub fn terms(terms: Terms) -> String {
    let mut s = String::new();
    for t in terms {
        s.push_str(&term(t.clone()));
        s.push(' ');
    }
    s
}

/// Pretty prints a rule into a string
#[must_use]
pub fn rule(rule: Rule) -> String {
    format!("{} = {};", terms(rule.redex), terms(rule.reduction).trim())
}

/// Pretty prints a sequence of rules into a string
#[must_use]
pub fn rules(rules: Rules) -> String {
    let mut s = String::new();
    for r in rules {
        s.push_str(&rule(r.clone()));
        s.push('\n');
    }
    s
}
