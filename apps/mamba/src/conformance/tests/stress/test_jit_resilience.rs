//! JIT resilience & stress tests (#gen12_fuzzing).
//!
//! Probes JIT stack depth, recursion, allocation/deallocation stress,
//! loop throughput, and memory stability under execution.

use super::{jit_assert_output, jit_try};

/// Test JIT execution of recursive functions at moderate stack depths.
#[test]
fn test_jit_recursion_limit_and_stack_depth() {
    let src = r#"
def rec(n: int):
    if n > 0:
        return rec(n - 1) + 1
    return 0

print(rec(50))
"#;
    jit_assert_output(src, "50");

    let src_tail = r#"
def acc_rec(n: int, acc: int):
    if n <= 0:
        return acc
    return acc_rec(n - 1, acc + 2)

print(acc_rec(80, 0))
"#;
    jit_assert_output(src_tail, "160");
}

/// Test JIT allocation stress: 10,000 list creations and garbage collection state.
#[test]
fn test_jit_allocation_stress() {
    let src_list = r#"
def stress_lists():
    total = 0
    for i in range(10000):
        x = [i, i + 1, i + 2]
        total += len(x)
    print(total)

stress_lists()
"#;
    jit_assert_output(src_list, "30000");

    let src_dict = r#"
def stress_dicts():
    d = {}
    for i in range(5000):
        d[i] = i * 2
    print(len(d), d[4999])

stress_dicts()
"#;
    jit_assert_output(src_dict, "5000 9998");
}

/// Test JIT high-iteration loop stress (50,000 iterations).
#[test]
fn test_jit_large_loop_stress() {
    let src = r#"
def loop_stress():
    acc = 0
    for i in range(50000):
        acc = (acc + i) % 1000000
    print(acc)

loop_stress()
"#;
    jit_assert_output(src, "975000");
}

/// Test JIT minimized range-loop witness exposing mid-loop checkpoint, final loop var, and final acc.
#[test]
fn test_jit_minimized_range_loop_witness() {
    let src = r#"
def minimized_loop():
    acc = 0
    checkpoint = -1
    for i in range(5):
        acc = (acc + i) % 7
        if i == 2:
            checkpoint = acc
    print(checkpoint, i, acc)

minimized_loop()
"#;
    jit_assert_output(src, "3 4 3");
}

/// Test JIT 4-level nested loops.
#[test]
fn test_jit_multi_nested_loops() {
    let src = r#"
def nested():
    count = 0
    for i in range(10):
        for j in range(10):
            for k in range(10):
                for l in range(10):
                    count += 1
    print(count)

nested()
"#;
    jit_assert_output(src, "10000");
}

/// Test JIT string construction and concatenation stress.
#[test]
fn test_jit_string_concat_stress() {
    let src = r#"
def str_stress():
    s = ""
    for i in range(1000):
        s += "a"
    print(len(s))

str_stress()
"#;
    jit_assert_output(src, "1000");
}
