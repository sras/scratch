{-# Language GADTs #-}
{-# Language DataKinds #-}
{-# Language RankNTypes #-}
{-# Language KindSignatures #-}
{-# Language ExistentialQuantification #-}
{-# Language ScopedTypeVariables #-}

module Main (main) where

data MyGADT s where
  IntGADT :: MyGADT Int
  StringGADT :: MyGADT String

fn :: MyGADT Int -> ()
fn IntGADT = ()

main :: IO ()
main = pure $ fn IntGADT
