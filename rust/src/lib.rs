#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![forbid(unsafe_code)]

mod ast;
pub mod parse;
pub mod pretty;

pub use crate::ast::*;

/// Rewrites a given sequence of terms with the given rules into a new sequence of rules
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn rewrite(rules: &[Rule], mut list: Vec<Term>) -> Terms {
    let mut index = list.len() - 1;
    loop {
        let mut matching = vec![];
        for rule in rules.to_owned() {
            let mut depth = 0;
            let mut got_to_end = true;
            for redex_term in rule.redex {
                if index >= depth && redex_term == list[index - depth] {
                    depth += 1;
                } else {
                    got_to_end = false;
                    break;
                }
            }
            if got_to_end {
                matching.push((depth - 1, rule.reduction.clone()));
            }
        }
        if let Some((depth, reduction)) =
            matching.into_iter().reduce(|(depth1, r1), (depth2, r2)| {
                if depth1 > depth2 {
                    (depth1, r1)
                } else {
                    (depth2, r2)
                }
            })
        {
            for _ in 0..=depth {
                list.remove(index - depth);
            }
            for reduction_term in reduction.iter().rev() {
                list.insert(index - depth, reduction_term.clone());
            }
            if list.is_empty() {
                return list;
            }
            index = list.len() - 1;
            continue;
        }
        if index > 0 && list[index].is_word("<") {
            if let Term::Quote(a) = list[index - 1].clone() {
                list.remove(index); // 'u'
                list.remove(index - 1); // quote;
                for term in a.into_iter().rev() {
                    list.insert(index - 1, term.clone());
                }
                if list.is_empty() {
                    return list;
                }
                index = list.len() - 1;
                continue;
            }
        }
        if index > 0 && list[index].is_word(">") {
            if let Term::Quote(a) = list[index - 1].clone() {
                list.remove(index); // 'wrap'
                list[index - 1] = Term::Quote(vec![Term::Quote(a)]);
                index = list.len() - 1;
                continue;
            }
        }
        if index > 1 && list[index].is_word(",") {
            if let Term::Quote(a) = list[index - 1].clone() {
                if let Term::Quote(b) = list[index - 2].clone() {
                    let mut new_quote = b.clone();
                    new_quote.append(&mut a.clone());
                    list.remove(index); // 'c'
                    list.remove(index - 1); // a
                    list[index - 2] = Term::make_quote(new_quote);
                    index = list.len() - 1;
                    continue;
                }
            }
        }
        if index > 1 && list[index].is_word("~") {
            if let Term::Quote(a) = list[index - 1].clone() {
                if let Term::Quote(b) = list[index - 2].clone() {
                    list[index - 1] = Term::Quote(b.clone());
                    list[index - 2] = Term::Quote(a.clone());
                    list.remove(index); // 'swap'
                    index = list.len() - 1;
                    continue;
                }
            }
        }
        if index > 0 && list[index].is_word("-") {
            if let Term::Quote(_) = list[index - 1].clone() {
                list.remove(index); // 'discard'
                list.remove(index - 1); // term
                if list.is_empty() {
                    return list;
                }
                index = list.len() - 1;
                continue;
            }
        }
        if index > 0 && list[index].is_word("+") {
            if let Term::Quote(a) = list[index - 1].clone() {
                list[index] = Term::Quote(a.clone());
                index = list.len() - 1;
                continue;
            }
        }
        if index == 0 {
            return list;
        }
        index -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrites_to(rules: &[Rule], begin: &str, end: &str) {
        assert_eq!(
            rewrite(rules, parse::terms(begin).unwrap()),
            parse::terms(end).unwrap()
        )
    }

    #[test]
    fn copy_test() {
        rewrites_to(&[], "(x) +", "(x) (x)");
        rewrites_to(&[], "(x) + (y)", "(x) (x) (y)");
        rewrites_to(&[], "(x) (y) +", "(x) (y) (y)");
        rewrites_to(&[], "(x) (y) + (z)", "(x) (y) (y) (z)");

        rewrites_to(&[], "+", "+");
        rewrites_to(&[], "x +", "x +");
    }

    #[test]
    fn swap_test() {
        rewrites_to(&[], "(x) (y) ~", "(y) (x)");
        rewrites_to(&[], "(x) (z) (y) ~", "(x) (y) (z)");
        rewrites_to(&[], "(x) (y) ~ (z)", "(y) (x) (z)");

        rewrites_to(&[], "~", "~");
        rewrites_to(&[], "x ~", "x ~");
        rewrites_to(&[], "x y ~", "x y ~");
        rewrites_to(&[], "(x) ~", "(x) ~");
    }

    #[test]
    fn discard_test() {
        rewrites_to(&[], "(x) -", "");
        rewrites_to(&[], "(x) (y) -", "(x)");
        rewrites_to(&[], "(x) (y) - (z)", "(x) (z)");

        rewrites_to(&[], "-", "-");
        rewrites_to(&[], "x -", "x -");
    }

    #[test]
    fn wrap_test() {
        rewrites_to(&[], "(x) >", "((x))");
        rewrites_to(&[], "(x) (y) >", "(x) ((y))");
        rewrites_to(&[], "(x) > (y)", "((x)) (y)");
        rewrites_to(&[], "(x) (y) > (z)", "(x) ((y)) (z)");

        rewrites_to(&[], ">", ">");
        rewrites_to(&[], "x >", "x >");
    }

    #[test]
    fn unwrap_test() {
        rewrites_to(&[], "() <", "");
        rewrites_to(&[], "(x) () <", "(x)");
        rewrites_to(&[], "() < (y)", "(y)");
        rewrites_to(&[], "(x) () < (y)", "(x) (y)");
        rewrites_to(&[], "(y) <", "y");
        rewrites_to(&[], "(x) (y) <", "(x) y");
        rewrites_to(&[], "(x) < (y)", "x (y)");
        rewrites_to(&[], "(x) (y) < (z)", "(x) y (z)");
        rewrites_to(&[], "(x y z) <", "x y z");

        rewrites_to(&[], "<", "<");
        rewrites_to(&[], "x <", "x <");
    }

    #[test]
    fn combine_test() {
        rewrites_to(&[], "() () ,", "()");
        rewrites_to(&[], "(x) () ,", "(x)");
        rewrites_to(&[], "() (y) ,", "(y)");
        rewrites_to(&[], "(x) (y) ,", "(x y)");

        rewrites_to(&[], ",", ",");
        rewrites_to(&[], "(x) ,", "(x) ,");
        rewrites_to(&[], "x ,", "x ,");
        rewrites_to(&[], "x y ,", "x y ,");
    }

    #[test]
    fn user_defined_test() {
        let rules = parse::rules("x = y z ; x x = aaaaaaaaa ;").unwrap();
        rewrites_to(&rules, "x", "y z");
        rewrites_to(&rules, "a x", "a y z");
        rewrites_to(&rules, "x a", "y z a");
        rewrites_to(&rules, "a x b", "a y z b");
        rewrites_to(&rules, "x x", "aaaaaaaaa");
    }
}
