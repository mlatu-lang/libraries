module Mlatu
  ( module Mlatu.AST,
    module Mlatu.Pretty,
    module Mlatu.Parse,
    rewrite,
  )
where

import Mlatu.AST
import Mlatu.Parse
import Mlatu.Pretty
import Protolude

rewrite :: Rules -> Terms -> Terms
rewrite rules terms = go [] (reverse terms)
  where
    match (x : xs) (y : ys)
      | x == y = (\(i, end) -> (i + 1, end)) <$> match xs ys
      | otherwise = Nothing
    match [] end = Just (0, end)
    match xs [] = Nothing

    go xs ys = case foldl'
      ( \prev rule ->
          case prev of
            Just (i, l) ->
              case match (redex rule) (reverse ys) of
                Just (j, end) | j > i -> Just (j, reverse (reduction rule) <> end)
                _ -> Just (i, l)
            Nothing ->
              case match (redex rule) ys of
                Just (i, end) -> Just (i, reverse (reduction rule) <> end)
                Nothing -> Nothing
      )
      Nothing
      rules of
      Just (_, end) -> go [] (reverse xs <> end)
      Nothing -> go' xs ys
    go' xs = \case
      (Word "," : (Quote q1) : (Quote q2) : ts) -> go [] (reverse xs <> (Quote (reverse (q1 <> q2)) : ts))
      (Word "<" : (Quote q) : ts) -> go [] (reverse xs <> (q <> ts))
      (Word ">" : (Quote t) : ts) -> go [] (reverse xs <> (Quote [Quote t] : ts))
      (Word "-" : (Quote _) : ts) -> go [] (reverse xs <> ts)
      (Word "+" : (Quote t) : ts) -> go [] (reverse xs <> ((Quote t) : (Quote t) : ts))
      (Word "~" : (Quote a) : (Quote b) : ts) -> go [] (reverse xs <> ((Quote b) : (Quote a) : ts))
      (t : ts) -> go (t : xs) ts
      [] -> xs
