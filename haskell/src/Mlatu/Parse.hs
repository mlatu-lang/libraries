module Mlatu.Parse (parseTerm, parseTerms, parseRule, parseRules) where

import Data.Set qualified as Set
import Mlatu.AST
import Protolude hiding (many, some, try)
import Text.Megaparsec
import Text.Megaparsec.Char

type Parser = Parsec Void Text

separators :: Parser ()
separators = skipMany (separatorChar <|> newline) <?> "whitespace"

quote :: Parser Term
quote = Quote <$> between (char '(') (char ')') terms <?> "quote"

wordChar :: Parser Char
wordChar = do
  c <- letterChar <|> numberChar <|> punctuationChar <|> symbolChar
  if c == '(' || c == ')' || c == ';' || c == '='
    then failure Nothing Set.empty
    else pure c

word :: Parser Term
word = Word . toS <$> (some wordChar <?> "word")

term :: Parser Term
term = (try word <|> quote) <?> "term"

terms :: Parser Terms
terms = separators >> (term `endBy` separators) <?> "terms"

rule :: Parser Rule
rule =
  ( do
      redex <- terms
      void $ char '='
      reduction <- terms
      void $ char ';'
      pure $ Rule {redex, reduction}
  )
    <?> "rule"

rules :: Parser Rules
rules = separators >> (rule `endBy` separators) <?> "rules"

parseTerm :: Text -> Either [Char] Term
parseTerm text = case runParser term "" text of
  Left error -> Left (errorBundlePretty error)
  Right term -> Right term

parseTerms :: Text -> Either [Char] Terms
parseTerms text = case runParser terms "" text of
  Left error -> Left (errorBundlePretty error)
  Right terms -> Right terms

parseRule :: Text -> Either [Char] Rule
parseRule text = case runParser rule "" text of
  Left error -> Left (errorBundlePretty error)
  Right rule -> Right rule

parseRules :: Text -> Either [Char] Rules
parseRules text = case runParser rules "" text of
  Left error -> Left (errorBundlePretty error)
  Right rule -> Right rule
