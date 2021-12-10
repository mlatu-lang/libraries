use im::Vector;
use lasso::Spur;
use typed_arena::Arena;

/// A mlatu term, the basis of the whole language
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Term {
    Word(Spur),
    Quote(Vector<Self>),
}

#[allow(clippy::mut_from_ref)]
impl Term {
    /// Makes a word term out of a spur
    #[must_use]
    pub fn make_word(arena: &Arena<Self>, spur: Spur) -> &mut Self {
        arena.alloc(Self::Word(spur))
    }

    /// Makes a quote out of anything that can be turned into a sequence of terms
    #[must_use]
    pub fn make_quote(arena: &Arena<Self>, terms: Vector<Self>) -> &mut Self {
        arena.alloc(Self::Quote(terms))
    }

    /// Checks whether the term is a word with the given spur
    #[must_use]
    pub fn is_word(&self, their_spur: Spur) -> bool {
        if let Self::Word(spur) = self {
            spur == &their_spur
        } else {
            false
        }
    }

    #[must_use]
    pub const fn is_quote(&self) -> bool {
        matches!(self, Self::Quote(_))
    }
}

/// A mlatu rule, used during rewriting
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct Rule {
    pub redex: Vector<Term>,
    pub reduction: Vector<Term>,
}
