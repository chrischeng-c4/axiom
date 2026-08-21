# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""string module: capwords + character-class constants (CPython 3.12 oracle)."""

import string

# --- string.capwords --------------------------------------------------
# Split on runs of whitespace, capitalize each word, rejoin with a single
# space; leading/trailing whitespace is dropped.
assert string.capwords("abc def ghi") == "Abc Def Ghi"
assert string.capwords("abc\tdef\nghi") == "Abc Def Ghi"
assert string.capwords("abc\t   def  \nghi") == "Abc Def Ghi"
assert string.capwords("ABC DEF GHI") == "Abc Def Ghi"
assert string.capwords("   aBc  DeF   ") == "Abc Def"

# A custom separator is honored for both the split and the rejoin, so
# inner whitespace and separator runs are preserved differently.
assert string.capwords("ABC-DEF-GHI", "-") == "Abc-Def-Ghi"
assert string.capwords("ABC-def DEF-ghi GHI") == "Abc-def Def-ghi Ghi"
assert string.capwords("\taBc\tDeF\t", "\t") == "\tAbc\tDef\t"
print("capwords:", string.capwords("hello world FOO"))

# --- character-class constants ---------------------------------------
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase
assert string.digits == "0123456789"
assert string.hexdigits == string.digits + "abcdefABCDEF"
assert string.octdigits == "01234567"
assert string.whitespace == " \t\n\r\x0b\x0c"
assert string.punctuation == "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
# printable is the concatenation of the other classes.
assert string.printable == (
    string.digits
    + string.ascii_lowercase
    + string.ascii_uppercase
    + string.punctuation
    + string.whitespace
)
print("constants ok:", len(string.printable))

print("string_module OK")
