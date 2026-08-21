# Atomic 241 pass conformance — io class binding / textwrap surface +
# dedent/indent value ops / string Formatter+Template class binding / bytes
# literal + immutable value ops / operator surface + value ops / heapq.merge
# basic / statistics surface + multimode/quantiles / fractions.Fraction class
# binding that match between CPython 3.12 and mamba.
import io
import textwrap
import string
import operator
import heapq
import statistics
import fractions


_ledger: list[int] = []

# 1) io class binding hasattr surface
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)

# 2) textwrap surface + value ops
assert hasattr(textwrap, "wrap") == True; _ledger.append(1)
assert hasattr(textwrap, "fill") == True; _ledger.append(1)
assert hasattr(textwrap, "dedent") == True; _ledger.append(1)
assert hasattr(textwrap, "indent") == True; _ledger.append(1)
assert hasattr(textwrap, "shorten") == True; _ledger.append(1)
assert textwrap.dedent("    hello\n    world") == "hello\nworld"; _ledger.append(1)
assert textwrap.indent("hello\nworld", "  ") == "  hello\n  world"; _ledger.append(1)

# 3) string Formatter / Template — class binding hasattr
assert hasattr(string, "Formatter") == True; _ledger.append(1)
assert hasattr(string, "Template") == True; _ledger.append(1)

# 4) bytes literal + immutable value ops
assert b"hello" == b"hello"; _ledger.append(1)
assert b"hello".decode("utf-8") == "hello"; _ledger.append(1)
assert b"\xab\xcd".hex() == "abcd"; _ledger.append(1)
assert bytes.fromhex("abcd") == b"\xab\xcd"; _ledger.append(1)
assert bytearray(b"hello") == bytearray(b"hello"); _ledger.append(1)
assert b"abc" + b"def" == b"abcdef"; _ledger.append(1)
assert len(b"hello") == 5; _ledger.append(1)
assert b"hello"[1] == 101; _ledger.append(1)
assert b"hello"[1:3] == b"el"; _ledger.append(1)
assert (b"ell" in b"hello") == True; _ledger.append(1)
assert b"a,b,c".split(b",") == [b"a", b"b", b"c"]; _ledger.append(1)
assert b",".join([b"a", b"b", b"c"]) == b"a,b,c"; _ledger.append(1)
assert b"hello".startswith(b"hel") == True; _ledger.append(1)
assert b"hello".endswith(b"llo") == True; _ledger.append(1)

# 5) operator hasattr surface
assert hasattr(operator, "add") == True; _ledger.append(1)
assert hasattr(operator, "sub") == True; _ledger.append(1)
assert hasattr(operator, "mul") == True; _ledger.append(1)
assert hasattr(operator, "truediv") == True; _ledger.append(1)
assert hasattr(operator, "floordiv") == True; _ledger.append(1)
assert hasattr(operator, "mod") == True; _ledger.append(1)
assert hasattr(operator, "pow") == True; _ledger.append(1)
assert hasattr(operator, "neg") == True; _ledger.append(1)
assert hasattr(operator, "pos") == True; _ledger.append(1)
assert hasattr(operator, "abs") == True; _ledger.append(1)
assert hasattr(operator, "lt") == True; _ledger.append(1)
assert hasattr(operator, "le") == True; _ledger.append(1)
assert hasattr(operator, "eq") == True; _ledger.append(1)
assert hasattr(operator, "ne") == True; _ledger.append(1)
assert hasattr(operator, "gt") == True; _ledger.append(1)
assert hasattr(operator, "ge") == True; _ledger.append(1)
assert hasattr(operator, "and_") == True; _ledger.append(1)
assert hasattr(operator, "or_") == True; _ledger.append(1)
assert hasattr(operator, "xor") == True; _ledger.append(1)
assert hasattr(operator, "invert") == True; _ledger.append(1)
assert hasattr(operator, "lshift") == True; _ledger.append(1)
assert hasattr(operator, "rshift") == True; _ledger.append(1)
assert hasattr(operator, "itemgetter") == True; _ledger.append(1)
assert hasattr(operator, "attrgetter") == True; _ledger.append(1)
assert hasattr(operator, "methodcaller") == True; _ledger.append(1)
assert hasattr(operator, "getitem") == True; _ledger.append(1)
assert hasattr(operator, "setitem") == True; _ledger.append(1)
assert hasattr(operator, "delitem") == True; _ledger.append(1)
assert hasattr(operator, "contains") == True; _ledger.append(1)
assert hasattr(operator, "is_") == True; _ledger.append(1)
assert hasattr(operator, "is_not") == True; _ledger.append(1)
assert hasattr(operator, "not_") == True; _ledger.append(1)
assert hasattr(operator, "truth") == True; _ledger.append(1)
assert hasattr(operator, "iadd") == True; _ledger.append(1)
assert operator.add(3, 4) == 7; _ledger.append(1)
assert operator.sub(10, 3) == 7; _ledger.append(1)
assert operator.mul(3, 4) == 12; _ledger.append(1)
assert operator.eq(3, 3) == True; _ledger.append(1)
assert operator.ne(3, 4) == True; _ledger.append(1)
assert operator.lt(3, 4) == True; _ledger.append(1)
assert operator.gt(4, 3) == True; _ledger.append(1)
assert operator.le(3, 3) == True; _ledger.append(1)
assert operator.ge(3, 3) == True; _ledger.append(1)

# 6) heapq.merge basic — without key/reverse
assert list(heapq.merge([1, 4, 7], [2, 5, 8])) == [1, 2, 4, 5, 7, 8]; _ledger.append(1)

# 7) statistics surface + value ops that conform
assert hasattr(statistics, "median_grouped") == True; _ledger.append(1)
assert hasattr(statistics, "fmean") == True; _ledger.append(1)
assert hasattr(statistics, "geometric_mean") == True; _ledger.append(1)
assert hasattr(statistics, "harmonic_mean") == True; _ledger.append(1)
assert hasattr(statistics, "quantiles") == True; _ledger.append(1)
assert hasattr(statistics, "multimode") == True; _ledger.append(1)
assert hasattr(statistics, "linear_regression") == True; _ledger.append(1)
assert hasattr(statistics, "StatisticsError") == True; _ledger.append(1)
assert statistics.multimode([1, 1, 2, 2, 3]) == [1, 2]; _ledger.append(1)
assert statistics.quantiles([1, 2, 3, 4, 5, 6, 7, 8, 9]) == [2.5, 5.0, 7.5]; _ledger.append(1)

# 8) fractions.Fraction class binding
assert hasattr(fractions, "Fraction") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_io_textwrap_string_bytes_operator_heapq_statistics_value_ops {sum(_ledger)} asserts")
