# Operational AssertionPass seed for the value contract of the
# `bisect` / `heapq` / `copy` / `string` / `shlex` / `textwrap`
# six-pack pinned to atomic 176: `bisect` (the documented
# `bisect_left` / `bisect_right` / `bisect` / `insort` /
# `insort_left` / `insort_right` module-level helper surface),
# `heapq` (the documented `heapify` / `heappop` / `heappush` /
# `heappushpop` / `heapreplace` / `nlargest` / `nsmallest` /
# `merge` module-level helper surface), `copy` (the documented
# `copy` shallow + `deepcopy` deep + `Error` exception class
# value contract), `string` (the documented `ascii_lowercase` /
# `ascii_uppercase` / `ascii_letters` / `digits` / `punctuation`
# / `whitespace` / `hexdigits` / `octdigits` constants + the
# documented `capwords` module-level helper value contract),
# `shlex` (the documented `split` / `quote` / `join` module-
# level helper value contract), and `textwrap` (the documented
# `dedent` / `indent` module-level helper value contract).
#
# The matching subset between mamba and CPython is the full
# `bisect` module-level helper layer (bisect_left / bisect_right
# / bisect / insort + hasattr surface), the full `heapq`
# module-level helper layer (heapify / heappop / heappush /
# nlargest / nsmallest + hasattr surface), the full `copy`
# module-level helper layer (copy shallow + deepcopy deep +
# Error hasattr), the `string` constant-string value layer
# (ascii_lowercase / ascii_uppercase / ascii_letters / digits /
# punctuation / whitespace / hexdigits / octdigits) + the
# `string.capwords` module-level helper + the partial `string`
# hasattr surface (ascii_lowercase / ascii_uppercase /
# ascii_letters / digits / punctuation / whitespace /
# hexdigits / octdigits / Template / Formatter / capwords —
# `printable` DIVERGES), the `shlex.split` / `shlex.quote` /
# `shlex.join` module-level helper layer + the partial
# `shlex` hasattr surface (split / quote / join —
# `shlex` DIVERGES), and the `textwrap.dedent` / `textwrap.
# indent` module-level helper layer + the partial `textwrap`
# hasattr surface (wrap / fill / shorten / dedent / indent —
# `TextWrapper` DIVERGES).
#
# Surface in this fixture:
#   • bisect.bisect_left / bisect_right / bisect — insertion-
#     index lookup;
#   • bisect.insort — in-place sorted insertion;
#   • bisect — module hasattr surface (bisect / bisect_left /
#     bisect_right / insort / insort_left / insort_right);
#   • heapq.heapify / heappop / heappush — min-heap mutation
#     contract;
#   • heapq.nlargest / nsmallest — k-extremum extraction;
#   • heapq — module hasattr surface (heapify / heappop /
#     heappush / heappushpop / heapreplace / nlargest /
#     nsmallest / merge);
#   • copy.copy — shallow-copy outer-equality / inner-identity
#     contract;
#   • copy.deepcopy — deep-copy outer-equality / inner-
#     reallocation contract;
#   • copy — module hasattr surface (copy / deepcopy / Error);
#   • string — ascii_lowercase / ascii_uppercase / ascii_letters
#     / digits / punctuation / whitespace / hexdigits /
#     octdigits constant value contract;
#   • string.capwords — title-case-per-word value contract;
#   • string — partial module hasattr surface (ascii_lowercase
#     / ascii_uppercase / ascii_letters / digits / punctuation
#     / whitespace / hexdigits / octdigits / Template /
#     Formatter / capwords — `printable` DIVERGES);
#   • shlex.split — POSIX tokenization;
#   • shlex.quote — shell-safe quoting;
#   • shlex.join — token-list join;
#   • shlex — partial module hasattr surface (split / quote /
#     join — `shlex` class identifier DIVERGES);
#   • textwrap.dedent — common-prefix strip;
#   • textwrap.indent — line-prefix injection;
#   • textwrap — partial module hasattr surface (wrap / fill /
#     shorten / dedent / indent — `TextWrapper` class
#     identifier DIVERGES).
#
# Behavioral edges that DIVERGE on mamba (textwrap.wrap /
# textwrap.fill / textwrap.shorten ignore the `width=` argument
# — wrapping contract broken, string.printable returns the
# empty string and hasattr(string, "printable") is False —
# documented printable constant is missing, string.Template(...)
# returns a `dict` not the documented Template instance and
# .substitute AttributeError on the `dict`, hasattr(textwrap,
# "TextWrapper") / hasattr(shlex, "shlex") are False —
# documented class identifiers are missing) are covered in
# the matching spec fixture
# `lang_textwrap_string_template_silent`.
import bisect
import heapq
import copy
import string
import shlex
import textwrap


_ledger: list[int] = []

