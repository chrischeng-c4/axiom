use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/random/choice_empty_raises_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choice_empty_raises_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choice_empty_raises_indexerror"
# subject = "random.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choice: choice_empty_raises_indexerror (errors)."""
import random

_raised = False
try:
    random.choice([])
except IndexError:
    _raised = True
assert _raised, "choice_empty_raises_indexerror: expected IndexError"
print("choice_empty_raises_indexerror OK")
"###);
    assert_output(&out, r###"choice_empty_raises_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_all_zero_weights_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_all_zero_weights_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_all_zero_weights_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_all_zero_weights_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices('AB', weights=[0.0, 0.0])
except ValueError:
    _raised = True
assert _raised, "choices_all_zero_weights_raises_valueerror: expected ValueError"
print("choices_all_zero_weights_raises_valueerror OK")
"###);
    assert_output(&out, r###"choices_all_zero_weights_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_both_weight_kinds_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_both_weight_kinds_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_both_weight_kinds_raises_typeerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_both_weight_kinds_raises_typeerror (errors)."""
import random

_raised = False
try:
    random.choices([1, 2], weights=[1, 1], cum_weights=[1, 2])
except TypeError:
    _raised = True
assert _raised, "choices_both_weight_kinds_raises_typeerror: expected TypeError"
print("choices_both_weight_kinds_raises_typeerror OK")
"###);
    assert_output(&out, r###"choices_both_weight_kinds_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_cum_weights_wrong_length_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_cum_weights_wrong_length_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_cum_weights_wrong_length_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_cum_weights_wrong_length_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices([1, 2, 3], cum_weights=[1, 2])
except ValueError:
    _raised = True
assert _raised, "choices_cum_weights_wrong_length_raises_valueerror: expected ValueError"
print("choices_cum_weights_wrong_length_raises_valueerror OK")
"###);
    assert_output(&out, r###"choices_cum_weights_wrong_length_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_empty_population_raises_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_empty_population_raises_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_empty_population_raises_indexerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_empty_population_raises_indexerror (errors)."""
import random

_raised = False
try:
    random.choices([], k=3)
except IndexError:
    _raised = True
assert _raised, "choices_empty_population_raises_indexerror: expected IndexError"
print("choices_empty_population_raises_indexerror OK")
"###);
    assert_output(&out, r###"choices_empty_population_raises_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_negative_total_weight_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_negative_total_weight_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_negative_total_weight_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_negative_total_weight_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices('ABC', weights=[3, -5, 1])
except ValueError:
    _raised = True
assert _raised, "choices_negative_total_weight_raises_valueerror: expected ValueError"
print("choices_negative_total_weight_raises_valueerror OK")
"###);
    assert_output(&out, r###"choices_negative_total_weight_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/choices_weights_wrong_length_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_choices_weights_wrong_length_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "choices_weights_wrong_length_raises_valueerror"
# subject = "random.choices"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.choices: choices_weights_wrong_length_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.choices([1, 2, 3], weights=[1.0])
except ValueError:
    _raised = True
assert _raised, "choices_weights_wrong_length_raises_valueerror: expected ValueError"
print("choices_weights_wrong_length_raises_valueerror OK")
"###);
    assert_output(&out, r###"choices_weights_wrong_length_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/randint_inverted_range_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_randint_inverted_range_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "randint_inverted_range_raises_valueerror"
# subject = "random.randint"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randint: randint_inverted_range_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.randint(10, 5)
except ValueError:
    _raised = True
assert _raised, "randint_inverted_range_raises_valueerror: expected ValueError"
print("randint_inverted_range_raises_valueerror OK")
"###);
    assert_output(&out, r###"randint_inverted_range_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/randrange_empty_range_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_randrange_empty_range_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "randrange_empty_range_raises_valueerror"
# subject = "random.randrange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.randrange: randrange_empty_range_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.randrange(5, 5)
except ValueError:
    _raised = True
assert _raised, "randrange_empty_range_raises_valueerror: expected ValueError"
print("randrange_empty_range_raises_valueerror OK")
"###);
    assert_output(&out, r###"randrange_empty_range_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/sample_k_too_large_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_sample_k_too_large_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_k_too_large_raises_valueerror"
# subject = "random.sample"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.sample: sample_k_too_large_raises_valueerror (errors)."""
import random

_raised = False
try:
    random.sample([1, 2], 5)
except ValueError:
    _raised = True
assert _raised, "sample_k_too_large_raises_valueerror: expected ValueError"
print("sample_k_too_large_raises_valueerror OK")
"###);
    assert_output(&out, r###"sample_k_too_large_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/sample_negative_counts_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_random_sample_negative_counts_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "sample_negative_counts_raises_valueerror"
# subject = "random.Random.sample"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.sample: negative counts raise ValueError, and k larger than the expanded population also raises ValueError"""
import random

gen = random.Random(0)

# Negative counts raise ValueError.
try:
    gen.sample(["red", "green", "blue"], counts=[-3, -7, -8], k=2)
    raise AssertionError("expected ValueError for negative counts")
except ValueError:
    pass

# k larger than the expanded population raises ValueError.
try:
    gen.sample(["red", "green"], counts=[10, 10], k=21)
    raise AssertionError("expected ValueError for k > total")
except ValueError:
    pass

print("sample_negative_counts_raises_valueerror OK")
"###);
    assert_output(&out, r###"sample_negative_counts_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/shuffle_immutable_sequence_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_random_shuffle_immutable_sequence_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "shuffle_immutable_sequence_raises_typeerror"
# subject = "random.shuffle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.shuffle: shuffle_immutable_sequence_raises_typeerror (errors)."""
import random

_raised = False
try:
    random.shuffle('string is immutable')
except TypeError:
    _raised = True
assert _raised, "shuffle_immutable_sequence_raises_typeerror: expected TypeError"
print("shuffle_immutable_sequence_raises_typeerror OK")
"###);
    assert_output(&out, r###"shuffle_immutable_sequence_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/system_random_getrandbits_validates_arg.py`.
#[test]
fn test_gen_errors_std_libs_random_system_random_getrandbits_validates_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "system_random_getrandbits_validates_arg"
# subject = "random.SystemRandom.getrandbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.getrandbits: SystemRandom.getrandbits validates its argument: getrandbits(-1) raises ValueError and getrandbits(10.1) raises TypeError"""
import random

gen = random.SystemRandom()
for bad in [(-1, ValueError), (10.1, TypeError)]:
    arg, exc = bad
    try:
        gen.getrandbits(arg)
        raise AssertionError(f"expected {exc.__name__} for getrandbits({arg!r})")
    except exc:
        pass

print("system_random_getrandbits_validates_arg OK")
"###);
    assert_output(&out, r###"system_random_getrandbits_validates_arg OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/random/system_random_randbytes_rejects_negative.py`.
#[test]
fn test_gen_errors_std_libs_random_system_random_randbytes_rejects_negative() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "system_random_randbytes_rejects_negative"
# subject = "random.SystemRandom.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.randbytes: SystemRandom.randbytes rejects a negative count with ValueError, and choice([]) on an empty sequence raises IndexError"""
import random

gen = random.SystemRandom()

# randbytes rejects a negative count.
try:
    gen.randbytes(-1)
    raise AssertionError("expected ValueError for randbytes(-1)")
except ValueError:
    pass

# choice on an empty sequence raises IndexError.
try:
    gen.choice([])
    raise AssertionError("expected IndexError for empty choice")
except IndexError:
    pass

print("system_random_randbytes_rejects_negative OK")
"###);
    assert_output(&out, r###"system_random_randbytes_rejects_negative OK
"###);
}
