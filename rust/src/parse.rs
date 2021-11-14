use crate::ast::{Rule, Rules, Term, Terms};
use combine::parser::char::char;
use combine::parser::choice::or;
use combine::{eof, many1, parser, satisfy, sep_end_by, skip_many, EasyParser, Parser, Stream};
use unic_ucd_category::GeneralCategory;

const RESERVED: &[char] = &['(', ')', ';', '='];

fn word_parser<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
{
    many1::<String, _, _>(satisfy(|c| {
        if RESERVED.contains(&c) {
            false
        } else {
            let cat = GeneralCategory::of(c);
            cat.is_letter()
                || cat.is_number()
                || cat.is_mark()
                || cat.is_symbol()
                || cat.is_punctuation()
        }
    }))
}

fn separator_parser<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    skip_many(satisfy(|c| {
        let cat = GeneralCategory::of(c);
        cat.is_separator() || c == '\n' || c == '\r'
    }))
}

fn term_parser<Input>() -> impl Parser<Input, Output = Term>
where
    Input: Stream<Token = char>,
{
    // let word = many1::<String, _, _>(satisfy(|c| !needs_escaping(c))).map(Term::make_word);
    let quote = char('(')
        .with(terms_parser())
        .skip(char(')'))
        .map(Term::make_quote);
    or(word_parser().map(Term::make_word), quote)
}

parser! {
    fn terms_parser[Input]()(Input) -> Terms
    where [Input: Stream<Token = char>] {
        separator_parser().with(sep_end_by::<Vec<_>, _, _, _>(term_parser(), separator_parser()))
    }
}

fn rule_parser<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
{
    terms_parser()
        .skip(char('='))
        .and(terms_parser())
        .skip(char(';'))
        .map(|(redex, reduction)| Rule { redex, reduction })
}

parser! {
    fn rules_parser[Input]()(Input) -> Rules
    where [Input: Stream<Token = char>] {
        separator_parser().with(sep_end_by::<Vec<_>, _, _, _>(rule_parser(), separator_parser()))
    }
}

/// Parses aa string into a term or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid term
pub fn term(input: &str) -> Result<Term, String> {
    let mut parser = term_parser().skip(eof());
    match parser.easy_parse(input) {
        Ok((result, _)) => Ok(result),
        Err(e) => Err(format!(
            "{}",
            e.map_position(|p| p.translate_position(input))
        )),
    }
}

/// Parses a string into a sequence of terms or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid sequence of terms
pub fn terms(input: &str) -> Result<Terms, String> {
    let mut parser = terms_parser().skip(eof());
    match parser.easy_parse(input) {
        Ok((result, _)) => Ok(result),
        Err(e) => Err(format!(
            "{}",
            e.map_position(|p| p.translate_position(input))
        )),
    }
}

/// Parses a string into a rule or a string error
///
/// # Errors
///
/// Returns an `Err` if the string was not a valid rule
pub fn rule(input: &str) -> Result<Rule, String> {
    let mut parser = rule_parser().skip(eof());
    match parser.easy_parse(input) {
        Ok((result, _)) => Ok(result),
        Err(e) => Err(format!(
            "{}",
            e.map_position(|p| p.translate_position(input))
        )),
    }
}

/// Parses a string into a sequence of rules or a string error
/// 
/// # Errors
///
/// Returns an `Err` if the string was not a valid sequence of rules
pub fn rules(input: &str) -> Result<Rules, String> {
    let mut parser = rules_parser().skip(eof());
    match parser.easy_parse(input) {
        Ok((result, _)) => Ok(result),
        Err(e) => Err(format!(
            "{}",
            e.map_position(|p| p.translate_position(input))
        )),
    }
}
