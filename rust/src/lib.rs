#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery, clippy::cargo)]

mod ast;
pub mod parse;
pub mod pretty;

pub use crate::ast::*;
use im::{vector, Vector};

fn matches(redex: &Vector<Term>, list: &[Term], start: usize) -> bool {
    if (redex.len() - 1) > start {
        return false;
    }
    for i in 0..redex.len() {
        if list[(start - (redex.len() - 1)) + i] != redex[i] {
            return false;
        }
    }
    true
}

/// Rewrites a given sequence of terms with the given rules into a new sequence of rules
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn rewrite(engine: &Engine, rules: &Vector<Rule>, original: Vector<Term>) -> Vector<Term> {
    let mut list: Vec<_> = original.into_iter().collect();
    let mut index = list.len() - 1;
    loop {
        let mut max_depth = 0;
        let mut max_reduction = None;
        for rule in rules {
            if (rule.redex.len() - 1) >= max_depth && matches(&rule.redex, &list, index) {
                max_depth = rule.redex.len() - 1;
                max_reduction = Some(rule.reduction.clone());
            }
        }
        if let Some(reduction) = max_reduction {
            for _ in 0..=max_depth {
                list.remove(index - max_depth);
            }
            for reduction_term in reduction.into_iter().rev() {
                list.insert(index - max_depth, reduction_term.clone());
            }
            if list.is_empty() {
                return vector![];
            }
            index = list.len() - 1;
            continue;
        }
        match list.get(index) {
            Some(Term::Prim(Primitive::Unwrap)) if index > 0 => {
                if let Term::Quote(a) = list[index - 1].clone() {
                    list.remove(index); // '<'
                    list.remove(index - 1); // input quote;
                    for term in a.into_iter().rev() {
                        list.insert(index - 1, term.clone());
                    }
                    if list.is_empty() {
                        return vector![];
                    }
                    index = list.len() - 1;
                    continue;
                }
            }
            Some(Term::Prim(Primitive::Wrap)) if index > 0 => {
                if list[index - 1].is_quote() {
                    list.remove(index); // '>'
                    list[index - 1] =
                        Term::make_quote(engine, vector![list[index - 1].clone()]).clone(); // input quote
                    index = list.len() - 1;
                    continue;
                }
            }
            Some(Term::Prim(Primitive::Discard)) if index > 0 => {
                if list[index - 1].is_quote() {
                    list.remove(index); // '-'
                    list.remove(index - 1); // term
                    if list.is_empty() {
                        return vector![];
                    }
                    index = list.len() - 1;
                    continue;
                }
            }
            Some(Term::Prim(Primitive::Copy)) if index > 0 => {
                if list[index - 1].is_quote() {
                    list[index] = list[index - 1].clone();
                    index = list.len() - 1;
                    continue;
                }
            }
            Some(Term::Prim(Primitive::Combine)) if index > 1 => {
                if let Term::Quote(a) = list[index - 1].clone() {
                    if let Term::Quote(b) = list[index - 2].clone() {
                        let mut new_quote = b.clone();
                        new_quote.extend(a);
                        list.remove(index); // ','
                        list.remove(index - 1);
                        list[index - 2] = Term::make_quote(engine, new_quote).clone();
                        index = list.len() - 1;
                        continue;
                    }
                }
            }
            Some(Term::Prim(Primitive::Swap)) if index > 1 => {
                if list[index - 1].is_quote() && list[index - 2].is_quote() {
                    list.swap(index - 2, index - 1); // two input quotes
                    list.remove(index); // '~'
                    index = list.len() - 1;
                    continue;
                }
            }
            _ => {}
        }
        if index == 0 {
            return Vector::from(list);
        }
        index -= 1;
    }
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
