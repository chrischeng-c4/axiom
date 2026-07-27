use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/random/binomialvariate_stays_in_range.py`.
#[test]
fn test_gen_behavior_std_libs_random_binomialvariate_stays_in_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "binomialvariate_stays_in_range"
# subject = "random.Random.binomialvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.binomialvariate: binomialvariate(5, 0.25) stays within [0, 5] for ordinary parameters across 50 seeded draws"""
import random

gen = random.Random(0)
for _ in range(50):
    assert gen.binomialvariate(5, 0.25) in range(6), "binomial out of [0,5]"

print("binomialvariate_stays_in_range OK")
"###);
    assert_output(&out, r###"binomialvariate_stays_in_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choice_covers_whole_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_random_choice_covers_whole_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choice_covers_whole_sequence"
# subject = "random.choice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choice: choice draws only from the sequence and, over enough seeded draws, covers every element: choice(['a','b','c']) eventually yields all three"""
import random

random.seed(3)
items = ["a", "b", "c"]
choices = {random.choice(items) for _ in range(50)}
assert choices == {"a", "b", "c"}, f"choice covers all: {choices!r}"

print("choice_covers_whole_sequence OK")
"###);
    assert_output(&out, r###"choice_covers_whole_sequence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_cum_weights_equivalent.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_cum_weights_equivalent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_cum_weights_equivalent"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: cum_weights is the prefix-sum form of weights and selects identically: choices('abcd', cum_weights=[1,1,1,1]) is ['a'] and cum_weights=[0,0,0,1] is ['d']"""
import random

gen = random.Random(0)
assert gen.choices("abcd", cum_weights=[1, 1, 1, 1]) == ["a"], "cum pick first"
assert gen.choices("abcd", cum_weights=[0, 0, 0, 1]) == ["d"], "cum pick last"

print("choices_cum_weights_equivalent OK")
"###);
    assert_output(&out, r###"choices_cum_weights_equivalent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_mixed_numeric_weight_types.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_mixed_numeric_weight_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_mixed_numeric_weight_types"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: int, float, and bool weight vectors are all accepted: choices('abcd', weights, k=5) draws only from 'abcd' for each numeric weight kind"""
import random

gen = random.Random(0)
for weights in ([15, 10, 25, 30], [15.1, 10.2, 25.2, 30.3], [True, False, True, False]):
    sample = gen.choices("abcd", weights, k=5)
    assert set(sample) <= set("abcd"), f"mixed weights {weights!r} -> {sample!r}"

print("choices_mixed_numeric_weight_types OK")
"###);
    assert_output(&out, r###"choices_mixed_numeric_weight_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_nonpositive_k_returns_empty.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_nonpositive_k_returns_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_nonpositive_k_returns_empty"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: k == 0 and k < 0 both yield an empty list: choices('abcd', k=0) and choices('abcd', k=-1) are both []"""
import random

gen = random.Random(0)
assert gen.choices("abcd", k=0) == [], "k=0 empty"
assert gen.choices("abcd", k=-1) == [], "k<0 empty"

print("choices_nonpositive_k_returns_empty OK")
"###);
    assert_output(&out, r###"choices_nonpositive_k_returns_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_single_weight_is_deterministic.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_single_weight_is_deterministic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_single_weight_is_deterministic"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: a weight vector targeting one element forces that draw: choices('abcd', [1,0,0,0]) is ['a'] and choices('abcd', [0,0,0,1]) is ['d']"""
import random

gen = random.Random(0)
assert gen.choices("abcd", [1, 0, 0, 0]) == ["a"], "weights pick first"
assert gen.choices("abcd", [0, 0, 0, 1]) == ["d"], "weights pick last"

print("choices_single_weight_is_deterministic OK")
"###);
    assert_output(&out, r###"choices_single_weight_is_deterministic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_uniform_draws_from_population.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_uniform_draws_from_population() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_uniform_draws_from_population"
# subject = "random.Random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.choices: uniform choices (no weights) return a list of length k drawn only from the population: choices('abcd', k=5) is a 5-element list whose elements are all in 'abcd'"""
import random

gen = random.Random(0)
out = gen.choices("abcd", k=5)
assert len(out) == 5, f"len = {len(out)!r}"
assert type(out) is list, f"type = {type(out)!r}"
assert set(out) <= set("abcd"), f"out = {out!r}"

print("choices_uniform_draws_from_population OK")
"###);
    assert_output(&out, r###"choices_uniform_draws_from_population OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/choices_weights_dominate_distribution.py`.