# 1) bisect.bisect_left / bisect_right / bisect — insertion-index
_arr = [1, 3, 5, 7, 9]
assert bisect.bisect_left(_arr, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(_arr, 5) == 3; _ledger.append(1)
assert bisect.bisect(_arr, 6) == 3; _ledger.append(1)
assert bisect.bisect_left(_arr, 0) == 0; _ledger.append(1)
assert bisect.bisect_right(_arr, 100) == 5; _ledger.append(1)

# 2) bisect.insort — in-place sorted insertion
_arr2 = [1, 3, 5, 7, 9]
bisect.insort(_arr2, 4)
assert _arr2 == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
_arr3 = [1, 3, 5]
bisect.insort(_arr3, 0)
assert _arr3 == [0, 1, 3, 5]; _ledger.append(1)

# 3) bisect — module hasattr surface
assert hasattr(bisect, "bisect") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_left") == True; _ledger.append(1)
assert hasattr(bisect, "bisect_right") == True; _ledger.append(1)
assert hasattr(bisect, "insort") == True; _ledger.append(1)
assert hasattr(bisect, "insort_left") == True; _ledger.append(1)
assert hasattr(bisect, "insort_right") == True; _ledger.append(1)

# 4) heapq.heapify / heappop / heappush — min-heap mutation
_h = [3, 1, 4, 1, 5, 9, 2, 6]
heapq.heapify(_h)
assert _h[0] == 1; _ledger.append(1)
assert heapq.heappop(_h) == 1; _ledger.append(1)
heapq.heappush(_h, 0)
assert _h[0] == 0; _ledger.append(1)

# 5) heapq.nlargest / nsmallest — k-extremum
assert heapq.nlargest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [9, 6, 5]; _ledger.append(1)
assert heapq.nsmallest(3, [3, 1, 4, 1, 5, 9, 2, 6]) == [1, 1, 2]; _ledger.append(1)

# 6) heapq — module hasattr surface
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappushpop") == True; _ledger.append(1)
assert hasattr(heapq, "heapreplace") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)
assert hasattr(heapq, "merge") == True; _ledger.append(1)

# 7) copy.copy — shallow-copy outer-equality / inner-identity
_src = [1, [2, 3], {4: 5}]
_shallow = copy.copy(_src)
assert _shallow == _src; _ledger.append(1)
assert (_shallow is _src) == False; _ledger.append(1)
assert (_shallow[1] is _src[1]) == True; _ledger.append(1)

# 8) copy.deepcopy — deep-copy outer-equality / inner-reallocation
_deep = copy.deepcopy(_src)
assert _deep == _src; _ledger.append(1)
assert (_deep[1] is _src[1]) == False; _ledger.append(1)

# 9) copy — module hasattr surface
assert hasattr(copy, "copy") == True; _ledger.append(1)
assert hasattr(copy, "deepcopy") == True; _ledger.append(1)
assert hasattr(copy, "Error") == True; _ledger.append(1)

# 10) string — constant value contract
assert string.ascii_lowercase == "abcdefghijklmnopqrstuvwxyz"; _ledger.append(1)
assert string.ascii_uppercase == "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; _ledger.append(1)
assert string.digits == "0123456789"; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)
assert len(string.whitespace) == 6; _ledger.append(1)
assert "!" in string.punctuation; _ledger.append(1)

# 11) string.capwords — title-case-per-word
assert string.capwords("hello world foo") == "Hello World Foo"; _ledger.append(1)
assert string.capwords("a b c") == "A B C"; _ledger.append(1)

# 12) string — partial module hasattr surface
#     (printable DIVERGES — moved to spec fixture)
assert hasattr(string, "ascii_lowercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_uppercase") == True; _ledger.append(1)
assert hasattr(string, "ascii_letters") == True; _ledger.append(1)
assert hasattr(string, "digits") == True; _ledger.append(1)
assert hasattr(string, "punctuation") == True; _ledger.append(1)
assert hasattr(string, "whitespace") == True; _ledger.append(1)
assert hasattr(string, "hexdigits") == True; _ledger.append(1)
assert hasattr(string, "octdigits") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "capwords") == True; _ledger.append(1)

# 13) shlex.split — POSIX tokenization
assert shlex.split('foo "bar baz" qux') == ["foo", "bar baz", "qux"]; _ledger.append(1)
assert shlex.split("a b c") == ["a", "b", "c"]; _ledger.append(1)

# 14) shlex.quote — shell-safe quoting
assert shlex.quote("hello world") == "'hello world'"; _ledger.append(1)
assert shlex.quote("nospaces") == "nospaces"; _ledger.append(1)

# 15) shlex.join — token-list join
assert shlex.join(["foo", "bar baz"]) == "foo 'bar baz'"; _ledger.append(1)
assert shlex.join(["a", "b"]) == "a b"; _ledger.append(1)

# 16) shlex — partial module hasattr surface
#     (shlex class identifier DIVERGES — moved to spec fixture)
assert hasattr(shlex, "split") == True; _ledger.append(1)
assert hasattr(shlex, "quote") == True; _ledger.append(1)
assert hasattr(shlex, "join") == True; _ledger.append(1)

# 17) textwrap.dedent — common-prefix strip
assert textwrap.dedent("  hello\n  world") == "hello\nworld"; _ledger.append(1)
assert textwrap.dedent("    a\n    b") == "a\nb"; _ledger.append(1)

# 18) textwrap.indent — line-prefix injection
assert textwrap.indent("hello\nworld", "> ") == "> hello\n> world"; _ledger.append(1)

# 19) textwrap — partial module hasattr surface
#     (TextWrapper class identifier DIVERGES — moved to spec
#     fixture, alongside wrap / fill / shorten value
#     divergence)
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)

# NB: textwrap.wrap / fill / shorten ignore the width= argument
# on mamba, string.printable is the empty string + hasattr
# False on mamba, string.Template(...) returns dict + .substitute
# AttributeError on mamba, hasattr(textwrap, "TextWrapper") and
# hasattr(shlex, "shlex") are False on mamba — all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_bisect_heapq_copy_string_value_ops {sum(_ledger)} asserts")
