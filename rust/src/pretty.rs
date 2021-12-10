use crate::{Rule, Term};
use im::Vector;
use lasso::{Resolver, Spur};

/// Pretty prints a term into a string
#[must_use]
pub fn term<R: Resolver<Spur>>(rodeo: &R, term: Term) -> String {
    match term {
        Term::Word(s) => rodeo.resolve(&s).to_string(),
        Term::Quote(q) => {
            format!("({})", terms(rodeo, q).trim())
        }
    }
}

/// Pretty prints a sequence of terms into a string
#[must_use]
pub fn terms<R: Resolver<Spur>>(rodeo: &R, terms: Vector<Term>) -> String {
    let mut s = String::new();
    for t in terms {
        s.push_str(&term(rodeo, t));
        s.push(' ');
    }
    s.trim().to_owned()
}

/// Pretty prints a rule into a string
#[must_use]
pub fn rule<R: Resolver<Spur>>(rodeo: &R, rule: Rule) -> String {
    let Rule { redex, reduction } = rule;
    format!(
        "{} = {};",
        terms(rodeo, redex),
        terms(rodeo, reduction).trim()
    )
}

/// Pretty prints a sequence of rules into a string
#[must_use]
pub fn rules<R: Resolver<Spur>>(rodeo: &R, rules: Vector<Rule>) -> String {
    let mut s = String::new();
    for r in rules {
        s.push_str(&rule(rodeo, r));
        s.push('\n');
    }
    s
}
