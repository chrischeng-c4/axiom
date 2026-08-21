# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""str predicates: Unicode-aware edge cases + casefold (CPython 3.12)."""

# --- casefold performs full case folding (more than .lower()) ---------
assert "hELlo".casefold() == "hello"
assert "ß".casefold() == "ss"      # eszett expands to two letters
assert "ﬁ".casefold() == "fi"      # fi ligature decomposes
assert "Σ".casefold() == "σ"       # final/medial sigma normalize
assert "µ".casefold() == "μ"       # micro sign -> Greek mu
print("casefold:", "Hello WORLD ß".casefold())

# --- isidentifier follows the Unicode identifier grammar --------------
assert "a".isidentifier()
assert "_x1".isidentifier()
assert "µ".isidentifier()          # Lo/Ll letters are valid starts
assert "𝔘𝔫𝔦".isidentifier()       # astral letters too
assert not "0".isidentifier()      # cannot start with a digit
assert not "a b".isidentifier()    # no spaces
assert not "©".isidentifier()      # symbol, not a letter
print("isidentifier:", "var_2".isidentifier())

# --- isspace recognizes more than ASCII whitespace --------------------
assert " ".isspace()
assert "\t\n\r\x0b\x0c".isspace()
assert " ".isspace()          # no-break space
assert " ".isspace()          # line separator
assert not "".isspace()            # empty is not whitespace
assert not "a ".isspace()
print("isspace ok")

# --- isprintable: empty + space are printable; controls are not -------
assert "".isprintable()
assert " ".isprintable()
assert "abcdefg".isprintable()
assert "ʹ".isprintable()           # printable letter-modifier
assert "👯".isprintable()          # printable emoji
assert not "abc\n".isprintable()   # newline is non-printable
assert not "͸".isprintable()  # unassigned code point
print("isprintable ok")

# --- strings of lone surrogates make every str predicate False --------
for s in ("\ud800", "\udfff", "\ud800\ud800", "\udfff\udfff"):
    for meth in ("islower", "isupper", "istitle", "isalpha", "isalnum",
                 "isdigit", "isspace", "isidentifier", "isprintable",
                 "isnumeric", "isdecimal"):
        assert not getattr(str, meth)(s), (meth, s)
print("surrogate predicates all False")

print("predicates_unicode_edge OK")
