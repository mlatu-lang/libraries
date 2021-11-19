module Mlatu.Pretty (prettyTerm, prettyTerms, prettyRule, prettyRules) where

import Mlatu.AST
import Protolude

prettyTerm :: Term -> Text
prettyTerm (Word word) = word
prettyTerm (Quote terms) = "(" <> prettyTerms terms <> ")"

prettyTerms :: Terms -> Text
prettyTerms [] = ""
prettyTerms [x] = prettyTerm x
prettyTerms (x : xs) = prettyTerm x <> " " <> prettyTerms xs

prettyRule :: Rule -> Text
prettyRule (Rule {redex, reduction}) = prettyTerms redex <> " = " <> prettyTerms reduction <> ";"

prettyRules :: Rules -> Text
prettyRules [] = ""
prettyRules [x] = prettyRule x
prettyRules (x : xs) = prettyRule x <> "\n" <> prettyRules xs
