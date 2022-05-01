use crate::ast::{Engine, Primitive, Rule, Term};
use im::Vector;
pub use lasso::ThreadedRodeo;
use std::iter::{FusedIterator, Iterator};
use std::str::Chars;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum Token {
    LeftParen,
    RightParen,
    Period,
    Equals,
    Prim(Primitive),
    Word(String),
    EndOfInput,
}

enum ParseError {
    Consumed(String),
    DidNotConsume(String),
}

struct Tokens<'iter> {
    chars: Chars<'iter>,
    peeked: Vec<Token>,
}

impl<'iter> Tokens<'iter> {
    fn iter_next(&mut self) -> Vec<Token> {
        let mut buf = String::new();
        loop {
            if let Some(c) = self.chars.next() {
                if c.is_whitespace() {
                    if !buf.is_empty() {
                        return vec![Token::Word(buf)];
                    }
                    continue;
                }
                match c {
                    '=' => {
                        return if buf.is_empty() {
                            vec![Token::Equals]
                        } else {
                            vec![Token::Word(buf), Token::Equals]
                        }
                    }
                    '.' => {
                        return if buf.is_empty() {
                            vec![Token::Period]
                        } else {
                            vec![Token::Word(buf), Token::Period]
                        }
                    }
                    '~' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Swap)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Swap)]
                        }
                    }
                    ',' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Combine)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Combine)]
                        }
                    }
                    '+' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Copy)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Copy)]
                        }
                    }
                    '-' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Discard)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Discard)]
                        }
                    }
                    '>' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Wrap)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Wrap)]
                        }
                    }
                    '<' => {
                        return if buf.is_empty() {
                            vec![Token::Prim(Primitive::Unwrap)]
                        } else {
                            vec![Token::Word(buf), Token::Prim(Primitive::Unwrap)]
                        }
                    }
                    '(' => {
                        return if buf.is_empty() {
                            vec![Token::LeftParen]
                        } else {
                            vec![Token::Word(buf), Token::LeftParen]
                        }
                    }
                    ')' => {
                        return if buf.is_empty() {
                            vec![Token::RightParen]
                        } else {
                            vec![Token::Word(buf), Token::RightParen]
                        }
                    }
                    _ => {
                        buf.push(c);
                    }
                }
            } else {
                return if buf.is_empty() {
                    vec![Token::EndOfInput]
                } else {
                    vec![Token::Word(buf), Token::EndOfInput]
                };
            }
        }
    }

    fn peek(&mut self) -> Token {
        if self.peeked.is_empty() {
            let next_tokens = self.iter_next();
            self.peeked.extend(next_tokens);
        }
        self.peeked[0].clone()
    }

    fn at_eoi(&mut self) -> bool {
        if self.peeked.is_empty() {
            let next_tokens = self.iter_next();
            self.peeked.extend(next_tokens);
        }
        self.peeked[0] == Token::EndOfInput
    }

    fn advance(&mut self) {
        if self.peeked.is_empty() {
            let next_tokens = self.iter_next();
            self.peeked.extend(next_tokens);
        }
        self.peeked.remove(0);
    }

    fn new(s: &'iter str) -> Self {
        Tokens {
            chars: s.chars(),
            peeked: vec![],
        }
    }
}

impl<'iter> Iterator for Tokens<'iter> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.peeked.is_empty() {
            let next_tokens = self.iter_next();
            self.peeked.extend(next_tokens);
        }
        Some(self.peeked.remove(0))
    }
}

impl<'iter> FusedIterator for Tokens<'iter> {}

fn display_token(token: &Token) -> String {
    match token {
        Token::LeftParen => "(".to_string(),
        Token::RightParen => ")".to_string(),
        Token::Prim(primitive) => primitive.to_string(),
        Token::Period => ".".to_string(),
        Token::Equals => "=".to_string(),
        Token::Word(s) => format!("word \"{}\"", s),
        Token::EndOfInput => "end of input".to_string(),
    }
}

fn parse_term<'a>(engine: &'a Engine, tokens: &mut Tokens) -> Result<&'a mut Term, ParseError> {
    match tokens.peek() {
        Token::Word(s) => {
            tokens.advance();
            Ok(Term::make_word(engine, engine.get_or_intern(s)))
        }
        Token::Prim(primitive) => {
            tokens.advance();
            Ok(Term::make_prim(engine, primitive))
        }
        Token::LeftParen => {
            tokens.advance();
            let terms = parse_terms(engine, tokens).map_err(ParseError::Consumed)?;
            match tokens.next() {
                Some(Token::RightParen) => Ok(Term::make_quote(engine, terms)),
                Some(token) => Err(ParseError::Consumed(format!(
                    "Expected ')' but found {}",
                    display_token(&token)
                ))),
                None => panic!("inconceivable"),
            }
        }
        token => Err(ParseError::DidNotConsume(format!(
            "Expected term but found {}",
            display_token(&token)
        ))),
    }
}

