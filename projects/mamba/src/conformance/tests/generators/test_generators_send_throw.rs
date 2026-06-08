//! send() / throw() round-trip conformance (AC2, AC3 — R2, R3).
//!
//! Ported from CPython 3.12 Lib/test/test_generators.py @ tag v3.12.0
//! (ExceptionTest, GenSendTest subsets).
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC2

use super::{assert_output, jit_capture};

/// AC2: send() happy path — value becomes the result of the suspended yield.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC2
#[test]
fn test_send_happy_path_echo() {
    let output = jit_capture(
        r#"def echo():
    while True:
        received = yield
        if received is None:
            return
        print(received)

g = echo()
next(g)             # prime to first yield
try:
    g.send('hello')
    g.send('world')
    g.send(None)    # triggers return -> StopIteration
except StopIteration:
    print('stopped')
"#,
    );
    assert_output(&output, "hello\nworld\nstopped\n");
}

/// AC2: send(non-None) on a just-started generator raises TypeError whose
/// message substring matches CPython ("can't send non-None value to a
/// just-started generator"). Ensures user-visible error text matches CPython.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC2
#[test]
fn test_send_nonnone_to_just_started_raises_typeerror() {
    // Print the exception message and assert (Rust-side) that the CPython
    // substring is present. This avoids depending on the Python `in`
    // operator over strings inside the JIT which is not always literal.
    let output = jit_capture(
        r#"def g():
    yield 1

gen = g()
try:
    gen.send('x')
except TypeError as e:
    print('typeerror:', str(e))
"#,
    );
    assert!(
        output.contains("can't send non-None value to a just-started generator"),
        "expected CPython TypeError message substring, got: {output}"
    );
}

/// AC2: send(None) is always allowed on a just-started generator — it primes
/// to the first yield just like next().
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC2
#[test]
fn test_send_none_primes_generator() {
    let output = jit_capture(
        r#"def g():
    val = yield 'primed'
    yield val + '!'

gen = g()
print(gen.send(None))
print(gen.send('ok'))
"#,
    );
    assert_output(&output, "primed\nok!\n");
}

/// AC3: throw() — exception caught inside generator, execution resumes.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC3
#[test]
fn test_throw_caught_resumes_execution() {
    let output = jit_capture(
        r#"def catch():
    try:
        yield 1
    except ValueError:
        yield 'caught'

g = catch()
print(next(g))
print(g.throw(ValueError('x')))
"#,
    );
    assert_output(&output, "1\ncaught\n");
}

/// AC3: throw() — uncaught exception propagates out of throw() to the caller.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC3
#[test]
fn test_throw_uncaught_propagates() {
    let output = jit_capture(
        r#"def propagate():
    yield 1

g = propagate()
print(next(g))
try:
    g.throw(ValueError('injected'))
except ValueError as e:
    print('propagated:', e)
"#,
    );
    assert_output(&output, "1\npropagated: injected\n");
}

/// AC3: throw() into a finally block runs the finally clause before the
/// exception continues propagating.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC3
#[test]
fn test_throw_into_finally_runs_cleanup() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    finally:
        print('cleanup')

gen = g()
next(gen)
try:
    gen.throw(RuntimeError('boom'))
except RuntimeError as e:
    print('propagated:', e)
"#,
    );
    assert_output(&output, "cleanup\npropagated: boom\n");
}
