str1 :: String
''  = "test1"

str2 :: String
str2 = str3 where
   str3 :: String
   {- comment -}
   '' = "test2"

nestéd :: Int -> Bool
''  0 = α where
   α :: Bool
-- comment
   '' = True
{-
   comment
-}
''  _ = β where
   β :: Bool
   β = False

column :: Int
'' = a where a :: Int
             a = 4

blankLine :: ()

''  = ()

spacesOnlyLine :: ()
          
''  = ()

