import Mlatu qualified as M
import Mlatu.AST
import Mlatu.Parse qualified as P
import Protolude
import Test.Hspec

main :: IO ()
main = hspec $ do
  describe "Mlatu.Parse.parseTerm" $ do
    it "handles whitespace correctly" $ do
      P.parseTerm ""
        `shouldBe` Left
          "1:1:\n\
          \  |\n\
          \1 | <empty line>\n\
          \  | ^\n\
          \unexpected end of input\n\
          \expecting term\n"
      P.parseTerm "  "
        `shouldBe` Left
          "1:1:\n\
          \  |\n\
          \1 |   \n\
          \  | ^\n\
          \unexpected space\n\
          \expecting term\n"
      P.parseTerm "\r\n"
        `shouldBe` Left
          "1:1:\n\
          \  |\n\
          \1 | \r\n\
          \  | ^\n\
          \unexpected carriage return\n\
          \expecting term\n"
      P.parseTerm "\0000"
        `shouldBe` Left
          "1:1:\n\
          \  |\n\
          \1 | \NUL\n\
          \  | ^\n\
          \unexpected null\n\
          \expecting term\n"
    it "handles Unicode words correctly" $ do
      P.parseTerm "hello" `shouldBe` Right (Word "hello")
      P.parseTerm "CaPiTaLsToO" `shouldBe` Right (Word "CaPiTaLsToO")
      P.parseTerm "Ω≈ç√∫˜µ≤≥÷" `shouldBe` Right (Word "Ω≈ç√∫˜µ≤≥÷")
      P.parseTerm "⁰⁴⁵₀₁₂" `shouldBe` Right (Word "⁰⁴⁵₀₁₂")
      P.parseTerm "찦차를" `shouldBe` Right (Word "찦차를")
      P.parseTerm "🐵🙈🙉🙊" `shouldBe` Right (Word "🐵🙈🙉🙊")
      P.parseTerm "الستار" `shouldBe` Right (Word "الستار")
    it "handles quotations correctly" $ do
      P.parseTerm "()" `shouldBe` Right (Quote [])
      P.parseTerm "(()()())" `shouldBe` Right (Quote [Quote [], Quote [], Quote []])
      P.parseTerm "(((((())))))" `shouldBe` Right (Quote [Quote [Quote [Quote [Quote [Quote []]]]]])
  describe "Mlatu.rewrite" $ do
    it "works correctly with '+'" $ do
      M.rewrite [] [Quote [Word "x"], Word "+"] `shouldBe` [Quote [Word "x"], Quote [Word "x"]]
      M.rewrite [] [Quote [Word "x"], Word "+", Quote [Word "y"]] `shouldBe` [Quote [Word "x"], Quote [Word "x"], Quote [Word "y"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "+"] `shouldBe` [Quote [Word "x"], Quote [Word "y"], Quote [Word "y"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "+", Quote [Word "z"]] `shouldBe` [Quote [Word "x"], Quote [Word "y"], Quote [Word "y"], Quote [Word "z"]]
      M.rewrite [] [Word "+"] `shouldBe` [Word "+"]
      M.rewrite [] [Word "x", Word "+"] `shouldBe` [Word "x", Word "+"]
    it "works correctly with '-'" $ do
      M.rewrite [] [Quote [Word "x"], Word "-"] `shouldBe` []
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "-"] `shouldBe` [Quote [Word "x"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "-", Quote [Word "z"]] `shouldBe` [Quote [Word "x"], Quote [Word "z"]]
      M.rewrite [] [Word "-"] `shouldBe` [Word "-"]
      M.rewrite [] [Word "x", Word "-"] `shouldBe` [Word "x", Word "-"]
    it "works correctly with '~'" $ do
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "~"] `shouldBe` [Quote [Word "y"], Quote [Word "x"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Quote [Word "z"], Word "~"] `shouldBe` [Quote [Word "x"], Quote [Word "z"], Quote [Word "y"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word "~", Quote [Word "z"]] `shouldBe` [Quote [Word "y"], Quote [Word "x"], Quote [Word "z"]]
      M.rewrite [] [Word "~"] `shouldBe` [Word "~"]
      M.rewrite [] [Word "x", Word "~"] `shouldBe` [Word "x", Word "~"]
      M.rewrite [] [Word "x", Word "y", Word "~"] `shouldBe` [Word "x", Word "y", Word "~"]
      M.rewrite [] [Quote [Word "x"], Word "~"] `shouldBe` [Quote [Word "x"], Word "~"]
    it "works correctly with '>'" $ do
      M.rewrite [] [Quote [Word "x"], Word ">"] `shouldBe` [Quote [Quote [Word "x"]]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word ">"] `shouldBe` [Quote [Word "x"], Quote [Quote [Word "y"]]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word ">", Quote [Word "z"]] `shouldBe` [Quote [Word "x"], Quote [Quote [Word "y"]], Quote [Word "z"]]
      M.rewrite [] [Word ">"] `shouldBe` [Word ">"]
      M.rewrite [] [Word "x", Word ">"] `shouldBe` [Word "x", Word ">"]
    it "works correctly with ','" $ do
      M.rewrite [] [Quote [], Quote [], Word ","] `shouldBe` [Quote []]
      M.rewrite [] [Quote [Word "x"], Quote [], Word ","] `shouldBe` [Quote [Word "x"]]
      M.rewrite [] [Quote [], Quote [Word "y"], Word ","] `shouldBe` [Quote [Word "y"]]
      M.rewrite [] [Quote [Word "x"], Quote [Word "y"], Word ","] `shouldBe` [Quote [Word "x", Word "y"]]
      M.rewrite [] [Word ","] `shouldBe` [Word ","]
      M.rewrite [] [Quote [Word "x"], Word ","] `shouldBe` [Quote [Word "x"], Word ","]
    it "works correctly with '<'" $ do
      M.rewrite [] [Quote [], Word "<"] `shouldBe` []
      M.rewrite [] [Word "x", Quote [], Word "<"] `shouldBe` [Word "x"]
      M.rewrite [] [Quote [], Word "<", Word "y"] `shouldBe` [Word "y"]
      M.rewrite [] [Word "x", Quote [], Word "<", Word "y"] `shouldBe` [Word "x", Word "y"]
      M.rewrite [] [Quote [Word "y"], Word "<"] `shouldBe` [Word "y"]
      M.rewrite [] [Word "x", Quote [Word "y"], Word "<"] `shouldBe` [Word "x", Word "y"]
      M.rewrite [] [Quote [Word "x"], Word "<", Word "y"] `shouldBe` [Word "x", Word "y"]
      M.rewrite [] [Word "x", Quote [Word "y"], Word "<", Word "z"] `shouldBe` [Word "x", Word "y", Word "z"]
      M.rewrite [] [Word "<"] `shouldBe` [Word "<"]
    it "works correctly with user-defined rules" $ do
      let rules = [Rule {redex = [Word "x"], reduction = [Word "y", Word "z"]}, Rule {redex = [Word "x", Word "x"], reduction = [Word "aaaaaaaaa"]}]
      M.rewrite rules [Word "x"] `shouldBe` [Word "y", Word "z"]
      M.rewrite rules [Word "a", Word "x"] `shouldBe` [Word "a", Word "y", Word "z"]
