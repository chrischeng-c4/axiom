use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/generator_float_inference/frange_for_loop_accumulates_floats.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_frange_for_loop_accumulates_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_for_loop_accumulates_floats"
# subject = "float-param generator accumulated across yield, consumed by for loop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator consumed by a for loop: each yielded value is the correct accumulated float."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


collected = []
for value in frange(1.0, 3.0, 0.5):
    collected.append(value)
assert collected == [1.0, 1.5, 2.0, 2.5, 3.0], collected
assert all(isinstance(x, float) for x in collected), collected
print("frange_for_loop_accumulates_floats OK")
"###);
    assert_output(&out, r###"frange_for_loop_accumulates_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/frange_list_accumulates_floats.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_frange_list_accumulates_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_list_accumulates_floats"
# subject = "float-param generator accumulated across yield, consumed by list()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator: FLOAT params accumulated across yield must yield correct floats into list()."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


result = list(frange(0.0, 1.0, 0.25))
assert result == [0.0, 0.25, 0.5, 0.75, 1.0], result
assert all(isinstance(x, float) for x in result), result
print("frange_list_accumulates_floats OK")
"###);
    assert_output(&out, r###"frange_list_accumulates_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/frange_step_decimal_drift.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_frange_step_decimal_drift() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_step_decimal_drift"
# subject = "float-param generator with binary-inexact step reproduces CPython drift exactly"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A frange-style float generator with a 0.1 step must reproduce CPython's binary float drift exactly."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


# 0.1 is not exactly representable in binary float; after three additions the
# accumulator is 0.30000000000000004, which is > 0.3, so the loop stops before
# yielding a fourth value. The pinned list captures CPython's exact drift.
result = list(frange(0.0, 0.3, 0.1))
assert result == [0.0, 0.1, 0.2], result
assert all(isinstance(x, float) for x in result), result
print("frange_step_decimal_drift OK")
"###);
    assert_output(&out, r###"frange_step_decimal_drift OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/frange_sum_accumulates_floats.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_frange_sum_accumulates_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_sum_accumulates_floats"
# subject = "float-param generator accumulated across yield, reduced by sum()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator reduced by sum(): the total of the yielded floats is the correct float."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


total = sum(frange(0.0, 2.0, 0.5))
assert isinstance(total, float), type(total)
assert total == 5.0, total
print("frange_sum_accumulates_floats OK")
"###);
    assert_output(&out, r###"frange_sum_accumulates_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/generator_float_local_accumulator.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_generator_float_local_accumulator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "generator_float_local_accumulator"
# subject = "generator with a float local accumulator carried across yield"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator carrying a float local accumulator across yields must yield correct running totals."""


def running_total(values):
    acc = 0.0
    for v in values:
        acc += v
        yield acc


result = list(running_total([0.5, 1.5, 2.0]))
assert result == [0.5, 2.0, 4.0], result
assert all(isinstance(x, float) for x in result), result
print("generator_float_local_accumulator OK")
"###);
    assert_output(&out, r###"generator_float_local_accumulator OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/genexpr_float_accumulator_mean.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_genexpr_float_accumulator_mean() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_float_accumulator_mean"
# subject = "mean computed from sum() over a float generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A mean computed by summing a float generator expression then dividing must give the correct float."""

samples = [1.0, 2.0, 3.0, 4.0]
total = sum(x for x in samples)
mean = total / len(samples)
assert isinstance(total, float), type(total)
assert isinstance(mean, float), type(mean)
assert total == 10.0, total
assert mean == 2.5, mean
print("genexpr_float_accumulator_mean OK")
"###);
    assert_output(&out, r###"genexpr_float_accumulator_mean OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/genexpr_sum_floats.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_genexpr_sum_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_sum_floats"
# subject = "generator expression of floats reduced by sum()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator expression producing floats, summed, must yield the correct float total."""

data = [1, 2, 3, 4]
total = sum(x * 1.5 for x in data)
assert isinstance(total, float), type(total)
assert total == 15.0, total
print("genexpr_sum_floats OK")
"###);
    assert_output(&out, r###"genexpr_sum_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/genexpr_sum_with_float_start.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_genexpr_sum_with_float_start() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_sum_with_float_start"
# subject = "sum() of an int genexpr with a float start value returns a float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sum() over a generator expression with a float start value must return the correct float total."""

total = sum((i for i in range(5)), 0.5)
assert isinstance(total, float), type(total)
assert total == 10.5, total
print("genexpr_sum_with_float_start OK")
"###);
    assert_output(&out, r###"genexpr_sum_with_float_start OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/genexpr_zip_floats.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_genexpr_zip_floats() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "genexpr_zip_floats"
# subject = "generator expression of floats feeding zip()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float generator expression feeding zip() must pair the correct float values."""

floats = (i + 0.5 for i in range(3))
labels = ["a", "b", "c"]
paired = list(zip(labels, floats))
assert paired == [("a", 0.5), ("b", 1.5), ("c", 2.5)], paired
assert all(isinstance(value, float) for _, value in paired), paired
print("genexpr_zip_floats OK")
"###);
    assert_output(&out, r###"genexpr_zip_floats OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/next_on_float_genexpr.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_next_on_float_genexpr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "next_on_float_genexpr"
# subject = "next() drawing successive floats from a generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""next() pulled from a float generator expression must return the correct float at each step."""

gen = (i * 0.25 for i in range(4))
first = next(gen)
second = next(gen)
assert isinstance(first, float) and isinstance(second, float), (first, second)
assert first == 0.0, first
assert second == 0.25, second
assert next(gen) == 0.5
assert next(gen) == 0.75
print("next_on_float_genexpr OK")
"###);
    assert_output(&out, r###"next_on_float_genexpr OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/next_on_frange_yields_float.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_next_on_frange_yields_float() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "next_on_frange_yields_float"
# subject = "next() on a float-param generator returns the correct accumulated float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""next() on a frange-style float-param generator must return the correct accumulated float each call."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


gen = frange(0.0, 10.0, 2.5)
assert next(gen) == 0.0
assert next(gen) == 2.5
third = next(gen)
assert isinstance(third, float), type(third)
assert third == 5.0, third
assert next(gen) == 7.5
print("next_on_frange_yields_float OK")
"###);
    assert_output(&out, r###"next_on_frange_yields_float OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/yield_computed_float_each_step.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_yield_computed_float_each_step() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_computed_float_each_step"
# subject = "generator yielding a freshly computed float each step"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator that computes a float each step (i / 2.0) must yield the correct float values."""


def halves(n):
    for i in range(n):
        yield i / 2.0


result = list(halves(5))
assert result == [0.0, 0.5, 1.0, 1.5, 2.0], result
assert all(isinstance(x, float) for x in result), result
print("yield_computed_float_each_step OK")
"###);
    assert_output(&out, r###"yield_computed_float_each_step OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/yield_float_division_value.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_yield_float_division_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_float_division_value"
# subject = "float division return value yielded by a generator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator yielding true-division (/) results must produce correct float quotients, not NaN-box bits."""


def reciprocals(values):
    for v in values:
        yield 1 / v


result = list(reciprocals([2, 4, 8]))
assert result == [0.5, 0.25, 0.125], result
assert all(isinstance(x, float) for x in result), result
print("yield_float_division_value OK")
"###);
    assert_output(&out, r###"yield_float_division_value OK
"###);
}

/// Ported from `tests/cpython/behavior/core/generator_float_inference/yield_float_param_unmodified.py`.
#[test]
fn test_gen_behavior_core_generator_float_inference_yield_float_param_unmodified() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_float_param_unmodified"
# subject = "generator yielding a float parameter unchanged"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator that yields a float parameter unmodified must produce that exact float, not NaN-box bits."""


def repeat_float(value, times):
    for _ in range(times):
        yield value


result = list(repeat_float(3.14, 3))
assert result == [3.14, 3.14, 3.14], result
assert all(isinstance(x, float) for x in result), result
print("yield_float_param_unmodified OK")
"###);
    assert_output(&out, r###"yield_float_param_unmodified OK
"###);
}
