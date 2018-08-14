{-# Language GADTs #-}
{-# Language DataKinds #-}
{-# Language TypeFamilies #-}
{-# Language ExistentialQuantification #-}
{-# Language MultiParamTypeClasses #-}
{-# Language ScopedTypeVariables #-}
{-# Language FlexibleContexts #-}
{-# Language FlexibleInstances #-}
{-# Language UndecidableInstances #-}
{-# Language GeneralizedNewtypeDeriving #-}

module Money
  ( Money
  , ReflectCurrency(..)
  , Currency(..)
  , exchange
  , exchangeRef
  , multiplyMoney
  , getZero
  , addMoney
  , formatMoney
  , CurrencySymbol(..)
  , ConversionRate(..)
  , mkMoney
  , IsCurrency(..)
  , ToCurrency(..)
  , HasConversion(..)
  , zero
  , HasIOConversion(..)
  , toCurrency
  , SymbolType(..)) where

import Data.String
import Control.Monad.IO.Class
import Control.Monad.Catch

data ConversionRate (a :: Currency) (b :: Currency) c = ConversionRate c -- multiplier to convert from currency a to b

data Currency = INR | USD | AUD deriving (Show) -- ..... as many as required

data ReflectCurrency (a :: Currency) where
  Inr :: ReflectCurrency 'INR
  Usd :: ReflectCurrency 'USD
  Aud :: ReflectCurrency 'AUD
  -- ...
  -- ...

newtype CurrencyWrapper (a :: Currency) = CurrencyWrapper Currency

class ToCurrency (a :: Currency) where -- Class to convert type level currency to value.
  getCurrency :: CurrencyWrapper a

instance ToCurrency 'AUD where
  getCurrency  = CurrencyWrapper AUD

instance ToCurrency 'INR where
  getCurrency  = CurrencyWrapper INR

instance ToCurrency 'USD where
  getCurrency  = CurrencyWrapper USD

toCurrency :: forall a b c. (ToCurrency a) => MoneyK b a c -> Currency
toCurrency m = let (CurrencyWrapper c :: CurrencyWrapper a) = getCurrency in c

newtype CurrencySymbol = CurrencySymbol String deriving (Show, IsString)

class HasConversion (a :: Currency) (base :: Currency) c  where -- This is in IO for using the instance down below, that defines this class for types that have HasIOConversion
  getRate :: (MonadIO m, MonadThrow m) => m (ConversionRate a base c)

class HasIOConversion a where
  getRateIO :: (MonadIO m, MonadThrow m) => Currency -> Currency -> m (ConversionRate b c a)

instance forall a b c (base :: Currency) . (HasIOConversion c, ToCurrency a, ToCurrency b, Fractional c) => HasConversion a b c where
  getRate = getRateIO (toCurrency (Money 0 :: Money base c a)) (toCurrency (Money 0 :: Money base c b))

data SymbolType = Default | Full | Compact | Symbol -- Variants of currency symbol

class IsCurrency (a::Currency) where -- Just used to defined the currency symbol
   getSymbol :: Money b c a -> CurrencySymbol
   getSymbol a = getSymbolOption Default a
   getSymbolOption :: SymbolType -> Money b c a -> CurrencySymbol
   getSymbolOption _ a = getSymbol a
   {-# MINIMAL getSymbolOption | getSymbol #-}

data Money (base :: Currency) c (x :: Currency) where -- The core money type
  Money :: (Fractional c) => c -> Money base c x

type MoneyK (b :: Currency) (t :: Currency) a = Money b a t -- An convinence alias to save the kind annotations

zero :: forall c m base . (HasConversion m base c, Fractional c) => Money base c m
zero = Money 0

mkMoney :: forall c m base . (HasConversion m base c, Fractional c) => ReflectCurrency m -> c -> Money base c m
mkMoney _ c = Money c

instance (IsCurrency a, IsCurrency b, Show c) => Show (Money b c a) where
  show b = formatMoney (\a b -> (show a) ++ (show b)) Default b

addMoney :: (Num c) => MoneyK b a c -> MoneyK b a c -> MoneyK b a c
addMoney (Money a) (Money b) = (Money $ a + b)
-- 
multiplyMoney :: MoneyK b a c -> c -> MoneyK b a c
multiplyMoney (Money a) m = Money (a * m)
-- 
getZero :: (IsCurrency a, Fractional c, HasConversion b a c) => MoneyK a b c  -> MoneyK a b c 
getZero _ = Money 0
-- 
exchange :: forall m a base c b. (Fractional c, HasConversion base base c, HasConversion a base c, HasConversion b base c, MonadIO m, MonadThrow m) => MoneyK base a c  -> m (MoneyK base b c)
exchange s = do
  toBase <- getRate :: m (ConversionRate a base c)
  fromBase <- getRate :: m (ConversionRate b base c)
  return $ convertBackward fromBase $ convertForward toBase s
  where
    convertForward :: ConversionRate a base c -> MoneyK base a c -> MoneyK base base c
    convertForward (ConversionRate r) (Money a) = (Money $ a * r)
    convertBackward :: ConversionRate b base c -> MoneyK base base c -> MoneyK base b c
    convertBackward (ConversionRate r) (Money a) = (Money $ a / r)

exchangeRef :: forall m a b c base. (Fractional c, HasIOConversion c, ToCurrency a, ToCurrency base, ToCurrency b, MonadIO m, MonadThrow m) => Money base c a  -> Money base c b -> m (Money base c b)
exchangeRef s r = exchange s

formatMoney :: forall a cur bas vt. (IsCurrency cur, IsCurrency bas) => (vt -> CurrencySymbol -> a) -> SymbolType -> MoneyK cur bas vt -> a
formatMoney ff st a@(Money m) = ff m (getSymbolOption st a)

