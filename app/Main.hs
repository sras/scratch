{-# Language DataKinds #-}
{-# Language ExistentialQuantification #-}
{-# Language FlexibleContexts #-}
{-# Language MonoLocalBinds #-}
{-# Language DataKinds #-}
{-# Language MultiParamTypeClasses #-}
{-# Language FlexibleInstances #-}
{-# Language OverloadedStrings #-}
{-# Language ScopedTypeVariables #-}


module Main where

import Money 

-- The following will be the infra code that configure the currencies and conversions

type MyMoney = Money 'INR Rational -- We set the base currency we want to use and also the underlying type (Rational) to store the money value

-- Instances that define currency symbol
instance IsCurrency 'INR where
  getSymbolOption Full _ = "Rupees"
  getSymbolOption Compact _ = "INR"
  getSymbolOption Default _ = "₹"
  getSymbolOption Symbol _ = "₹"

instance IsCurrency 'USD where
  getSymbol _ = "USD"

instance IsCurrency 'AUD where
  getSymbol _ = "AUD"

-- Instances that define conversion rate fetching function
-- This would typically be a procedure that reads data from
-- data base or an api
instance HasIOConversion Rational where
  getRateIO USD INR = return $ ConversionRate 65 
  getRateIO USD AUD = return $ ConversionRate 2 
  getRateIO USD USD = return $ ConversionRate 1 
  getRateIO AUD INR = return $ ConversionRate 32 
  getRateIO INR INR = return $ ConversionRate 1

-- The following is a sample code that handles some business logic

data Receipt = forall c. (IsCurrency c, ToCurrency c, HasConversion c 'INR Rational) => Receipt { receiptAmount :: MyMoney c }

-- ^^ A sample record with a money field inside it. The currency type does not appear on the LHS
--  so while reading from the db, we ca read multiple Recipts with differing currencies.
--  as shown in the below function we will be able to deal with the wrapped money value without knowing the
--  actual currency.

myMoneyFormat :: (Show a) => a -> CurrencySymbol -> String
myMoneyFormat v (CurrencySymbol s) = show v ++ " " ++ s

discountAmount :: Receipt -> IO ()
discountAmount Receipt {receiptAmount = a} = putStrLn $ formatMoney myMoneyFormat Default $ multiplyMoney a 0.5
-- 
printZero :: Receipt -> IO ()
printZero Receipt {receiptAmount = a} = putStrLn $ formatMoney myMoneyFormat Default $ (getZero a)
-- 
printAmount :: Receipt -> IO ()
printAmount Receipt {receiptAmount = a} = putStrLn $ formatMoney myMoneyFormat Default $  a
-- 
convertToAUD :: Receipt -> IO ()
convertToAUD Receipt {receiptAmount = a} = do
  m <- exchange a :: IO (MyMoney 'AUD)
  putStrLn $ formatMoney myMoneyFormat Default $ m
-- 
convertToINR :: Receipt -> IO ()
convertToINR Receipt {receiptAmount = a} = do
  m <- exchange a :: IO (MyMoney 'INR)
  putStrLn $ formatMoney myMoneyFormat Default $ m

convertToUnknown :: Receipt -> Receipt -> IO ()
convertToUnknown Receipt {receiptAmount = a} Receipt {receiptAmount = b} = do
  m <- exchangeRef a b
  putStrLn $ formatMoney myMoneyFormat Default $ m

main :: IO ()
main = do
  [p, r] <- return $ [Receipt (mkMoney Usd 20), Receipt (mkMoney Aud 20)] -- Read some recipts from db
  printAmount p
  discountAmount p
  printZero p
  convertToAUD p
  convertToINR p
  convertToUnknown p r
