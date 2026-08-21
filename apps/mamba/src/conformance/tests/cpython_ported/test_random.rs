//! Py3.12 conformance tests for the `random` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_random.py):
//!   functional surface only — random(), randint(), choice(), sample(),
//!   shuffle(). Exact values are not asserted because mamba's PRNG
//!   sequence differs from CPython's Mersenne Twister; instead, range
//!   and cardinality invariants are checked.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_random_random_in_unit_interval() {
    let out = jit_capture(
        r#"import random
random.seed(42)
v = random.random()
print(0 <= v < 1)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_random_randint_in_range() {
    let out = jit_capture(
        r#"import random
random.seed(42)
v = random.randint(0, 10)
print(0 <= v <= 10)
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_random_choice_from_population() {
    let out = jit_capture(
        r#"import random
random.seed(7)
v = random.choice([1, 2, 3, 4, 5])
print(v in [1, 2, 3, 4, 5])
"#,
    );
    assert_output(&out, "True\n");
}

#[test]
fn test_random_sample_distinct_cardinality() {
    let out = jit_capture(
        r#"import random
random.seed(7)
xs = random.sample([1, 2, 3, 4, 5], 3)
print(len(xs))
print(len(set(xs)))
"#,
    );
    assert_output(&out, "3\n3\n");
}

#[test]
fn test_random_shuffle_preserves_multiset() {
    let out = jit_capture(
        r#"import random
random.seed(7)
xs = [1, 2, 3, 4, 5]
random.shuffle(xs)
print(sorted(xs))
print(len(xs))
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5]\n5\n");
}
