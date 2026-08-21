# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(textwrap, 'TextWrapper')`
# (the documented "textwrap exposes the TextWrapper class" — mamba
# returns False), `textwrap.shorten('hello world foo bar', width=10)`
# (the documented "shorten collapses text to at most `width` chars,
# adding the placeholder '[...]' when truncation occurs" — mamba
# returns the input unchanged), `textwrap.indent('a\nb\n', '  ')`
# (the documented "indent prepends prefix to every newline-
# terminated segment, preserving the trailing newline" — mamba
# returns '  a\n  b', dropping the trailing newline), `hasattr
# (string, 'printable')` (the documented "string exposes the
# `printable` constant" — mamba returns False), `string.Template
# ('hi $name').substitute(name='x')` (the documented "Template
# class supports $-placeholder substitution via .substitute" —
# mamba raises AttributeError because Template is a plain dict),
# `hasattr(unicodedata, 'lookup')` (the documented "unicodedata
# exposes the lookup() reverse-name resolver" — mamba returns
# False), `unicodedata.name('A')` (the documented "name('A')
# returns 'LATIN CAPITAL LETTER A'" — mamba returns the generic
# 'UNICODE CHAR 0041'), `len(unicodedata.normalize('NFD', 'café'))`
# (the documented "NFD decomposes precomposed characters — 'café'
# becomes 5 code points" — mamba returns 4, no decomposition),
# `unicodedata.digit('5')` (the documented "digit('5') returns 5"
# — mamba raises AttributeError, function missing), and `shlex
# .split('a # b', comments=True)` (the documented "comments=True
# strips '#' and trailing tokens" — mamba returns ['a', '#', 'b'],
# ignoring the comments kwarg).
# Ten-pack pinned to atomic 263.
#
# Behavioral edges that CONFORM on mamba (textwrap — hasattr fill/
# wrap/dedent/indent/shorten + dedent strips common leading ws,
# dedent mixed indent keeps offset, indent without trailing nl
# prepends prefix. string — hasattr ascii_letters/lowercase/
# uppercase/digits/hexdigits/octdigits/punctuation/whitespace +
# hasattr Template/Formatter/capwords + ascii_lowercase=='abc..xyz',
# ascii_uppercase=='ABC..XYZ', digits=='0..9', hexdigits=
# '0..9a..fA..F', octdigits=='0..7', '!' in punctuation, '@' in
# punctuation, ' ' in whitespace, capwords basic. unicodedata —
# hasattr name/category/normalize/decimal/bidirectional/unidata_
# version + category 'A' Lu, 'a' Ll, '0' Nd, ' ' Zs, normalize NFC
# preserves precomposed, decimal('5')==5, decimal('0')==0,
# bidirectional('A')=='L'. shlex — hasattr split/quote/join +
# split('a b c')==['a','b','c'], split('a "b c" d')==['a','b c',
# 'd'], split('') is empty list, quote('hello')=='hello', quote
# ('hello world') wraps in single quotes, quote("it's") escapes
# the inner quote, join roundtrip, join with space-bearing element)
# are covered in the matching pass fixture
# `test_textwrap_string_unicodedata_shlex_value_ops`.
import textwrap
import string
import unicodedata
import shlex
from typing import Any


_ledger: list[int] = []

# 1) hasattr(textwrap, 'TextWrapper') — textwrap exposes TextWrapper
#    (mamba: returns False)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 2) textwrap.shorten collapses to width with '[...]' placeholder
#    (mamba: returns input unchanged)
assert textwrap.shorten("hello world foo bar", width=10) == "[...]"; _ledger.append(1)

# 3) textwrap.indent preserves trailing newline
#    (mamba: drops the trailing newline)
assert textwrap.indent("a\nb\n", "  ") == "  a\n  b\n"; _ledger.append(1)

# 4) hasattr(string, 'printable') — string exposes printable
#    (mamba: returns False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 5) string.Template('hi $name').substitute(name='x') == 'hi x'
#    (mamba: Template is plain dict — AttributeError on .substitute)
def _template_substitute() -> Any:
    try:
        return string.Template("hi $name").substitute(name="x")
    except AttributeError:
        return None
assert _template_substitute() == "hi x"; _ledger.append(1)

# 6) hasattr(unicodedata, 'lookup') — reverse-name resolver
#    (mamba: returns False)
assert hasattr(unicodedata, "lookup") == True; _ledger.append(1)

# 7) unicodedata.name('A') == 'LATIN CAPITAL LETTER A'
#    (mamba: returns generic 'UNICODE CHAR 0041')
assert unicodedata.name("A") == "LATIN CAPITAL LETTER A"; _ledger.append(1)

# 8) len(unicodedata.normalize('NFD', 'café')) == 5 (decomposed)
#    (mamba: returns 4 — no decomposition performed)
assert len(unicodedata.normalize("NFD", "café")) == 5; _ledger.append(1)

# 9) unicodedata.digit('5') == 5
#    (mamba: AttributeError — function missing)
def _unicodedata_digit() -> Any:
    try:
        return unicodedata.digit("5")
    except AttributeError:
        return None
assert _unicodedata_digit() == 5; _ledger.append(1)

# 10) shlex.split('a # b', comments=True) == ['a']
#     (mamba: returns ['a', '#', 'b'] — comments kwarg ignored)
assert shlex.split("a # b", comments=True) == ["a"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_textwrap_string_unicodedata_shlex_silent {sum(_ledger)} asserts")
