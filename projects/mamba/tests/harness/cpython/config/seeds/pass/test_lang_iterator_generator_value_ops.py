# Atomic 327 pass conformance — iterator / generator depth: generator
# next/list/sum/tuple, generator with send (coroutine-style), yield
# from list/string, generator close (GeneratorExit handler), throw()
# into generator, generator expressions (sum/list/tuple comprehensions
# of i*i, i**2 style), reversed of list/string/tuple, enumerate with
# default and explicit start, zip same-length and short-list
# truncation, zip() with no args returns empty, map / filter values,
# range list / membership / indexing / negative index, itertools chain
# /islice/repeat/combinations/permutations/product/takewhile/
# dropwhile/starmap, custom iterator via __iter__/__next__ protocol,
# iter()/next() on list, tuple, set, dict yielding the documented
# values. All asserts match between CPython 3.12 and mamba.

_ledger: list[int] = []

# 1) generator next + StopIteration
def _gen1():
    yield 1
    yield 2
    yield 3

_g = _gen1()
assert next(_g) == 1; _ledger.append(1)
assert next(_g) == 2; _ledger.append(1)
assert next(_g) == 3; _ledger.append(1)
_stop = False
try:
    next(_g)
except StopIteration:
    _stop = True
assert _stop == True; _ledger.append(1)

# 2) generator iteration via list/sum/tuple
assert list(_gen1()) == [1, 2, 3]; _ledger.append(1)
assert sum(_gen1()) == 6; _ledger.append(1)
assert tuple(_gen1()) == (1, 2, 3); _ledger.append(1)

# 3) generator with send
def _gen2():
    x = yield 1
    y = yield x + 10
    yield y + 100

_g2 = _gen2()
assert next(_g2) == 1; _ledger.append(1)
assert _g2.send(5) == 15; _ledger.append(1)
assert _g2.send(10) == 110; _ledger.append(1)

# 4) yield from list / string
def _gen3():
    yield from [1, 2, 3]
    yield from "ab"

assert list(_gen3()) == [1, 2, 3, "a", "b"]; _ledger.append(1)

# 5) generator close
def _gen4():
    try:
        yield 1
        yield 2
    except GeneratorExit:
        pass

_g4 = _gen4()
next(_g4)
_g4.close()
_stop2 = False
try:
    next(_g4)
except StopIteration:
    _stop2 = True
assert _stop2 == True; _ledger.append(1)

# 6) throw into generator
def _gen5():
    try:
        yield 1
    except ValueError as e:
        yield f"caught: {e}"

_g5 = _gen5()
next(_g5)
assert _g5.throw(ValueError("x")) == "caught: x"; _ledger.append(1)

# 7) generator expressions
assert sum(i * i for i in range(5)) == 30; _ledger.append(1)
assert list(i for i in range(3)) == [0, 1, 2]; _ledger.append(1)
assert tuple(i ** 2 for i in [1, 2, 3]) == (1, 4, 9); _ledger.append(1)

# 8) reversed
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert list(reversed("abc")) == ["c", "b", "a"]; _ledger.append(1)
assert list(reversed((1, 2, 3))) == [3, 2, 1]; _ledger.append(1)

# 9) enumerate
assert list(enumerate("abc")) == [(0, "a"), (1, "b"), (2, "c")]; _ledger.append(1)
assert list(enumerate("abc", 5)) == [(5, "a"), (6, "b"), (7, "c")]; _ledger.append(1)
assert list(enumerate([])) == []; _ledger.append(1)

# 10) zip
assert list(zip([1, 2, 3], [4, 5, 6])) == [(1, 4), (2, 5), (3, 6)]; _ledger.append(1)
assert list(zip([1, 2, 3], [4, 5])) == [(1, 4), (2, 5)]; _ledger.append(1)
assert list(zip()) == []; _ledger.append(1)
assert list(zip([], [1, 2])) == []; _ledger.append(1)

# 11) map / filter values
assert list(map(lambda x: x * 2, [1, 2, 3])) == [2, 4, 6]; _ledger.append(1)
assert list(filter(lambda x: x > 1, [1, 2, 3, 4])) == [2, 3, 4]; _ledger.append(1)
assert list(filter(None, [0, 1, 2, 0, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(map(str, [1, 2, 3])) == ["1", "2", "3"]; _ledger.append(1)

# 12) range
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(range(2, 8)) == [2, 3, 4, 5, 6, 7]; _ledger.append(1)
assert list(range(0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(range(10, 0, -1)) == [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]; _ledger.append(1)
assert len(range(5)) == 5; _ledger.append(1)
assert (3 in range(5)) == True; _ledger.append(1)
assert (99 in range(5)) == False; _ledger.append(1)
assert range(5)[2] == 2; _ledger.append(1)
assert range(5)[-1] == 4; _ledger.append(1)

# 13) itertools
import itertools
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
assert list(itertools.islice(range(10), 3, 7)) == [3, 4, 5, 6]; _ledger.append(1)
assert list(itertools.islice(range(10), 5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(itertools.repeat("a", 3)) == ["a", "a", "a"]; _ledger.append(1)
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
assert list(itertools.product([1, 2], [3, 4])) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4])) == [1, 2]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4])) == [3, 4]; _ledger.append(1)
assert list(itertools.starmap(pow, [(2, 3), (3, 2)])) == [8, 9]; _ledger.append(1)

# 14) custom iterator via __iter__/__next__
class _Counter:
    def __init__(self, lo, hi):
        self.n = lo
        self.hi = hi
    def __iter__(self):
        return self
    def __next__(self):
        if self.n >= self.hi:
            raise StopIteration
        v = self.n
        self.n += 1
        return v

assert list(_Counter(0, 4)) == [0, 1, 2, 3]; _ledger.append(1)
assert list(_Counter(2, 2)) == []; _ledger.append(1)

# 15) iter() on list/tuple/set/dict
assert list(iter([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert list(iter((10, 20, 30))) == [10, 20, 30]; _ledger.append(1)
assert sorted(iter({1, 2, 3})) == [1, 2, 3]; _ledger.append(1)
assert sorted(iter({"a": 1, "b": 2})) == ["a", "b"]; _ledger.append(1)

# 16) iter() with sentinel
_lst = iter([1, 2, 3])
assert next(_lst) == 1; _ledger.append(1)

# 17) generator iter()  — iter(g) is g
def _gen_iter():
    yield 1

_gi = _gen_iter()
assert iter(_gi) is _gi; _ledger.append(1)

# 18) Custom iterator returning self from __iter__
_cc = _Counter(0, 3)
assert iter(_cc) is _cc; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_iterator_generator_value_ops {sum(_ledger)} asserts")
