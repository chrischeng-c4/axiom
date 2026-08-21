# Atomic 231 pass conformance — cmath/math/itertools/operator/collections/
# functools value ops + hasattr that match between CPython 3.12 and mamba.
import cmath
import math
import itertools
import operator
import collections
import functools
import string

_ledger: list[int] = []

# 1) cmath — value ops + core surface hasattr
assert cmath.sqrt(-1) == 1j; _ledger.append(1)
assert cmath.pi == 3.141592653589793; _ledger.append(1)
assert cmath.e == 2.718281828459045; _ledger.append(1)
assert cmath.exp(0) == (1+0j); _ledger.append(1)
assert cmath.isnan(complex(0, 0)) == False; _ledger.append(1)
assert cmath.isinf(complex(0, 0)) == False; _ledger.append(1)
assert cmath.isfinite(complex(0, 0)) == True; _ledger.append(1)
assert cmath.phase(complex(0, 1)) == 1.5707963267948966; _ledger.append(1)
assert cmath.polar(complex(1, 0)) == (1.0, 0.0); _ledger.append(1)
assert cmath.rect(1, 0) == (1+0j); _ledger.append(1)
assert hasattr(cmath, "sqrt") == True; _ledger.append(1)
assert hasattr(cmath, "tau") == True; _ledger.append(1)
assert hasattr(cmath, "inf") == True; _ledger.append(1)
assert hasattr(cmath, "nan") == True; _ledger.append(1)
assert hasattr(cmath, "infj") == True; _ledger.append(1)
assert hasattr(cmath, "nanj") == True; _ledger.append(1)
assert hasattr(cmath, "log") == True; _ledger.append(1)
assert hasattr(cmath, "log10") == True; _ledger.append(1)
assert hasattr(cmath, "cos") == True; _ledger.append(1)
assert hasattr(cmath, "sin") == True; _ledger.append(1)
assert hasattr(cmath, "tan") == True; _ledger.append(1)
assert hasattr(cmath, "acos") == True; _ledger.append(1)
assert hasattr(cmath, "asin") == True; _ledger.append(1)
assert hasattr(cmath, "atan") == True; _ledger.append(1)
assert hasattr(cmath, "cosh") == True; _ledger.append(1)
assert hasattr(cmath, "sinh") == True; _ledger.append(1)
assert hasattr(cmath, "tanh") == True; _ledger.append(1)
assert hasattr(cmath, "isclose") == True; _ledger.append(1)

# 2) math — value ops (number theory + basic arithmetic + trig identities)
assert math.comb(5, 2) == 10; _ledger.append(1)
assert math.perm(5, 2) == 20; _ledger.append(1)
assert math.gcd(12, 18) == 6; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.isqrt(10) == 3; _ledger.append(1)
assert math.prod([1, 2, 3, 4]) == 24; _ledger.append(1)
assert math.fsum([0.1] * 10) == 1.0; _ledger.append(1)
assert math.dist([0, 0], [3, 4]) == 5.0; _ledger.append(1)
assert math.hypot(3, 4) == 5.0; _ledger.append(1)
assert math.degrees(math.pi) == 180.0; _ledger.append(1)
assert math.radians(180) == 3.141592653589793; _ledger.append(1)
assert math.floor(2.7) == 2; _ledger.append(1)
assert math.ceil(2.3) == 3; _ledger.append(1)
assert math.trunc(2.7) == 2; _ledger.append(1)
assert math.fabs(-5) == 5.0; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.copysign(3, -1) == -3.0; _ledger.append(1)
assert math.fmod(10, 3) == 1.0; _ledger.append(1)

