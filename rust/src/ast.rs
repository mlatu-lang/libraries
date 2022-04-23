use im::Vector;
use lasso::Spur;
use lasso::ThreadedRodeo;
use std::fmt;
use typed_arena::Arena;

/// The mlatu engine
#[derive(Default)]
pub struct Engine {
    arena: Arena<Term>,
    rodeo: ThreadedRodeo,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, term: Term) -> &mut Term {
        self.arena.alloc(term)
    }

    pub fn get_or_intern(&self, s: String) -> Spur {
        self.rodeo.get_or_intern(s)
    }

    pub fn resolve(&self, s: &Spur) -> &str {
        self.rodeo.resolve(s)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Primitive {
    Copy,
    Discard,
    Wrap,
    Unwrap,
    Swap,
    Combine,
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Copy => write!(f, "+"),
            Self::Discard => write!(f, "-"),
            Self::Swap => write!(f, "~"),
            Self::Combine => write!(f, ","),
            Self::Wrap => write!(f, ">"),
            Self::Unwrap => write!(f, "<"),
        }
    }
}

/// A mlatu term, the basis of the whole language
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Term {
    Word(Spur),
    Prim(Primitive),
    Quote(Vector<Self>),
}

#[allow(clippy::mut_from_ref)]
impl Term {
    /// Makes a word term out of a spur
    #[must_use]
    pub fn make_word(engine: &Engine, spur: Spur) -> &mut Self {
        engine.alloc(Self::Word(spur))
    }

    /// Makes a quote out of anything that can be turned into a sequence of terms
    #[must_use]
    pub fn make_quote(engine: &Engine, terms: Vector<Self>) -> &mut Self {
        engine.alloc(Self::Quote(terms))
    }

    #[must_use]
    pub fn make_prim(engine: &Engine, primitive: Primitive) -> &mut Self {
        engine.alloc(Self::Prim(primitive))
    }

    #[must_use]
    pub const fn is_quote(&self) -> bool {
        matches!(self, Self::Quote(_))
    }
}

/// A mlatu rule, used during rewriting
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Default)]
pub struct Rule {
    pub redex: Vector<Term>,
    pub reduction: Vector<Term>,
}

impl Rule {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
