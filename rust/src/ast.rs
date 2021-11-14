/// A sequence of mlatu terms
pub type Terms = Vec<Term>;

/// A mlatu term, the basis of the whole language
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub enum Term {
    Word(String),
    Quote(Terms),
}

impl Term {
    /// Makes a word term out of anything that can be turned into a string
    #[must_use]
    pub fn make_word(name: impl Into<String>) -> Self {
        Self::Word(name.into())
    }

    /// Makes a quote out of anything that can be turned into a sequence of terms
    #[must_use]
    pub fn make_quote(terms: impl Into<Terms>) -> Self {
        Self::Quote(terms.into())
    }
}

/// A mlatu redex, the left side of a rule
pub type Redex = Terms;

/// A mlatu reduction, the right side of a rule
pub type Reduction = Terms;

/// A mlatu rule, used during rewriting
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct Rule {
    pub redex: Redex,
    pub reduction: Reduction,
}

/// A sequence of mlatu rules, which form a grammar
pub type Rules = Vec<Rule>;
