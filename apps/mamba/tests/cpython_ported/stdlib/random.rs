//! Ported from Lib/test/test_random_ported.py
//! Integration tests: stdlib/random.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_random_ported.py`.
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

/// Ported from `Lib/test/test_random_ported.py`.
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

/// Ported from `Lib/test/test_random_ported.py`.
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

/// Ported from `Lib/test/test_random_ported.py`.
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

/// Ported from `Lib/test/test_random_ported.py`.
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

