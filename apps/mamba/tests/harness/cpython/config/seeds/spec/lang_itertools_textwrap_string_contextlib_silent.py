# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `itertools.chain.from_iterable` class
# method (the documented "alternate constructor flattens an iterable
# of iterables" — mamba returns [] and `hasattr(chain, 'from_iterable')`
# is False), `itertools.count` (the documented "infinite arithmetic
# generator usable via islice" — mamba returns [] when sliced),
# `itertools.cycle` (the documented "infinite repeating generator
# usable via islice" — mamba returns [] when sliced),
# `hasattr(textwrap, 'TextWrapper')` (the documented "TextWrapper
# configurable class" — mamba returns False because the class is
# missing), `textwrap.fill` (the documented "fills the paragraph by
# wrapping lines to the width" — mamba returns the original text
# unchanged), `string.Template().substitute()` (the documented
# "Template instance has .substitute that replaces $name placeholders"
# — mamba raises AttributeError because Template is a dict object
# without the method), `hasattr(contextlib, 'ExitStack')` (the
# documented "ExitStack composable stack of context managers" — mamba
# returns False because the class is missing), `contextlib.suppress`
# (the documented "with-block swallows the specified exception type"
# — mamba re-raises the exception), `contextlib.contextmanager`
# decorator (the documented "decorator turns a generator into a
# context manager that binds the yielded value" — mamba binds the
# wrong value), and `hasattr(string, 'printable')` (the documented
# "string module exposes a `printable` constant" — mamba returns
# False because the symbol is missing).
# Ten-pack pinned to atomic 255.
#
# Behavioral edges that CONFORM on mamba (itertools — chain two
# iterables, islice 3 forms, repeat with count, takewhile/dropwhile
# predicate, product/permutations/combinations, starmap pow,
# zip_longest with fillvalue, accumulate, filterfalse, compress,
# groupby, hasattr chain/islice/count/cycle/takewhile/dropwhile/
# repeat/product/permutations/combinations/starmap/zip_longest/
# accumulate/filterfalse/compress/groupby. textwrap — wrap short
# string, hasattr wrap/fill/dedent/indent/shorten. string —
# ascii_letters/ascii_lowercase/ascii_uppercase/digits/hexdigits
# values + hasattr ascii_letters/_lowercase/_uppercase/digits/
# hexdigits/octdigits/punctuation/whitespace/Template/Formatter.
# contextlib — hasattr contextmanager/suppress/nullcontext +
# nullcontext yields its argument) are covered in the matching pass
# fixture `test_itertools_textwrap_string_contextlib_value_ops`.
import itertools
import textwrap
import string
import contextlib
from typing import Any


_ledger: list[int] = []

# 1) itertools.chain.from_iterable — alternate constructor
#    (mamba: returns [])
def _chain_from_iter() -> list:
    return list(itertools.chain.from_iterable([[1, 2], [3, 4]]))
assert _chain_from_iter() == [1, 2, 3, 4]; _ledger.append(1)

# 2) itertools.count via islice — infinite arithmetic generator
#    (mamba: returns [])
def _count_islice() -> list:
    return list(itertools.islice(itertools.count(5, 2), 3))
assert _count_islice() == [5, 7, 9]; _ledger.append(1)

# 3) itertools.cycle via islice — infinite repeating generator
#    (mamba: returns [])
def _cycle_islice() -> list:
    return list(itertools.islice(itertools.cycle("ab"), 6))
assert _cycle_islice() == ["a", "b", "a", "b", "a", "b"]; _ledger.append(1)

# 4) hasattr textwrap.TextWrapper — configurable class
#    (mamba: returns False — class missing)
assert hasattr(textwrap, "TextWrapper") == True; _ledger.append(1)

# 5) textwrap.fill — wraps lines to the given width
#    (mamba: returns the original text unchanged)
def _fill() -> str:
    return textwrap.fill("hello world how are you", width=10)
assert _fill() == "hello\nworld how\nare you"; _ledger.append(1)

# 6) string.Template.substitute — replaces $name placeholders
#    (mamba: AttributeError because Template is a dict)
def _template_sub() -> Any:
    try:
        return string.Template("Hello $name").substitute(name="World")
    except AttributeError:
        return None
assert _template_sub() == "Hello World"; _ledger.append(1)

# 7) hasattr contextlib.ExitStack — composable stack
#    (mamba: returns False — class missing)
assert hasattr(contextlib, "ExitStack") == True; _ledger.append(1)

# 8) contextlib.suppress — swallows the specified exception type
#    (mamba: re-raises the exception)
def _suppress() -> str:
    try:
        with contextlib.suppress(ValueError):
            raise ValueError("ignored")
    except ValueError:
        return "raised"
    return "ok"
assert _suppress() == "ok"; _ledger.append(1)

# 9) contextlib.contextmanager — binds the yielded value
#    (mamba: binds the wrong value)
@contextlib.contextmanager
def _mycm():
    yield "X"

def _cm_use() -> Any:
    with _mycm() as v:
        return v
assert _cm_use() == "X"; _ledger.append(1)

# 10) hasattr string.printable — exposed module constant
#     (mamba: returns False — symbol missing)
assert hasattr(string, "printable") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_itertools_textwrap_string_contextlib_silent {sum(_ledger)} asserts")