fn parse_nonquote_term<'a>(
    engine: &'a Engine,
    tokens: &mut Tokens,
) -> Result<&'a mut Term, ParseError> {
    match tokens.peek() {
        Token::Word(s) => {
            tokens.advance();
            Ok(Term::make_word(engine, engine.get_or_intern(s)))
        }
        Token::Prim(primitive) => {
            tokens.advance();
            Ok(Term::make_prim(engine, primitive))
        }
        token => Err(ParseError::DidNotConsume(format!(
            "Expected non-quote term but found {}",
            display_token(&token)
        ))),
    }
}

fn parse_terms(engine: &Engine, tokens: &mut Tokens) -> Result<Vector<Term>, String> {
    let mut terms = Vector::new();
    loop {
        match parse_term(engine, tokens) {
            Ok(term) => terms.push_back(term.clone()),
            Err(ParseError::Consumed(err)) => return Err(err),
            Err(ParseError::DidNotConsume(_)) => break,
        }
    }

    Ok(terms)
}

fn parse_nonquote_terms(engine: &Engine, tokens: &mut Tokens) -> Result<Vector<Term>, String> {
    let mut terms = Vector::new();
    loop {
        match parse_nonquote_term(engine, tokens) {
            Ok(term) => terms.push_back(term.clone()),
            Err(ParseError::Consumed(err)) => return Err(err),
            Err(ParseError::DidNotConsume(_)) => break,
        }
    }

    Ok(terms)
}

fn parse_rule(engine: &Engine, tokens: &mut Tokens) -> Result<Rule, ParseError> {
    let redex = parse_nonquote_terms(engine, tokens).map_err(ParseError::Consumed)?;
    match tokens.peek() {
        Token::Equals => {
            tokens.advance();
            let reduction = parse_terms(engine, tokens).map_err(ParseError::Consumed)?;
            match tokens.next() {
                Some(Token::Period) => Ok(Rule { redex, reduction }),
                Some(token) => Err(ParseError::Consumed(format!(
                    "Expected ';' but found {}",
                    display_token(&token)
                ))),
                None => panic!("inconceivable"),
            }
        }
        token => {
            let s = format!("Expected '=' but found {}", display_token(&token));
            Err(if redex.is_empty() {
                ParseError::DidNotConsume(s)
            } else {
                ParseError::Consumed(s)
            })
        }
    }
}

fn parse_rules(engine: &Engine, tokens: &mut Tokens) -> Result<Vector<Rule>, ParseError> {
    let mut rules = Vector::new();
    loop {
        match parse_rule(engine, tokens) {
            Ok(rule) => {
                rules.push_back(rule);
            }
            Err(ParseError::Consumed(err)) => return Err(ParseError::Consumed(err)),
            Err(ParseError::DidNotConsume(_)) => break,
        }
    }
    Ok(rules)
}

/// Parses aa string into a term or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid term
pub fn term<'a>(engine: &'a Engine, input: &str) -> Result<&'a mut Term, String> {
    let mut tokens = Tokens::new(input);
    match parse_term(engine, &mut tokens) {
        Ok(term) => {
            if tokens.at_eoi() {
                Ok(term)
            } else {
                Err(format!(
                    "Expected end of input but found {}",
                    display_token(&tokens.peek())
                ))
            }
        }
        Err(ParseError::DidNotConsume(err) | ParseError::Consumed(err)) => Err(err),
    }
}

/// Parses a string into a sequence of terms or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid sequence of terms
pub fn terms(engine: &Engine, input: &str) -> Result<Vector<Term>, String> {
    let mut tokens = Tokens::new(input);
    match parse_terms(engine, &mut tokens) {
        Ok(terms) => {
            if tokens.at_eoi() {
                Ok(terms)
            } else {
                Err(format!(
                    "Expected end of input but found {}",
                    display_token(&tokens.peek())
                ))
            }
        }
        Err(err) => Err(err),
    }
}

/// Parses a string into a rule or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid rule
pub fn rule(engine: &Engine, input: &str) -> Result<Rule, String> {
    let mut tokens = Tokens::new(input);
    match parse_rule(engine, &mut tokens) {
        Ok(rule) => {
            if tokens.at_eoi() {
                Ok(rule)
            } else {
                Err(format!(
                    "Expected end of input but found {}",
                    display_token(&tokens.peek())
                ))
            }
        }
        Err(ParseError::DidNotConsume(err) | ParseError::Consumed(err)) => Err(err),
    }
}

/// Parses a string into a sequence of rules or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid sequence of rules
pub fn rules(engine: &Engine, input: &str) -> Result<Vector<Rule>, String> {
    let mut tokens = Tokens::new(input);
    match parse_rules(engine, &mut tokens) {
        Ok(rules) => {
            if tokens.at_eoi() {
                Ok(rules)
            } else {
                Err(format!(
                    "Expected end of input but found {}",
                    display_token(&tokens.peek())
                ))
            }
        }
        Err(ParseError::DidNotConsume(err) | ParseError::Consumed(err)) => Err(err),
    }
}
