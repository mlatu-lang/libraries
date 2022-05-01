#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::cargo
)]

mod ast;
pub mod parse;
pub mod pretty;

pub use crate::ast::*;
use im::{vector, Vector};

/// Rewrites a given sequence of terms with the given rules into a new sequence of rules
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn rewrite(engine: &Engine, rules: &Vector<Rule>, terms: Vector<Term>) -> Vector<Term> {
    let mut start = 0;
    while start < terms.len() {
        let skipped = terms.skip(start);
        let mut length = skipped.len();
        while length > 0 {
            let pattern = &skipped.take(length);
            for rule in rules {
                if &rule.redex == pattern {
                    return rewrite(
                        engine,
                        rules,
                        terms.take(start) + rule.reduction.clone() + skipped.skip(length),
                    );
                }
            }
            if length == 2 {
                match skipped[1] {
                    Term::Prim(Primitive::Unwrap) => {
                        if let Term::Quote(a) = skipped[0].clone() {
                            return rewrite(
                                engine,
                                rules,
                                terms.take(start) + a + skipped.skip(length),
                            );
                        }
                    }
                    Term::Prim(Primitive::Wrap) => {
                        if skipped[0].is_quote() {
                            let mut new_terms = terms.take(start);
                            new_terms.push_back(
                                Term::make_quote(engine, vector![skipped[0].clone()]).clone(),
                            );
                            new_terms.append(skipped.skip(length));
                            return rewrite(engine, rules, new_terms);
                        }
                    }
                    Term::Prim(Primitive::Discard) => {
                        if skipped[0].is_quote() {
                            let new_terms = terms.take(start) + skipped.skip(length);
                            return rewrite(engine, rules, new_terms);
                        }
                    }
                    Term::Prim(Primitive::Copy) => {
                        if skipped[0].is_quote() {
                            let mut new_terms = terms.take(start + 1);
                            new_terms.push_back(skipped[0].clone());
                            new_terms.append(skipped.skip(length));
                            return rewrite(engine, rules, new_terms);
                        }
                    }
                    _ => {}
                }
            }
            if length == 3 {
                match skipped[2] {
                    Term::Prim(Primitive::Combine) => {
                        if let Term::Quote(a) = skipped[0].clone() {
                            if let Term::Quote(b) = skipped[1].clone() {
                                let mut new_terms = terms.take(start);
                                new_terms.push_back(Term::make_quote(engine, a + b).clone());
                                new_terms.append(skipped.skip(length));
                                return rewrite(engine, rules, new_terms);
                            }
                        }
                    }
                    Term::Prim(Primitive::Swap) => {
                        if skipped[0].is_quote() && skipped[1].is_quote() {
                            let mut new_terms = terms.take(start);
                            new_terms.push_back(skipped[1].clone());
                            new_terms.push_back(skipped[0].clone());
                            new_terms.append(skipped.skip(length));
                            return rewrite(engine, rules, new_terms);
                        }
                    }
                    _ => {}
                }
            }
            length -= 1;
        }
        start += 1;
    }
    terms
}

#[cfg(test)]
mod tests {
    use super::*;
    use im::vector;

    fn rewrites_to(engine: &Engine, rules: Vector<Rule>, begin: &str, end: &str) {
        let begin_terms = parse::terms(engine, begin).unwrap();
        let rewritten = rewrite(engine, &rules, begin_terms);
        assert_eq!(pretty::terms(engine, rewritten), end.to_owned());
    }

    #[test]
    fn copy_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "(x) +", "(x) (x)");
        rewrites_to(&engine, vector![], "(x) + (y)", "(x) (x) (y)");
        rewrites_to(&engine, vector![], "(x) (y) +", "(x) (y) (y)");
        rewrites_to(&engine, vector![], "(x) (y) + (z)", "(x) (y) (y) (z)");
        rewrites_to(&engine, vector![], "+", "+");
        rewrites_to(&engine, vector![], "x +", "x +");
    }

    #[test]
    fn swap_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "(x) (y) ~", "(y) (x)");
        rewrites_to(&engine, vector![], "(x) (z) (y) ~", "(x) (y) (z)");
        rewrites_to(&engine, vector![], "(x) (y) ~ (z)", "(y) (x) (z)");

        rewrites_to(&engine, vector![], "~", "~");
        rewrites_to(&engine, vector![], "x ~", "x ~");
        rewrites_to(&engine, vector![], "x y ~", "x y ~");
        rewrites_to(&engine, vector![], "(x) ~", "(x) ~");
    }

    #[test]
    fn discard_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "(x) -", "");
        rewrites_to(&engine, vector![], "(x) (y) -", "(x)");
        rewrites_to(&engine, vector![], "(x) (y) - (z)", "(x) (z)");

        rewrites_to(&engine, vector![], "-", "-");
        rewrites_to(&engine, vector![], "x -", "x -");
    }

    #[test]
    fn wrap_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "(x) >", "((x))");
        rewrites_to(&engine, vector![], "(x) (y) >", "(x) ((y))");
        rewrites_to(&engine, vector![], "(x) > (y)", "((x)) (y)");
        rewrites_to(&engine, vector![], "(x) (y) > (z)", "(x) ((y)) (z)");

        rewrites_to(&engine, vector![], ">", ">");
        rewrites_to(&engine, vector![], "x >", "x >");
    }

    #[test]
    fn unwrap_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "() <", "");
        rewrites_to(&engine, vector![], "(x) () <", "(x)");
        rewrites_to(&engine, vector![], "() < (y)", "(y)");
        rewrites_to(&engine, vector![], "(x) () < (y)", "(x) (y)");
        rewrites_to(&engine, vector![], "(y) <", "y");
        rewrites_to(&engine, vector![], "(x) (y) <", "(x) y");
        rewrites_to(&engine, vector![], "(x) < (y)", "x (y)");
        rewrites_to(&engine, vector![], "(x) (y) < (z)", "(x) y (z)");
        rewrites_to(&engine, vector![], "(x y z) <", "x y z");

        rewrites_to(&engine, vector![], "<", "<");
        rewrites_to(&engine, vector![], "x <", "x <");
    }

    #[test]
    fn combine_test() {
        let engine = Engine::new();

        rewrites_to(&engine, vector![], "() () ,", "()");
        rewrites_to(&engine, vector![], "(x) () ,", "(x)");
        rewrites_to(&engine, vector![], "() (y) ,", "(y)");
        rewrites_to(&engine, vector![], "(x) (y) ,", "(x y)");

        rewrites_to(&engine, vector![], ",", ",");
        rewrites_to(&engine, vector![], "(x) ,", "(x) ,");
        rewrites_to(&engine, vector![], "x ,", "x ,");
        rewrites_to(&engine, vector![], "x y ,", "x y ,");
    }

    #[test]
    fn user_defined_test() {
        let engine = Engine::new();
        let rules = parse::rules(&engine, "x = y z. x x = aaaaaaaaa.").unwrap();

        rewrites_to(&engine, rules.clone(), "x", "y z");
        rewrites_to(&engine, rules.clone(), "a x", "a y z");
        rewrites_to(&engine, rules.clone(), "x a", "y z a");
        rewrites_to(&engine, rules.clone(), "a x b", "a y z b");
        rewrites_to(&engine, rules.clone(), "x x", "aaaaaaaaa");
    }
}
