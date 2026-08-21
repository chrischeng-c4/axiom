# Atomic 255 pass conformance — itertools module
# (hasattr surface chain/islice/count/cycle/takewhile/dropwhile/repeat/
# product/permutations/combinations/starmap/zip_longest/accumulate/
# filterfalse/compress/groupby + chain two iterables, islice 3 forms
# (n / start..stop / start..stop..step), repeat with count cap,
# takewhile/dropwhile under predicate, product cross, permutations
# r=2 ordered tuples, combinations r=2 unordered tuples, starmap pow,
# zip_longest with fillvalue, accumulate running sum, filterfalse odd,
# compress mask, groupby consecutive runs) + textwrap module
# (hasattr surface wrap/fill/dedent/indent/shorten + wrap short
# string returns single-element list) + string module
# (constants ascii_letters/ascii_lowercase/ascii_uppercase/digits/
# hexdigits values + hasattr ascii_letters/_lowercase/_uppercase/
# digits/hexdigits/octdigits/punctuation/whitespace/Template/
# Formatter) + contextlib module (hasattr contextmanager/suppress/
# nullcontext + nullcontext yields the passed value). All asserts
# match between CPython 3.12 and mamba.
import itertools
import textwrap
import string
import contextlib


_ledger: list[int] = []

# 1) itertools — hasattr surface
assert hasattr(itertools, "chain") == True; _ledger.append(1)
assert hasattr(itertools, "islice") == True; _ledger.append(1)
assert hasattr(itertools, "count") == True; _ledger.append(1)
assert hasattr(itertools, "cycle") == True; _ledger.append(1)
assert hasattr(itertools, "takewhile") == True; _ledger.append(1)
assert hasattr(itertools, "dropwhile") == True; _ledger.append(1)
assert hasattr(itertools, "repeat") == True; _ledger.append(1)
assert hasattr(itertools, "product") == True; _ledger.append(1)
assert hasattr(itertools, "permutations") == True; _ledger.append(1)
assert hasattr(itertools, "combinations") == True; _ledger.append(1)
assert hasattr(itertools, "starmap") == True; _ledger.append(1)
assert hasattr(itertools, "zip_longest") == True; _ledger.append(1)
assert hasattr(itertools, "accumulate") == True; _ledger.append(1)
assert hasattr(itertools, "filterfalse") == True; _ledger.append(1)
assert hasattr(itertools, "compress") == True; _ledger.append(1)
assert hasattr(itertools, "groupby") == True; _ledger.append(1)

# 2) itertools — chain two iterables
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)

# 3) itertools — islice (3 forms)
assert list(itertools.islice(range(10), 3)) == [0, 1, 2]; _ledger.append(1)
assert list(itertools.islice(range(10), 2, 5)) == [2, 3, 4]; _ledger.append(1)
assert list(itertools.islice(range(10), 0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)

# 4) itertools — repeat with count cap
assert list(itertools.repeat("x", 4)) == ["x", "x", "x", "x"]; _ledger.append(1)

# 5) itertools — takewhile / dropwhile
assert list(itertools.takewhile(lambda x: x < 5, [1, 3, 6, 2])) == [1, 3]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 5, [1, 3, 6, 2])) == [6, 2]; _ledger.append(1)

# 6) itertools — product / permutations / combinations
assert list(itertools.product([1, 2], ["a", "b"])) == [(1, "a"), (1, "b"), (2, "a"), (2, "b")]; _ledger.append(1)
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)

# 7) itertools — starmap / zip_longest
assert list(itertools.starmap(pow, [(2, 3), (3, 2)])) == [8, 9]; _ledger.append(1)
assert list(itertools.zip_longest([1, 2, 3], ["a"], fillvalue="?")) == [(1, "a"), (2, "?"), (3, "?")]; _ledger.append(1)

# 8) itertools — accumulate / filterfalse / compress
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: x % 2 == 0, [1, 2, 3, 4, 5])) == [1, 3, 5]; _ledger.append(1)
assert list(itertools.compress("ABCDE", [1, 0, 1, 0, 1])) == ["A", "C", "E"]; _ledger.append(1)

# 9) itertools — groupby
assert [(k, list(g)) for k, g in itertools.groupby("aabbcc")] == [("a", ["a", "a"]), ("b", ["b", "b"]), ("c", ["c", "c"])]; _ledger.append(1)

# 10) textwrap — hasattr surface
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)

# 11) textwrap — wrap returns single-element list when text fits
assert textwrap.wrap("hello world", width=20) == ["hello world"]; _ledger.append(1)

# 12) string — constants
assert string.ascii_letters == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)

# 13) string — hasattr surface
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)

# 14) contextlib — hasattr surface (only matching symbols on mamba)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 15) contextlib — nullcontext yields the value
def _nullctx_yield() -> str:
    with contextlib.nullcontext("hi") as x:
        return x
assert _nullctx_yield() == "hi"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_itertools_textwrap_string_contextlib_value_ops {sum(_ledger)} asserts")