#[test]
fn test_gen_behavior_std_libs_random_choices_weights_dominate_distribution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "choices_weights_dominate_distribution"
# subject = "random.choices"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: weights bias the distribution: choices([0,1], weights=[1,99], k=100) yields ones for the large majority (count of 1 exceeds 80)"""
import random

random.seed(7)
weighted = random.choices([0, 1], weights=[1, 99], k=100)
# With weight 99x, 1 should dominate.
assert weighted.count(1) > 80, f"weighted choices: {weighted.count(1)}/100 ones"

print("choices_weights_dominate_distribution OK")
"###);
    assert_output(&out, r###"choices_weights_dominate_distribution OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/degenerate_params_collapse_to_constant.py`.
#[test]
fn test_gen_behavior_std_libs_random_degenerate_params_collapse_to_constant() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "degenerate_params_collapse_to_constant"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: degenerate parameters collapse distributions to a constant: uniform(10,10)=10, triangular(10,10,10)=10, gauss(10,0)=10, expovariate(inf)=0, binomialvariate(10,1.0)=10, etc."""
import random

gen = random.Random(0)
constants = [
    (gen.uniform, (10.0, 10.0), 10.0),
    (gen.triangular, (10.0, 10.0, 10.0), 10.0),
    (gen.gauss, (10.0, 0.0), 10.0),
    (gen.normalvariate, (10.0, 0.0), 10.0),
    (gen.expovariate, (float("inf"),), 0.0),
    (gen.paretovariate, (float("inf"),), 1.0),
    (gen.binomialvariate, (10, 0.0), 0),
    (gen.binomialvariate, (10, 1.0), 10),
]
for variate, args, expected in constants:
    assert variate(*args) == expected, f"{variate.__name__}{args} != {expected!r}"

print("degenerate_params_collapse_to_constant OK")
"###);
    assert_output(&out, r###"degenerate_params_collapse_to_constant OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/getrandbits_returns_bounded_int.py`.
#[test]
fn test_gen_behavior_std_libs_random_getrandbits_returns_bounded_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "getrandbits_returns_bounded_int"
# subject = "random.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.getrandbits: getrandbits(16) returns an int in [0, 2**16): the result is an int and 0 <= bits < 65536"""
import random

random.seed(8)
bits = random.getrandbits(16)
assert isinstance(bits, int), f"getrandbits type = {type(bits)!r}"
assert 0 <= bits < 2 ** 16, f"getrandbits 16-bit: {bits!r}"

print("getrandbits_returns_bounded_int OK")
"###);
    assert_output(&out, r###"getrandbits_returns_bounded_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/getstate_setstate_round_trips.py`.
#[test]
fn test_gen_behavior_std_libs_random_getstate_setstate_round_trips() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "getstate_setstate_round_trips"
# subject = "random.Random.getstate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.getstate: getstate snapshots the generator and setstate rewinds it exactly: draws taken before and after restoring the snapshot are equal"""
import random

gen = random.Random(12345)

# getstate snapshots the generator; setstate rewinds it exactly.
snapshot = gen.getstate()
before = [gen.random() for _ in range(5)]
gen.setstate(snapshot)
after = [gen.random() for _ in range(5)]
assert before == after, f"setstate replay: {before!r} != {after!r}"

print("getstate_setstate_round_trips OK")
"###);
    assert_output(&out, r###"getstate_setstate_round_trips OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_bug_31478.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_bug_31478() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_bug_31478"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_bug_31478"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_bug_31478
"""Auto-ported test: MersenneTwister_TestBasicOps::test_bug_31478 (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()

class BadInt(int):

    def __abs__(self):
        return None
try:
    gen.seed(BadInt())
except TypeError:
    pass
print("MersenneTwister_TestBasicOps::test_bug_31478: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_bug_31478: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_choices_algorithms.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_choices_algorithms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_choices_algorithms"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_choices_algorithms"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_choices_algorithms
"""Auto-ported test: MersenneTwister_TestBasicOps::test_choices_algorithms (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
choices = gen.choices
n = 104729
gen.seed(8675309)
a = gen.choices(range(n), k=10000)
gen.seed(8675309)
b = gen.choices(range(n), [1] * n, k=10000)

assert a == b
gen.seed(8675309)
c = gen.choices(range(n), cum_weights=range(1, n + 1), k=10000)

assert a == c
population = ['Red', 'Black', 'Green']
weights = [18, 18, 2]
cum_weights = [18, 36, 38]
expanded_population = ['Red'] * 18 + ['Black'] * 18 + ['Green'] * 2
gen.seed(9035768)
a = gen.choices(expanded_population, k=10000)
gen.seed(9035768)
b = gen.choices(population, weights, k=10000)

assert a == b
gen.seed(9035768)
c = gen.choices(population, cum_weights=cum_weights, k=10000)

assert a == c
print("MersenneTwister_TestBasicOps::test_choices_algorithms: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_choices_algorithms: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_choices_subnormal.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_choices_subnormal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_choices_subnormal"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_choices_subnormal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_choices_subnormal
"""Auto-ported test: MersenneTwister_TestBasicOps::test_choices_subnormal (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
choices = gen.choices
choices(population=[1, 2], weights=[1e-323, 1e-323], k=5000)
print("MersenneTwister_TestBasicOps::test_choices_subnormal: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_choices_subnormal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_gauss.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_gauss() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_gauss"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_gauss"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_gauss
"""Auto-ported test: MersenneTwister_TestBasicOps::test_gauss (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
for seed in (1, 12, 123, 1234, 12345, 123456, 654321):
    gen.seed(seed)
    x1 = gen.random()
    y1 = gen.gauss(0, 1)
    gen.seed(seed)
    x2 = gen.random()
    y2 = gen.gauss(0, 1)

    assert x1 == x2

    assert y1 == y2
print("MersenneTwister_TestBasicOps::test_gauss: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_gauss: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_long_seed.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_long_seed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_long_seed"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_long_seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_long_seed
"""Auto-ported test: MersenneTwister_TestBasicOps::test_long_seed (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
seed = (1 << 10000 * 8) - 1
gen.seed(seed)
print("MersenneTwister_TestBasicOps::test_long_seed: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_long_seed: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_mu_sigma_default_args.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_mu_sigma_default_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_mu_sigma_default_args"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_mu_sigma_default_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_mu_sigma_default_args
"""Auto-ported test: MersenneTwister_TestBasicOps::test_mu_sigma_default_args (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()

assert isinstance(gen.normalvariate(), float)

assert isinstance(gen.gauss(), float)
print("MersenneTwister_TestBasicOps::test_mu_sigma_default_args: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_mu_sigma_default_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_sample_on_dicts.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_sample_on_dicts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_sample_on_dicts"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_sample_on_dicts"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_sample_on_dicts
"""Auto-ported test: MersenneTwister_TestBasicOps::test_sample_on_dicts (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()

try:
    gen.sample(dict.fromkeys('abcdef'), 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("MersenneTwister_TestBasicOps::test_sample_on_dicts: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_sample_on_dicts: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/mersenne_twister_test_basic_ops__test_seed_no_mutate_bug_44018.py`.
#[test]
fn test_gen_behavior_std_libs_random_mersenne_twister_test_basic_ops__test_seed_no_mutate_bug_44018() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "mersenne_twister_test_basic_ops__test_seed_no_mutate_bug_44018"
# subject = "cpython.test_random.MersenneTwister_TestBasicOps.test_seed_no_mutate_bug_44018"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::MersenneTwister_TestBasicOps::test_seed_no_mutate_bug_44018
"""Auto-ported test: MersenneTwister_TestBasicOps::test_seed_no_mutate_bug_44018 (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.Random()
a = bytearray(b'1234')
gen.seed(a)

assert a == bytearray(b'1234')
print("MersenneTwister_TestBasicOps::test_seed_no_mutate_bug_44018: ok")
"###);
    assert_output(&out, r###"MersenneTwister_TestBasicOps::test_seed_no_mutate_bug_44018: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/module_helpers_share_default_generator.py`.
#[test]
fn test_gen_behavior_std_libs_random_module_helpers_share_default_generator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "module_helpers_share_default_generator"
# subject = "random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.seed: module-level helpers share one default generator: random.seed(2024) then [random.random() for _ in range(5)] replays identically after re-seeding"""
import random

random.seed(2024)
a = [random.random() for _ in range(5)]
random.seed(2024)
b = [random.random() for _ in range(5)]
assert a == b, f"module reseed replay: {a!r} != {b!r}"

print("module_helpers_share_default_generator OK")
"###);
    assert_output(&out, r###"module_helpers_share_default_generator OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/no_arg_seed_autoseeds_fresh_state.py`.
#[test]
fn test_gen_behavior_std_libs_random_no_arg_seed_autoseeds_fresh_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "no_arg_seed_autoseeds_fresh_state"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: seeding with no argument auto-seeds from an OS entropy source: two consecutive getstate() snapshots after seed() differ"""
import random

gen = random.Random(12345)

# Seeding with no argument auto-seeds, producing a fresh state each time.
gen.seed()
auto1 = gen.getstate()
gen.seed()
auto2 = gen.getstate()
assert auto1 != auto2, "autoseed should differ"

print("no_arg_seed_autoseeds_fresh_state OK")
"###);
    assert_output(&out, r###"no_arg_seed_autoseeds_fresh_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/normalvariate_gauss_default_args.py`.
#[test]
fn test_gen_behavior_std_libs_random_normalvariate_gauss_default_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "normalvariate_gauss_default_args"
# subject = "random.Random.normalvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.normalvariate: normalvariate() and gauss() accept default mu/sigma and return floats"""
import random

gen = random.Random(0)
assert isinstance(gen.normalvariate(), float), "normalvariate() not float"
assert isinstance(gen.gauss(), float), "gauss() not float"

print("normalvariate_gauss_default_args OK")
"###);
    assert_output(&out, r###"normalvariate_gauss_default_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/randbytes_returns_exact_length.py`.
#[test]
fn test_gen_behavior_std_libs_random_randbytes_returns_exact_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randbytes_returns_exact_length"
# subject = "random.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randbytes: module-level randbytes(n) returns exactly n bytes (a bytes object) and randbytes(0) returns b''"""
import random

random.seed(0)
data = random.randbytes(8)
assert type(data) is bytes and len(data) == 8, f"randbytes = {data!r}"
assert random.randbytes(0) == b"", "randbytes(0)"

print("randbytes_returns_exact_length OK")
"###);
    assert_output(&out, r###"randbytes_returns_exact_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/randint_endpoints_inclusive.py`.
#[test]
fn test_gen_behavior_std_libs_random_randint_endpoints_inclusive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randint_endpoints_inclusive"
# subject = "random.randint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randint: randint endpoints are inclusive: seeded random.randint(1, 2) over 100 draws produces both 1 and 2"""
import random

random.seed(1)
vals = {random.randint(1, 2) for _ in range(100)}
assert 1 in vals and 2 in vals, f"both endpoints seen: {vals!r}"

print("randint_endpoints_inclusive OK")
"###);
    assert_output(&out, r###"randint_endpoints_inclusive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/random_instance_pickle_round_trips.py`.
#[test]
fn test_gen_behavior_std_libs_random_random_instance_pickle_round_trips() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "random_instance_pickle_round_trips"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: pickling a Random preserves generator state: a dumps/loads round-trip reproduces the same 10-draw stream as the original"""
import random

import pickle

src = random.Random(7)
blob = pickle.dumps(src)
orig = [src.random() for _ in range(10)]
restored = pickle.loads(blob)
assert orig == [restored.random() for _ in range(10)], "pickle replay differs"

print("random_instance_pickle_round_trips OK")
"###);
    assert_output(&out, r###"random_instance_pickle_round_trips OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/random_instances_same_seed_same_stream.py`.
#[test]
fn test_gen_behavior_std_libs_random_random_instances_same_seed_same_stream() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "random_instances_same_seed_same_stream"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: two independent Random(10) instances are deterministic and identical: each produces the same [randint(0,100) for _ in range(5)] sequence"""
import random

rng1 = random.Random(10)
rng2 = random.Random(10)
from1 = [rng1.randint(0, 100) for _ in range(5)]
from2 = [rng2.randint(0, 100) for _ in range(5)]
assert from1 == from2, f"same seed = same seq: {from1!r} != {from2!r}"

print("random_instances_same_seed_same_stream OK")
"###);
    assert_output(&out, r###"random_instances_same_seed_same_stream OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/randrange_excludes_stop.py`.
#[test]
fn test_gen_behavior_std_libs_random_randrange_excludes_stop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "randrange_excludes_stop"
# subject = "random.randrange"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randrange: randrange(5) yields values in the half-open range [0, 5): all draws satisfy 0 <= v < 5 and the top value 4 does appear"""
import random

random.seed(2)
rr = [random.randrange(5) for _ in range(50)]
assert all(0 <= v < 5 for v in rr), f"randrange [0,5): {rr!r}"
assert 4 in rr, "4 should appear"

print("randrange_excludes_stop OK")
"###);
    assert_output(&out, r###"randrange_excludes_stop OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/sample_counts_respects_multiplicity_caps.py`.
#[test]
fn test_gen_behavior_std_libs_random_sample_counts_respects_multiplicity_caps() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_counts_respects_multiplicity_caps"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: counts expands each element by its multiplicity and respects the caps: sampling k=700 from colors with counts gives sum 700, each color within its cap, and a zero-count element excluded"""
import random

from collections import Counter

gen = random.Random(0)
colors = ["red", "green", "blue", "orange", "black", "brown", "amber"]
counts = [500, 200, 20, 10, 5, 0, 1]
summary = Counter(gen.sample(colors, counts=counts, k=700))
assert sum(summary.values()) == 700, f"total = {sum(summary.values())!r}"
for color, cap in zip(colors, counts):
    assert summary[color] <= cap, f"{color}: {summary[color]} > {cap}"
assert "brown" not in summary, "zero-count element excluded"

print("sample_counts_respects_multiplicity_caps OK")
"###);
    assert_output(&out, r###"sample_counts_respects_multiplicity_caps OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/sample_has_no_duplicates.py`.
#[test]
fn test_gen_behavior_std_libs_random_sample_has_no_duplicates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "sample_has_no_duplicates"
# subject = "random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.sample: sample draws without replacement: sample(range(20), 10) returns 10 distinct elements, all drawn from the pool"""
import random

random.seed(4)
pool = list(range(20))
s = random.sample(pool, 10)
assert len(s) == 10, f"sample len = {len(s)!r}"
assert len(set(s)) == 10, "sample no duplicates"
assert all(v in pool for v in s), "sample from pool"

print("sample_has_no_duplicates OK")
"###);
    assert_output(&out, r###"sample_has_no_duplicates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/seed_accepts_scalar_str_bytes_types.py`.
#[test]
fn test_gen_behavior_std_libs_random_seed_accepts_scalar_str_bytes_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_accepts_scalar_str_bytes_types"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: Random().seed accepts None / int (incl. huge & negative) / bool / float / str / bytes seeds without raising, and a bytes seed reproduces the same first draw"""
import random

gen = random.Random()

# Hashable scalars and byte strings are all valid seeds.
for arg in [None, 0, 1, -1, 10 ** 20, -10 ** 20, False, True, 3.14, "a", b"xy"]:
    gen.seed(arg)

# Same seed -> same first draw, confirming the seed actually took effect.
gen.seed(b"xy")
first = gen.random()
gen.seed(b"xy")
assert gen.random() == first, "bytes seed not reproducible"

print("seed_accepts_scalar_str_bytes_types OK")
"###);
    assert_output(&out, r###"seed_accepts_scalar_str_bytes_types OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/seed_does_not_mutate_bytearray_buffer.py`.
#[test]
fn test_gen_behavior_std_libs_random_seed_does_not_mutate_bytearray_buffer() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_does_not_mutate_bytearray_buffer"
# subject = "random.Random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.seed: seeding from a bytearray must not mutate the caller's buffer (bug 44018): seed(bytearray(b'1234')) leaves the bytearray unchanged"""
import random

gen = random.Random()

# Seeding from a bytearray must not mutate the caller's buffer (bug 44018).
buf = bytearray(b"1234")
gen.seed(buf)
assert buf == bytearray(b"1234"), f"seed mutated buffer: {buf!r}"

print("seed_does_not_mutate_bytearray_buffer OK")
"###);
    assert_output(&out, r###"seed_does_not_mutate_bytearray_buffer OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/seed_makes_stream_reproducible.py`.
#[test]
fn test_gen_behavior_std_libs_random_seed_makes_stream_reproducible() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "seed_makes_stream_reproducible"
# subject = "random.seed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.seed: re-seeding the module RNG with the same value (99) reproduces the same randint sequence; two seeded runs of [randint(0,100) for _ in range(5)] are equal"""
import random

random.seed(99)
a = [random.randint(0, 100) for _ in range(5)]
random.seed(99)
b = [random.randint(0, 100) for _ in range(5)]
assert a == b, f"seed reproducible: {a!r} != {b!r}"

print("seed_makes_stream_reproducible OK")
"###);
    assert_output(&out, r###"seed_makes_stream_reproducible OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/shuffle_is_in_place_permutation.py`.
#[test]
fn test_gen_behavior_std_libs_random_shuffle_is_in_place_permutation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "shuffle_is_in_place_permutation"
# subject = "random.shuffle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.shuffle: shuffle mutates the list in place and is a permutation: after shuffling list(range(5)) the sorted result is still [0,1,2,3,4]"""
import random

random.seed(5)
lst = list(range(5))
random.shuffle(lst)
assert sorted(lst) == [0, 1, 2, 3, 4], f"shuffle preserves elements: {lst!r}"

print("shuffle_is_in_place_permutation OK")
"###);
    assert_output(&out, r###"shuffle_is_in_place_permutation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_getrandbits_widths.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_getrandbits_widths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_getrandbits_widths"
# subject = "random.SystemRandom.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.getrandbits: SystemRandom.getrandbits(k) yields a k-bit non-negative integer for k in {1,8,32,64,128} and getrandbits(0) == 0"""
import random

gen = random.SystemRandom()
for k in (1, 8, 32, 64, 128):
    v = gen.getrandbits(k)
    assert 0 <= v < 2 ** k, f"getrandbits({k}) = {v!r}"
assert gen.getrandbits(0) == 0, "getrandbits(0)"

print("system_random_getrandbits_widths OK")
"###);
    assert_output(&out, r###"system_random_getrandbits_widths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_randbytes_lengths.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_randbytes_lengths() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_randbytes_lengths"
# subject = "random.SystemRandom.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.randbytes: SystemRandom.randbytes(n) returns exactly n bytes for n in 1..9 and randbytes(0) == b''"""
import random

gen = random.SystemRandom()
for n in range(1, 10):
    data = gen.randbytes(n)
    assert type(data) is bytes and len(data) == n, f"randbytes({n}) = {data!r}"
assert gen.randbytes(0) == b"", "randbytes(0)"

print("system_random_randbytes_lengths OK")
"###);
    assert_output(&out, r###"system_random_randbytes_lengths OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_random_and_randint.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_random_and_randint() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_random_and_randint"
# subject = "random.SystemRandom"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom: SystemRandom.random() stays in [0, 1) and randint endpoints are inclusive (degenerate randint(5,5) == 5)"""
import random

gen = random.SystemRandom()
r = gen.random()
assert isinstance(r, float) and 0.0 <= r < 1.0, f"random = {r!r}"
assert gen.randint(5, 5) == 5, "degenerate randint"

print("system_random_random_and_randint OK")
"###);
    assert_output(&out, r###"system_random_random_and_randint OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_autoseed.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_autoseed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_autoseed"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_autoseed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_autoseed
"""Auto-ported test: SystemRandom_TestBasicOps::test_autoseed (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
gen.seed()
print("SystemRandom_TestBasicOps::test_autoseed: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_autoseed: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_choices_subnormal.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_choices_subnormal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_choices_subnormal"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_choices_subnormal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_choices_subnormal
"""Auto-ported test: SystemRandom_TestBasicOps::test_choices_subnormal (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
choices = gen.choices
choices(population=[1, 2], weights=[1e-323, 1e-323], k=5000)
print("SystemRandom_TestBasicOps::test_choices_subnormal: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_choices_subnormal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_gauss.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_gauss() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_gauss"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_gauss"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_gauss
"""Auto-ported test: SystemRandom_TestBasicOps::test_gauss (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
gen.gauss_next = None
gen.seed(100)

assert gen.gauss_next == None
print("SystemRandom_TestBasicOps::test_gauss: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_gauss: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_mu_sigma_default_args.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_mu_sigma_default_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_mu_sigma_default_args"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_mu_sigma_default_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_mu_sigma_default_args
"""Auto-ported test: SystemRandom_TestBasicOps::test_mu_sigma_default_args (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()

assert isinstance(gen.normalvariate(), float)

assert isinstance(gen.gauss(), float)
print("SystemRandom_TestBasicOps::test_mu_sigma_default_args: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_mu_sigma_default_args: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_randrange_nonunit_step.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_randrange_nonunit_step() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_randrange_nonunit_step"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_randrange_nonunit_step"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_randrange_nonunit_step
"""Auto-ported test: SystemRandom_TestBasicOps::test_randrange_nonunit_step (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
rint = gen.randrange(0, 10, 2)

assert rint in (0, 2, 4, 6, 8)
rint = gen.randrange(0, 2, 2)

assert rint == 0
print("SystemRandom_TestBasicOps::test_randrange_nonunit_step: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_randrange_nonunit_step: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_sample_on_dicts.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_sample_on_dicts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_sample_on_dicts"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_sample_on_dicts"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_sample_on_dicts
"""Auto-ported test: SystemRandom_TestBasicOps::test_sample_on_dicts (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()

try:
    gen.sample(dict.fromkeys('abcdef'), 2)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("SystemRandom_TestBasicOps::test_sample_on_dicts: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_sample_on_dicts: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_seed_no_mutate_bug_44018.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_seed_no_mutate_bug_44018() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_seed_no_mutate_bug_44018"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_seed_no_mutate_bug_44018"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_seed_no_mutate_bug_44018
"""Auto-ported test: SystemRandom_TestBasicOps::test_seed_no_mutate_bug_44018 (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
a = bytearray(b'1234')
gen.seed(a)

assert a == bytearray(b'1234')
print("SystemRandom_TestBasicOps::test_seed_no_mutate_bug_44018: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_seed_no_mutate_bug_44018: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/system_random_test_basic_ops__test_seedargs.py`.
#[test]
fn test_gen_behavior_std_libs_random_system_random_test_basic_ops__test_seedargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "system_random_test_basic_ops__test_seedargs"
# subject = "cpython.test_random.SystemRandom_TestBasicOps.test_seedargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::SystemRandom_TestBasicOps::test_seedargs
"""Auto-ported test: SystemRandom_TestBasicOps::test_seedargs (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
gen = random.SystemRandom()
gen.seed(100)
print("SystemRandom_TestBasicOps::test_seedargs: ok")
"###);
    assert_output(&out, r###"SystemRandom_TestBasicOps::test_seedargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/test_distributions__test_von_mises_large_kappa.py`.
#[test]
fn test_gen_behavior_std_libs_random_test_distributions__test_von_mises_large_kappa() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_von_mises_large_kappa"
# subject = "cpython.test_random.TestDistributions.test_von_mises_large_kappa"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_von_mises_large_kappa
"""Auto-ported test: TestDistributions::test_von_mises_large_kappa (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
random.vonmisesvariate(0, 1000000000000000.0)
random.vonmisesvariate(0, 1e+100)
print("TestDistributions::test_von_mises_large_kappa: ok")
"###);
    assert_output(&out, r###"TestDistributions::test_von_mises_large_kappa: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/test_distributions__test_von_mises_range.py`.
#[test]
fn test_gen_behavior_std_libs_random_test_distributions__test_von_mises_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_von_mises_range"
# subject = "cpython.test_random.TestDistributions.test_von_mises_range"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_von_mises_range
"""Auto-ported test: TestDistributions::test_von_mises_range (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
g = random.Random()
N = 100
for mu in (0.0, 0.1, 3.1, 6.2):
    for kappa in (0.0, 2.3, 500.0):
        for _ in range(N):
            sample = g.vonmisesvariate(mu, kappa)

            assert 0 <= sample <= random.TWOPI
print("TestDistributions::test_von_mises_range: ok")
"###);
    assert_output(&out, r###"TestDistributions::test_von_mises_range: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/test_distributions__test_zeroinputs.py`.
#[test]
fn test_gen_behavior_std_libs_random_test_distributions__test_zeroinputs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_distributions__test_zeroinputs"
# subject = "cpython.test_random.TestDistributions.test_zeroinputs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::TestDistributions::test_zeroinputs
"""Auto-ported test: TestDistributions::test_zeroinputs (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---
g = random.Random()
x = [g.random() for i in range(50)] + [0.0] * 5
g.random = x[:].pop
g.uniform(1, 10)
g.random = x[:].pop
g.paretovariate(1.0)
g.random = x[:].pop
g.expovariate(1.0)
g.random = x[:].pop
g.expovariate()
g.random = x[:].pop
g.weibullvariate(1.0, 1.0)
g.random = x[:].pop
g.vonmisesvariate(1.0, 1.0)
g.random = x[:].pop
g.normalvariate(0.0, 1.0)
g.random = x[:].pop
g.gauss(0.0, 1.0)
g.random = x[:].pop
g.lognormvariate(0.0, 1.0)
g.random = x[:].pop
g.vonmisesvariate(0.0, 1.0)
g.random = x[:].pop
g.gammavariate(0.01, 1.0)
g.random = x[:].pop
g.gammavariate(1.0, 1.0)
g.random = x[:].pop
g.gammavariate(200.0, 1.0)
g.random = x[:].pop
g.betavariate(3.0, 3.0)
g.random = x[:].pop
g.triangular(0.0, 1.0, 1.0 / 3.0)
print("TestDistributions::test_zeroinputs: ok")
"###);
    assert_output(&out, r###"TestDistributions::test_zeroinputs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/test_module__test_magic_constants.py`.
#[test]
fn test_gen_behavior_std_libs_random_test_module__test_magic_constants() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "test_module__test_magic_constants"
# subject = "cpython.test_random.TestModule.testMagicConstants"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_random.py::TestModule::testMagicConstants
"""Auto-ported test: TestModule::testMagicConstants (CPython 3.12 oracle)."""


import unittest
import unittest.mock
import random
import os
import time
import pickle
import warnings
import test.support
from functools import partial
from math import log, exp, pi, fsum, sin, factorial
from test import support
from fractions import Fraction
from collections import abc, Counter


try:
    random.SystemRandom().random()
except NotImplementedError:
    SystemRandom_available = False
else:
    SystemRandom_available = True

def gamma(z, sqrt2pi=(2.0 * pi) ** 0.5):
    if z < 0.5:
        return pi / sin(pi * z) / gamma(1.0 - z)
    az = z + (7.0 - 0.5)
    return az ** (z - 0.5) / exp(az) * sqrt2pi * fsum([0.9999999999995183, 676.5203681218835 / z, -1259.139216722289 / (z + 1.0), 771.3234287757674 / (z + 2.0), -176.6150291498386 / (z + 3.0), 12.50734324009056 / (z + 4.0), -0.1385710331296526 / (z + 5.0), 9.934937113930748e-06 / (z + 6.0), 1.659470187408462e-07 / (z + 7.0)])


# --- test body ---

assert abs(random.NV_MAGICCONST - 1.71552776992141) < 1e-07

assert abs(random.TWOPI - 6.28318530718) < 1e-07

assert abs(random.LOG4 - 1.38629436111989) < 1e-07

assert abs(random.SG_MAGICCONST - 2.50407739677627) < 1e-07
print("TestModule::testMagicConstants: ok")
"###);
    assert_output(&out, r###"TestModule::testMagicConstants: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/uniform_stays_within_bounds.py`.
#[test]
fn test_gen_behavior_std_libs_random_uniform_stays_within_bounds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "uniform_stays_within_bounds"
# subject = "random.uniform"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.uniform: uniform(2.0, 5.0) returns floats within the closed bound [2.0, 5.0] across 100 seeded draws"""
import random

random.seed(6)
us = [random.uniform(2.0, 5.0) for _ in range(100)]
assert all(2.0 <= u <= 5.0 for u in us), "uniform in [2,5]"

print("uniform_stays_within_bounds OK")
"###);
    assert_output(&out, r###"uniform_stays_within_bounds OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/random/vonmisesvariate_stays_in_circle.py`.
#[test]
fn test_gen_behavior_std_libs_random_vonmisesvariate_stays_in_circle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "vonmisesvariate_stays_in_circle"
# subject = "random.Random.vonmisesvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.vonmisesvariate: vonmisesvariate(mu, kappa) stays within [0, 2*pi] across a grid of mu/kappa combinations including kappa=0"""
import random

gen = random.Random(0)
for mu in (0.0, 0.1, 3.1, 6.2):
    for kappa in (0.0, 2.3, 500.0):
        for _ in range(20):
            v = gen.vonmisesvariate(mu, kappa)
            assert 0.0 <= v <= random.TWOPI, f"vonmises out of range: {v!r}"

print("vonmisesvariate_stays_in_circle OK")
"###);
    assert_output(&out, r###"vonmisesvariate_stays_in_circle OK
"###);
}
