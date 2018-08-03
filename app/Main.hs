{-# Language GADTs #-}
{-# Language DataKinds #-}
{-# Language RankNTypes #-}
{-# Language KindSignatures #-}
{-# Language ExistentialQuantification #-}
{-# Language ScopedTypeVariables #-}

module Main where

data ConversionRate (a :: Currency) (b :: Currency) = ConversionRate Double

class IsCurrency (a::Currency) where
  splitCurrency :: (Money a) -> (Double, String)
  toUSD :: ConversionRate a USD
  zero :: Money a
  zero = Money 0

data Currency = INR | USD | AUD

data Money (x :: Currency) where
  Money :: Double -> Money x

instance (IsCurrency a) => Show (Money a) where
  show b = let (a, c) = splitCurrency b in (show a) ++ " " ++ c

addMoney :: Money a -> Money a -> Money a
addMoney (Money a) (Money b) = (Money $ a + b)

multiplyMoney :: Money a -> Double -> Money a
multiplyMoney (Money a) m = Money (a * m)

convertForward :: Money a -> ConversionRate a b -> Money b
convertForward (Money a) (ConversionRate r) = (Money $ a * r)

convertBackward :: Money b -> ConversionRate a b -> Money a
convertBackward (Money a) (ConversionRate r) = (Money $ a / r)

getZero :: (IsCurrency a) => Money a -> Money a
getZero b = zero

convert :: forall a b. (IsCurrency a, IsCurrency b) => Money a -> Money b
convert s =
  let
    x =  (toUSD :: ConversionRate a USD)
    y =  (toUSD :: ConversionRate b USD)
  in convertBackward (convertForward s x) y

-----

instance IsCurrency INR where
  splitCurrency (Money x) = (x, "INR")
  toUSD = ConversionRate (1/65)

instance IsCurrency USD where
  splitCurrency (Money x) = (x, "USD")
  toUSD = ConversionRate 1

instance IsCurrency AUD where
  splitCurrency (Money x) = (x, "AUD")
  toUSD = ConversionRate (0.5)

data Receipt = forall x. (IsCurrency x) => Receipt { paxAmount :: Money x }

discountAmount :: Receipt -> IO ()
discountAmount Receipt {paxAmount = a} = putStrLn $ show $ multiplyMoney a 0.5

printZero :: Receipt -> IO ()
printZero Receipt {paxAmount = a} = putStrLn $ show $ (getZero a)

printAmount :: Receipt -> IO ()
printAmount Receipt {paxAmount = a} = putStrLn $ show $ a

convertToAUD :: Receipt -> IO ()
convertToAUD Receipt {paxAmount = a} = putStrLn $ show $ (convert a :: Money AUD)

main :: IO ()
main = do
  let p = Receipt (Money 10 :: Money INR)
  printAmount p
  discountAmount p
  printZero p
  convertToAUD p
