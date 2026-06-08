//! close() / GeneratorExit conformance (AC4 — R4).
//!
//! Ported from CPython 3.12 Lib/test/test_generators.py @ tag v3.12.0
//! (CloseTest / ExceptionTest cleanup paths).
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4

use super::{assert_output, jit_capture};

/// AC4: close() on an active generator returns None and advances the body
/// to the Completed state. A subsequent next() raises StopIteration.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4
#[test]
fn test_close_happy_path_returns_none_then_stopiteration() {
    let output = jit_capture(
        r#"def g():
    yield 1
    yield 2

gen = g()
print(next(gen))
result = gen.close()
print('close returned:', result)
try:
    next(gen)
except StopIteration:
    print('stopped after close')
"#,
    );
    assert_output(
        &output,
        "1\nclose returned: None\nstopped after close\n",
    );
}

/// AC4: GeneratorExit propagates to an `except GeneratorExit` handler inside
/// the generator body, allowing user cleanup code to run.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4
#[test]
fn test_close_triggers_generatorexit_handler() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
    except GeneratorExit:
        print('GeneratorExit caught')
        raise   # must re-raise to satisfy the protocol

gen = g()
next(gen)
gen.close()
print('done')
"#,
    );
    assert_output(&output, "GeneratorExit caught\ndone\n");
}

/// AC4: close() on a generator that swallows GeneratorExit and yields again
/// raises RuntimeError whose message substring matches CPython ("generator
/// ignored GeneratorExit").
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4
#[test]
fn test_close_ignored_generatorexit_runtime_error() {
    let output = jit_capture(
        r#"def ignores_exit():
    try:
        yield 1
    except GeneratorExit:
        yield 2   # protocol violation — yields after GeneratorExit

gen = ignores_exit()
next(gen)
try:
    gen.close()
except RuntimeError as e:
    msg = str(e)
    if "generator ignored GeneratorExit" in msg:
        print('runtime error ok')
    else:
        print('runtime error wrong msg:', msg)
"#,
    );
    assert_output(&output, "runtime error ok\n");
}

/// AC4: finally block runs on close().
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4
#[test]
fn test_close_runs_finally_block() {
    let output = jit_capture(
        r#"def g():
    try:
        yield 1
        yield 2
    finally:
        print('finalized')

gen = g()
next(gen)
gen.close()
print('done')
"#,
    );
    assert_output(&output, "finalized\ndone\n");
}

/// AC4 / R8: close() on an unstarted generator is a no-op. The body never
/// runs, and subsequent close() calls are also silent.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC4
#[test]
fn test_close_unstarted_generator_is_silent() {
    let output = jit_capture(
        r#"def g():
    print('never runs')
    yield 1

gen = g()
gen.close()
gen.close()  # idempotent
print('silent ok')
"#,
    );
    assert_output(&output, "silent ok\n");
}
