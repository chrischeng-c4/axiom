//! `yield from` delegation conformance (AC5 — R5).
//!
//! Ported from CPython 3.12 Lib/test/test_generators.py @ tag v3.12.0
//! (YieldFromTests) and PEP 380 examples.
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5

use super::{assert_output, jit_capture};

/// AC5: `yield from` forwards values from the inner generator through the
/// outer one; the outer acts as a transparent conduit.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_forwards_values() {
    let output = jit_capture(
        r#"def inner():
    yield 1
    yield 2
    yield 3

def outer():
    yield from inner()
    yield 'after'

for v in outer():
    print(v)
"#,
    );
    assert_output(&output, "1\n2\n3\nafter\n");
}

/// AC5: send() is delegated through `yield from` to the inner generator.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_delegates_send() {
    let output = jit_capture(
        r#"def inner():
    received = yield 'ready'
    yield 'got-' + received

def outer():
    yield from inner()

g = outer()
print(next(g))
print(g.send('abc'))
"#,
    );
    assert_output(&output, "ready\ngot-abc\n");
}

/// AC5: throw() into outer generator is delegated to the inner sub-iterator
/// and caught at the inner yield point.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_delegates_throw_caught_inside() {
    let output = jit_capture(
        r#"def inner():
    try:
        yield 1
    except ValueError as e:
        yield 'caught:' + str(e)

def outer():
    yield from inner()

g = outer()
print(next(g))
print(g.throw(ValueError('boom')))
"#,
    );
    assert_output(&output, "1\ncaught:boom\n");
}

/// AC5: close() forwards GeneratorExit into the inner generator before it
/// reaches the outer.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_delegates_close() {
    let output = jit_capture(
        r#"def inner():
    try:
        yield 1
    except GeneratorExit:
        print('inner closed')
        raise

def outer():
    try:
        yield from inner()
    except GeneratorExit:
        print('outer closed')
        raise

g = outer()
next(g)
g.close()
"#,
    );
    assert_output(&output, "inner closed\nouter closed\n");
}

/// AC5: PEP 380 — `return expr` in the inner generator sets
/// StopIteration.value, which the outer consumes as the value of the
/// `yield from` expression.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_captures_inner_return_value() {
    let output = jit_capture(
        r#"def inner():
    yield 1
    return 'done'

def outer():
    result = yield from inner()
    yield result

g = outer()
print(next(g))
print(next(g))
"#,
    );
    assert_output(&output, "1\ndone\n");
}

/// AC5: `yield from` works with a non-generator iterable (list / range).
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC5
#[test]
fn test_yield_from_non_generator_iterable() {
    let output = jit_capture(
        r#"def outer():
    yield from [10, 20, 30]
    yield from range(3)

for v in outer():
    print(v)
"#,
    );
    assert_output(&output, "10\n20\n30\n0\n1\n2\n");
}
