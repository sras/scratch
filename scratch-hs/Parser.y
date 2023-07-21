{
module Main where
}

%name myExprParser
%tokentype { Token }
%error { parseError }
%left '+' '-'


%token
  '+' { TkOperator OpPlus }
  '-' { TkOperator OpMinus }
  '(' { TkDelimeter BrackOpen }
  ',' { TkDelimeter Comma }
  ')' { TkDelimeter BrackClose }
  int { TkLiteral (LiteralInt $$) }
  iden { TkIdentifier $$ }

%%

Exp : Exp '+' Exp { BinExpr OpPlus $1 $3 }
    | Exp '-' Exp { BinExpr OpMinus $1 $3 }
    | '(' Exp ')' { $2 }
    | iden '(' ArgList ')' { FnCall $1 $3 }
    | Term { ExpTerm $1 }
ArgList : Exp { ArgList [$1] }
        | Exp ',' ArgList { let ArgList x = $3 in ArgList ($1 : x) }
Term : int { (TermLiteral (LiteralInt $1)) }
     | iden { TermIden $1 }
{

data Operator
  = OpPlus
  | OpMinus
  deriving Show

data Delimeter
  = BrackOpen
  | BrackClose
  | Comma
  deriving Show


data Literal
  = LiteralInt Int
  deriving Show

type Identifier = String

data Token
  = TkOperator Operator
  | TkDelimeter Delimeter
  | TkLiteral Literal
  | TkIdentifier Identifier
  deriving Show

data ArgList = ArgList [Exp]
  deriving Show

data Exp
  = BinExpr Operator Exp Exp
  | FnCall Identifier ArgList
  | ExpTerm Term
  deriving Show

data Term
  = TermLiteral Literal
  | TermIden Identifier
  deriving Show

parseError :: [Token] -> a
parseError t = error (show t)

main :: IO ()
main = pure ()
}
