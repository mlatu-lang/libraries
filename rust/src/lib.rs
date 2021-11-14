#![deny(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![forbid(unsafe_code)]

mod ast;
pub mod parse;
pub mod pretty;

pub use crate::ast::*;

/// Rewrites a given sequence of terms with the given rules into a new sequence of rules
#[must_use]
pub fn rewrite(rules: &[Rule], mut list: Vec<Term>) -> Terms {
    let mut index = list.len() - 1;
    loop {
        let mut matching = vec![];
        for rule in rules.to_owned() {
            let mut depth = 0;
            let mut got_to_end = true;
            for redex_term in rule.redex {
                if index >= depth && Some(&redex_term) == list.get(index - depth) {
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
        if index > 0 && Some(&Term::make_word("u")) == list.get(index) {
            let second = list.get(index - 1).cloned();
            if let Some(Term::Quote(q)) = second {
                list.remove(index); // 'u'
                list.remove(index - 1); // quote;
                for term in q.into_iter().rev() {
                    list.insert(index - 1, term);
                }
                if list.is_empty() {
                    return list;
                }
                index = list.len() - 1;
                continue;
            }
        }
        if index > 0 && Some(&Term::make_word("q")) == list.get(index) {
            list.remove(index); // 'q'
            list[index - 1] = Term::make_quote(vec![list[index - 1].clone()]);
            index = list.len() - 1;
            continue;
        }
        if index > 1 && Some(&Term::make_word("c")) == list.get(index) {
            if let Some(Term::Quote(a)) = list.get(index - 1) {
                if let Some(Term::Quote(b)) = list.get(index - 2) {
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
        if index > 1 && Some(&Term::make_word("s")) == list.get(index) {
            list.remove(index); // 's'
            list.swap(index - 2, index - 1);
            index = list.len() - 1;
            continue;
        }
        if index > 0 && Some(&Term::make_word("r")) == list.get(index) {
            list.remove(index); // 'r'
            list.remove(index - 1); // term
            if list.is_empty() {
                return list;
            }
            index = list.len() - 1;
            continue;
        }
        if index > 0 && Some(&Term::make_word("d")) == list.get(index) {
            list[index] = list[index - 1].clone();
            index = list.len() - 1;
            continue;
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
        assert_eq!(rewrite(rules, parse::terms(begin).unwrap()), parse::terms(end).unwrap())
    }

    #[test]
    fn dup_test() {
        rewrites_to(&[], "x d", "x x");
        rewrites_to(&[], "x d y", "x x y");
        rewrites_to(&[], "x y d", "x y y");
        rewrites_to(&[], "x y d z", "x y y z");

        rewrites_to(&[], "d", "d");
    }

    #[test]
    fn swap_test() {
        rewrites_to(&[], "x y s", "y x");
        rewrites_to(&[], "x y z s", "x z y");
        rewrites_to(&[], "x y s z", "y x z");

        rewrites_to(&[], "s", "s");
        rewrites_to(&[], "x s", "x s");
    }

    #[test]
    fn remove_test() {
        rewrites_to(&[], "x r", "");
        rewrites_to(&[], "x y r", "x");
        rewrites_to(&[], "x y r z", "x z"); 

        rewrites_to(&[], "r", "r");
    }

    #[test]
    fn quote_test() {
        rewrites_to(&[], "x q", "(x)");
        rewrites_to(&[], "x y q", "x (y)");
        rewrites_to(&[], "x q y", "(x) y");
        rewrites_to(&[], "x y q z", "x (y) z");

        rewrites_to(&[], "q", "q");
    }

    #[test]
    fn unquote_test() {
        rewrites_to(&[], "() u", "");
        rewrites_to(&[], "x () u", "x");
        rewrites_to(&[], "() u y", "y");
        rewrites_to(&[], "x () u y", "x y");
        rewrites_to(&[], "(y) u", "y");
        rewrites_to(&[], "x (y) u", "x y");
        rewrites_to(&[], "(x) u y", "x y");
        rewrites_to(&[], "x (y) u z", "x y z");
        rewrites_to(&[], "(x y z) u", "x y z");

        rewrites_to(&[], "u", "u");
    }

    #[test]
    fn concat_test() {
        rewrites_to(&[], "() () c", "()");
        rewrites_to(&[], "(x) () c", "(x)");
        rewrites_to(&[], "() (y) c", "(y)");
        rewrites_to(&[], "(x) (y) c", "(x y)");

        rewrites_to(&[], "c", "c");
        rewrites_to(&[], "(x) c", "(x) c");
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
