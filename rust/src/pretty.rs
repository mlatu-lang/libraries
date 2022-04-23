use crate::{Engine, Rule, Term};
use im::Vector;

/// Pretty prints a term into a string
#[must_use]
pub fn term(engine: &Engine, term: Term) -> String {
    match term {
        Term::Word(s) => engine.resolve(&s).to_string(),
        Term::Prim(primitive) => primitive.to_string(),
        Term::Quote(q) => {
            format!("({})", terms(engine, q).trim())
        }
    }
}

/// Pretty prints a sequence of terms into a string
#[must_use]
pub fn terms(engine: &Engine, terms: Vector<Term>) -> String {
    let mut s = String::new();
    for t in terms {
        s.push_str(&term(engine, t));
        s.push(' ');
    }
    s.trim().to_owned()
}

/// Pretty prints a rule into a string
#[must_use]
pub fn rule(engine: &Engine, rule: Rule) -> String {
    let Rule { redex, reduction } = rule;
    format!(
        "{} = {}.",
        terms(engine, redex),
        terms(engine, reduction).trim()
    )
}

/// Pretty prints a sequence of rules into a string
#[must_use]
pub fn rules(engine: &Engine, rules: Vector<Rule>) -> String {
    let mut s = String::new();
    for r in rules {
        s.push_str(&rule(engine, r));
        s.push('\n');
    }
    s
}
