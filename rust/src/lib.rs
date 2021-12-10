#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

mod ast;
pub mod parse;
pub mod pretty;

pub use crate::ast::*;
use im::{vector, Vector};

pub type Arena = typed_arena::Arena<Term>;
pub type Rodeo = lasso::ThreadedRodeo;

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
pub fn rewrite(
    arena: &Arena,
    rodeo: &Rodeo,
    rules: &Vector<Rule>,
    original: Vector<Term>,
) -> Vector<Term> {
    let copy_spur = rodeo.get_or_intern("+");
    let discard_spur = rodeo.get_or_intern("-");
    let swap_spur = rodeo.get_or_intern("~");
    let combine_spur = rodeo.get_or_intern(",");
    let unwrap_spur = rodeo.get_or_intern("<");
    let wrap_spur = rodeo.get_or_intern(">");
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
        if index > 0 && list[index].is_word(unwrap_spur) {
            if let Term::Quote(a) = list[index - 1].clone() {
                list.remove(index); // '<'
                list.remove(index - 1); // quote;
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
        if index > 0 && list[index].is_word(wrap_spur) && list[index - 1].is_quote() {
            list.remove(index); // 'wrap'
            list[index - 1] = Term::make_quote(arena, vector![list[index - 1].clone()]).clone();
            index = list.len() - 1;
            continue;
        }
        if index > 1 && list[index].is_word(combine_spur) {
            if let Term::Quote(a) = list[index - 1].clone() {
                if let Term::Quote(b) = list[index - 2].clone() {
                    let mut new_quote = b.clone();
                    new_quote.extend(a);
                    list.remove(index); // ','
                    list.remove(index - 1); // a
                    list[index - 2] = Term::make_quote(arena, new_quote).clone();
                    index = list.len() - 1;
                    continue;
                }
            }
        }
        if index > 1
            && list[index].is_word(swap_spur)
            && list[index - 1].is_quote()
            && list[index - 2].is_quote()
        {
            list.swap(index - 2, index - 1);
            list.remove(index); // '~'
            index = list.len() - 1;
            continue;
        }
        if index > 0 && list[index].is_word(discard_spur) && list[index - 1].is_quote() {
            list.remove(index); // '-'
            list.remove(index - 1); // term
            if list.is_empty() {
                return vector![];
            }
            index = list.len() - 1;
            continue;
        }
        if index > 0 && list[index].is_word(copy_spur) && list[index - 1].is_quote() {
            list[index] = list[index - 1].clone();
            index = list.len() - 1;
            continue;
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

    fn rewrites_to(arena: &Arena, rodeo: &Rodeo, rules: Vector<Rule>, begin: &str, end: &str) {
        let begin_terms = parse::terms(arena, rodeo, begin).unwrap();
        let rewritten = rewrite(arena, rodeo, &rules, begin_terms);
        assert_eq!(pretty::terms(&rodeo, rewritten), end.to_owned());
    }

    #[test]
    fn copy_test() {
        let arena = Arena::new();
        let rodeo = Rodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "(x) +", "(x) (x)");
        rewrites_to(&arena, &rodeo, vector![], "(x) + (y)", "(x) (x) (y)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) +", "(x) (y) (y)");
        rewrites_to(
            &arena,
            &rodeo,
            vector![],
            "(x) (y) + (z)",
            "(x) (y) (y) (z)",
        );
        rewrites_to(&arena, &rodeo, vector![], "+", "+");
        rewrites_to(&arena, &rodeo, vector![], "x +", "x +");
    }

    #[test]
    fn swap_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "(x) (y) ~", "(y) (x)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (z) (y) ~", "(x) (y) (z)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) ~ (z)", "(y) (x) (z)");

        rewrites_to(&arena, &rodeo, vector![], "~", "~");
        rewrites_to(&arena, &rodeo, vector![], "x ~", "x ~");
        rewrites_to(&arena, &rodeo, vector![], "x y ~", "x y ~");
        rewrites_to(&arena, &rodeo, vector![], "(x) ~", "(x) ~");
    }

    #[test]
    fn discard_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "(x) -", "");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) -", "(x)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) - (z)", "(x) (z)");

        rewrites_to(&arena, &rodeo, vector![], "-", "-");
        rewrites_to(&arena, &rodeo, vector![], "x -", "x -");
    }

    #[test]
    fn wrap_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "(x) >", "((x))");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) >", "(x) ((y))");
        rewrites_to(&arena, &rodeo, vector![], "(x) > (y)", "((x)) (y)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) > (z)", "(x) ((y)) (z)");

        rewrites_to(&arena, &rodeo, vector![], ">", ">");
        rewrites_to(&arena, &rodeo, vector![], "x >", "x >");
    }

    #[test]
    fn unwrap_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "() <", "");
        rewrites_to(&arena, &rodeo, vector![], "(x) () <", "(x)");
        rewrites_to(&arena, &rodeo, vector![], "() < (y)", "(y)");
        rewrites_to(&arena, &rodeo, vector![], "(x) () < (y)", "(x) (y)");
        rewrites_to(&arena, &rodeo, vector![], "(y) <", "y");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) <", "(x) y");
        rewrites_to(&arena, &rodeo, vector![], "(x) < (y)", "x (y)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) < (z)", "(x) y (z)");
        rewrites_to(&arena, &rodeo, vector![], "(x y z) <", "x y z");

        rewrites_to(&arena, &rodeo, vector![], "<", "<");
        rewrites_to(&arena, &rodeo, vector![], "x <", "x <");
    }

    #[test]
    fn combine_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();

        rewrites_to(&arena, &rodeo, vector![], "() () ,", "()");
        rewrites_to(&arena, &rodeo, vector![], "(x) () ,", "(x)");
        rewrites_to(&arena, &rodeo, vector![], "() (y) ,", "(y)");
        rewrites_to(&arena, &rodeo, vector![], "(x) (y) ,", "(x y)");

        rewrites_to(&arena, &rodeo, vector![], ",", ",");
        rewrites_to(&arena, &rodeo, vector![], "(x) ,", "(x) ,");
        rewrites_to(&arena, &rodeo, vector![], "x ,", "x ,");
        rewrites_to(&arena, &rodeo, vector![], "x y ,", "x y ,");
    }

    #[test]
    fn user_defined_test() {
        let arena = Arena::new();
        let rodeo = ThreadedRodeo::new();
        let rules = parse::rules(&arena, &rodeo, "x = y z ; x x = aaaaaaaaa ;").unwrap();
        rewrites_to(&arena, &rodeo, rules.clone(), "x", "y z");
        rewrites_to(&arena, &rodeo, rules.clone(), "a x", "a y z");
        rewrites_to(&arena, &rodeo, rules.clone(), "x a", "y z a");
        rewrites_to(&arena, &rodeo, rules.clone(), "a x b", "a y z b");
        rewrites_to(&arena, &rodeo, rules.clone(), "x x", "aaaaaaaaa");
    }
}
