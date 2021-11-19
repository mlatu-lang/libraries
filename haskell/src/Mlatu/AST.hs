module Mlatu.AST where

import Protolude

data Term = Word Text | Quote Terms
  deriving (Ord, Eq, Show, Read)

type Terms = [Term]

type Redex = Terms

type Reduction = Terms

data Rule = Rule {redex :: Redex, reduction :: Reduction}
  deriving (Ord, Eq, Show, Read)

type Rules = [Rule]
