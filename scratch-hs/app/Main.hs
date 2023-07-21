{-# Language GADTs #-}
{-# Language DataKinds #-}
{-# Language RankNTypes #-}
{-# Language KindSignatures #-}
{-# Language RoleAnnotations #-}
{-# Language ExistentialQuantification #-}
{-# Language ScopedTypeVariables #-}

module Main (main) where

data MyGADT s where
  IntGADT :: MyGADT Int
  StringGADT :: MyGADT String

main :: IO ()
main = pure ()

-- data CType a = CInt | CPair (CType a) (CType a)

data Foo a = MkFoo (Foo a); type role Foo phantom

