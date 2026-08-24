use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_chained_reciprocal.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_chained_reciprocal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_chained_reciprocal"
# subject = "chained float division return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning 1.0/(j+1) must yield the correct reciprocal float."""


def reciprocal(j):
    return 1.0 / (j + 1)


r = reciprocal(3)
assert r == 0.25, r
assert isinstance(r, float), type(r)
assert reciprocal(0) == 1.0, reciprocal(0)
assert reciprocal(1) == 0.5, reciprocal(1)
print("return_chained_reciprocal OK")
"###);
    assert_output(&out, r###"return_chained_reciprocal OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_conditional_float.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_conditional_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_conditional_float"
# subject = "conditional-branch float return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function whose both branches return a computed float must yield correct floats."""


def pick(x):
    if x > 0:
        return x / 2
    else:
        return x * 1.5


pos = pick(9)
neg = pick(-4)
assert pos == 4.5, pos
assert neg == -6.0, neg
assert isinstance(pos, float), type(pos)
assert isinstance(neg, float), type(neg)
print("return_conditional_float OK")
"###);
    assert_output(&out, r###"return_conditional_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_abs_expr.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_abs_expr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_abs_expr"
# subject = "float absolute-value return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning abs() of a computed float must yield the correct float."""


def magnitude(a, b):
    return abs(a - b)


r = magnitude(2.0, 5.5)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r * 2.0 == 7.0, r
print("return_float_abs_expr OK")
"###);
    assert_output(&out, r###"return_float_abs_expr OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_add.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_add() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_add"
# subject = "float add return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a+b on floats must yield the correct float sum."""


def add(a, b):
    return a + b


r = add(1.25, 2.5)
assert r == 3.75, r
assert isinstance(r, float), type(r)
assert r - 0.75 == 3.0, r
print("return_float_add OK")
"###);
    assert_output(&out, r###"return_float_add OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_modulo.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_modulo() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_modulo"
# subject = "float modulo return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a%b on floats must yield the correct float remainder."""


def remainder(a, b):
    return a % b


r = remainder(5.5, 2.0)
assert r == 1.5, r
assert isinstance(r, float), type(r)
assert r + 4.0 == 5.5, r
print("return_float_modulo OK")
"###);
    assert_output(&out, r###"return_float_modulo OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_multiply.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_multiply() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_multiply"
# subject = "float multiply return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a*b on floats must yield the correct float product."""


def scale(a, b):
    return a * b


r = scale(2.5, 4.0)
assert r == 10.0, r
assert isinstance(r, float), type(r)
assert r + 0.5 == 10.5, r
print("return_float_multiply OK")
"###);
    assert_output(&out, r###"return_float_multiply OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_negate.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_negate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_negate"
# subject = "float unary negation return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning -a on a float must yield the correct negated float."""


def negate(a):
    return -a


r = negate(3.5)
assert r == -3.5, r
assert isinstance(r, float), type(r)
assert r + 3.5 == 0.0, r
print("return_float_negate OK")
"###);
    assert_output(&out, r###"return_float_negate OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_param.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_param() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_param"
# subject = "float parameter returned unchanged"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a float parameter unchanged must yield that exact float."""


def identity(x):
    return x


r = identity(2.75)
assert r == 2.75, r
assert isinstance(r, float), type(r)
assert r * 4.0 == 11.0, r
print("return_float_param OK")
"###);
    assert_output(&out, r###"return_float_param OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_power.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_power() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_power"
# subject = "float power return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a**b producing a float must yield the correct float."""


def power(a, b):
    return a ** b


r = power(2.0, 3.0)
assert r == 8.0, r
assert isinstance(r, float), type(r)
assert r / 2.0 == 4.0, r
print("return_float_power OK")
"###);
    assert_output(&out, r###"return_float_power OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_float_subtract.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_float_subtract() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_float_subtract"
# subject = "float subtract return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a-b on floats must yield the correct float difference."""


def sub(a, b):
    return a - b


r = sub(5.5, 2.0)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r + 2.0 == 5.5, r
print("return_float_subtract OK")
"###);
    assert_output(&out, r###"return_float_subtract OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_int_int_division_is_float.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_int_int_division_is_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_int_int_division_is_float"
# subject = "int/int true division returns an exact-valued float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""int/int true division that is mathematically whole must still return a float, not an int."""


def divide(a, b):
    return a / b


r = divide(6, 3)
assert r == 2.0, r
assert isinstance(r, float), type(r)
assert r != 6, r
assert r / 2 == 1.0, r
print("return_int_int_division_is_float OK")
"###);
    assert_output(&out, r###"return_int_int_division_is_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_local_float_var.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_local_float_var() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_local_float_var"
# subject = "float returned via a local variable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function computing a float into a local var then returning it must yield it correctly."""


def compute(a, b):
    total = a / b
    total = total + 1.0
    return total


r = compute(6, 4)
assert r == 2.5, r
assert isinstance(r, float), type(r)
assert r - 1.0 == 1.5, r
print("return_local_float_var OK")
"###);
    assert_output(&out, r###"return_local_float_var OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_math_sin.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_math_sin() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_math_sin"
# subject = "function returning math.sin result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning math.sin(x) must yield the correct float."""
import math


def sine(x):
    return math.sin(x)


r = sine(0.0)
assert r == 0.0, r
assert isinstance(r, float), type(r)
assert sine(math.pi / 2) == 1.0, sine(math.pi / 2)
print("return_math_sin OK")
"###);
    assert_output(&out, r###"return_math_sin OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_math_sqrt.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_math_sqrt() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_math_sqrt"
# subject = "function returning math.sqrt result"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning math.sqrt(x) must yield the correct float root."""
import math


def root(x):
    return math.sqrt(x)


r = root(16.0)
assert r == 4.0, r
assert isinstance(r, float), type(r)
assert root(2.0) == math.sqrt(2.0), root(2.0)
print("return_math_sqrt OK")
"###);
    assert_output(&out, r###"return_math_sqrt OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_method_computed_float.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_method_computed_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_method_computed_float"
# subject = "method returning a computed float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A method returning a computed float must yield the correct float value."""


class Box:
    def __init__(self, value):
        self.value = value

    def half(self):
        return self.value / 2


b = Box(9)
r = b.half()
assert r == 4.5, r
assert isinstance(r, float), type(r)
assert r * 2 == 9.0, r
print("return_method_computed_float OK")
"###);
    assert_output(&out, r###"return_method_computed_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_nested_call_float.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_nested_call_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_nested_call_float"
# subject = "float returned from a nested call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""An outer function returning the result of an inner float-returning call must be correct."""


def inner(a, b):
    return a / b


def outer(a, b):
    return inner(a, b) + 0.5


r = outer(7, 2)
assert r == 4.0, r
assert isinstance(r, float), type(r)
assert inner(1, 4) == 0.25, inner(1, 4)
print("return_nested_call_float OK")
"###);
    assert_output(&out, r###"return_nested_call_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_recursive_float.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_recursive_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_recursive_float"
# subject = "recursion accumulating a float return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A recursive function summing reciprocals must return the correct accumulated float."""


def harmonic(n):
    if n <= 0:
        return 0.0
    return 1.0 / n + harmonic(n - 1)


r = harmonic(4)
assert r == 1.0 + 0.5 + (1.0 / 3) + 0.25, r
assert isinstance(r, float), type(r)
assert harmonic(1) == 1.0, harmonic(1)
print("return_recursive_float OK")
"###);
    assert_output(&out, r###"return_recursive_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/float_return_inference/return_true_division.py`.
#[test]
fn test_gen_behavior_core_float_return_inference_return_true_division() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "float_return_inference"
# dimension = "behavior"
# case = "return_true_division"
# subject = "float division return value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A function returning a/b (true division of ints) must yield the correct float."""


def divide(a, b):
    return a / b


r = divide(7, 2)
assert r == 3.5, r
assert isinstance(r, float), type(r)
assert r * 2 == 7.0, r
print("return_true_division OK")
"###);
    assert_output(&out, r###"return_true_division OK
"###);
}