# 3) itertools — value ops on finite outputs
assert list(itertools.chain([1, 2], [3, 4])) == [1, 2, 3, 4]; _ledger.append(1)
assert list(itertools.product([1, 2], [3, 4])) == [(1, 3), (1, 4), (2, 3), (2, 4)]; _ledger.append(1)
assert list(itertools.permutations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 1), (2, 3), (3, 1), (3, 2)]; _ledger.append(1)
assert list(itertools.combinations([1, 2, 3], 2)) == [(1, 2), (1, 3), (2, 3)]; _ledger.append(1)
assert list(itertools.combinations_with_replacement([1, 2], 2)) == [(1, 1), (1, 2), (2, 2)]; _ledger.append(1)
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10]; _ledger.append(1)
assert list(itertools.compress([1, 2, 3, 4], [1, 0, 1, 0])) == [1, 3]; _ledger.append(1)
assert list(itertools.dropwhile(lambda x: x < 3, [1, 2, 3, 4, 1, 2])) == [3, 4, 1, 2]; _ledger.append(1)
assert list(itertools.takewhile(lambda x: x < 3, [1, 2, 3, 4, 1, 2])) == [1, 2]; _ledger.append(1)
assert list(itertools.filterfalse(lambda x: x < 3, [1, 2, 3, 4])) == [3, 4]; _ledger.append(1)
assert list(itertools.islice(range(10), 2, 8, 2)) == [2, 4, 6]; _ledger.append(1)
assert list(itertools.starmap(pow, [(2, 3), (3, 2)])) == [8, 9]; _ledger.append(1)
assert list(itertools.zip_longest([1, 2, 3], ["a", "b"], fillvalue="-")) == [(1, "a"), (2, "b"), (3, "-")]; _ledger.append(1)
assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)]; _ledger.append(1)
assert list(itertools.batched([1, 2, 3, 4, 5, 6, 7], 3)) == [(1, 2, 3), (4, 5, 6), (7,)]; _ledger.append(1)
assert list(itertools.repeat("x", 3)) == ["x", "x", "x"]; _ledger.append(1)

# 4) operator — value ops
assert operator.add(2, 3) == 5; _ledger.append(1)
assert operator.sub(5, 3) == 2; _ledger.append(1)
assert operator.mul(4, 5) == 20; _ledger.append(1)
assert operator.floordiv(10, 3) == 3; _ledger.append(1)
assert operator.mod(10, 3) == 1; _ledger.append(1)
assert operator.pow(2, 3) == 8; _ledger.append(1)
assert operator.neg(5) == -5; _ledger.append(1)
assert operator.eq(3, 3) == True; _ledger.append(1)
assert operator.lt(2, 3) == True; _ledger.append(1)
assert operator.gt(3, 2) == True; _ledger.append(1)
assert operator.and_(0b1100, 0b1010) == 8; _ledger.append(1)
assert operator.or_(0b1100, 0b1010) == 14; _ledger.append(1)
assert operator.xor(0b1100, 0b1010) == 6; _ledger.append(1)
assert operator.invert(5) == -6; _ledger.append(1)
assert operator.lshift(1, 3) == 8; _ledger.append(1)
assert operator.rshift(8, 1) == 4; _ledger.append(1)
assert operator.not_(False) == True; _ledger.append(1)
assert operator.truth(0) == False; _ledger.append(1)
assert operator.is_(None, None) == True; _ledger.append(1)
assert operator.contains([1, 2, 3], 2) == True; _ledger.append(1)

# 5) collections — Counter / OrderedDict / defaultdict / deque / ChainMap value ops
_ctr = collections.Counter("aabcbc")
assert dict(_ctr) == {"a": 2, "b": 2, "c": 2}; _ledger.append(1)
assert _ctr.most_common(1) == [("a", 2)]; _ledger.append(1)
_od = collections.OrderedDict([("a", 1), ("b", 2)])
assert list(_od.keys()) == ["a", "b"]; _ledger.append(1)
_dd: dict = collections.defaultdict(int)
_dd["x"] += 1
_dd["x"] += 1
assert _dd["x"] == 2; _ledger.append(1)
_dq = collections.deque([1, 2, 3])
_dq.appendleft(0)
_dq.append(4)
assert list(_dq) == [0, 1, 2, 3, 4]; _ledger.append(1)
_cm = collections.ChainMap({"a": 1}, {"b": 2, "a": 3})
assert _cm["a"] == 1; _ledger.append(1)
assert _cm["b"] == 2; _ledger.append(1)

# 6) functools — reduce + partial value ops
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]) == 10; _ledger.append(1)
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4]) == 24; _ledger.append(1)
assert functools.partial(pow, 2)(3) == 8; _ledger.append(1)
assert functools.partial(pow, 2)(10) == 1024; _ledger.append(1)

# 7) string — capwords value op + ASCII constants
assert string.capwords("hello world") == "Hello World"; _ledger.append(1)
assert string.capwords("foo bar baz") == "Foo Bar Baz"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_cmath_math_itertools_operator_collections_value_ops {sum(_ledger)} asserts")
