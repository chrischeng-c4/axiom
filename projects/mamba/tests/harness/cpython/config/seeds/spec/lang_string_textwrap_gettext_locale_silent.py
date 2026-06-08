# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(string, 'printable')` (the
# documented "string exposes the printable character class" — mamba
# returns False), `hasattr(textwrap, 'TextWrapper')` (the documented
# "textwrap exposes the TextWrapper configuration class" — mamba
# returns False), `textwrap.shorten('a b c d e f', width=10)` (the
# documented "shorten truncates to 'a b [...]'" — mamba returns
# 'a b c d e f' — no truncation), `textwrap.wrap('a b c d e',
# width=4)` (the documented "wrap returns ['a b', 'c d', 'e']" —
# mamba returns ['a b c d e'] — no wrapping), `textwrap.fill
# ('a b c d e', width=4)` (the documented "fill returns 'a b\\nc d
# \\ne'" — mamba returns 'a b c d e' — no wrapping), `hasattr
# (gettext, 'gettext')` (the documented "gettext exposes the gettext
# translation entry" — mamba returns False — gettext module is
# None), `hasattr(gettext, 'NullTranslations')` (the documented
# "gettext exposes the NullTranslations class" — mamba returns
# False), `hasattr(locale, 'LC_MONETARY')` (the documented "locale
# exposes the LC_MONETARY category constant" — mamba returns False),
# `hasattr(locale, 'Error')` (the documented "locale exposes the
# Error exception class" — mamba returns False), and `locale.LC_ALL
# == 0` (the documented "LC_ALL category constant is 0" — mamba
# returns 6).
# Ten-pack pinned to atomic 278.
#
# Behavioral edges that CONFORM on mamba (string — hasattr ascii_
# lowercase/ascii_uppercase/ascii_letters/digits/hexdigits/octdigits/
# punctuation/whitespace/Template/Formatter/capwords + ascii_
# lowercase == 'a..z' + ascii_uppercase 'A..Z' + digits '0..9' +
# hexdigits + octdigits + len 26/26/52/10/22/8 + capwords 'hi there'
# == 'Hi There'. textwrap — hasattr wrap/fill/dedent/indent/shorten
# + dedent strips common indent. locale — hasattr getlocale/setlocale/
# LC_ALL/LC_CTYPE/LC_NUMERIC/LC_TIME) are covered in the matching
# pass fixture `test_string_textwrap_gettext_locale_value_ops`.
import string
import textwrap
import gettext
import locale


_ledger: list[int] = []

# 1) hasattr(string, 'printable') — printable char class
#    (mamba: returns False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 2) hasattr(textwrap, 'TextWrapper') — wrapper config class
#    (mamba: returns False)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 3) textwrap.shorten('a b c d e f', width=10) — 'a b [...]'
#    (mamba: returns 'a b c d e f' — no truncation)
assert textwrap.shorten("a b c d e f", width=10) == "a b [...]"; _ledger.append(1)

# 4) textwrap.wrap('a b c d e', width=4) — ['a b', 'c d', 'e']
#    (mamba: returns ['a b c d e'] — no wrapping)
assert textwrap.wrap("a b c d e", width=4) == ["a b", "c d", "e"]; _ledger.append(1)

# 5) textwrap.fill('a b c d e', width=4) — 'a b\nc d\ne'
#    (mamba: returns 'a b c d e' — no wrapping)
assert textwrap.fill("a b c d e", width=4) == "a b\nc d\ne"; _ledger.append(1)

# 6) hasattr(gettext, 'gettext') — translation entry
#    (mamba: returns False — gettext module is None)
assert hasattr(gettext, "gettext") == True; _ledger.append(1)

# 7) hasattr(gettext, 'NullTranslations') — NullTranslations class
#    (mamba: returns False)
assert hasattr(gettext, "NullTranslations") == True; _ledger.append(1)

# 8) hasattr(locale, 'LC_MONETARY') — monetary category
#    (mamba: returns False)
assert hasattr(locale, "LC_MONETARY") == True; _ledger.append(1)

# 9) hasattr(locale, 'Error') — locale exception class
#    (mamba: returns False)
assert hasattr(locale, "Error") == True; _ledger.append(1)

# 10) locale.LC_ALL == 0 — category constant value
#     (mamba: returns 6)
assert locale.LC_ALL == 0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_textwrap_gettext_locale_silent {sum(_ledger)} asserts")
