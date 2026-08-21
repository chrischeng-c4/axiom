//! Async generator protocol conformance (AC6 — R7, PEP 525).
//!
//! Ported from CPython 3.12 Lib/test/test_asyncgen.py @ tag v3.12.0 subset
//! (AsyncGenSyntaxTest / AsyncGenAsyncioTest — with the minimal in-process
//! driver instead of asyncio, as asyncio task wrapping is out of scope).
//!
//! Async generators drive through the runtime via `async for`, `asend`,
//! `athrow`, `aclose`. As of this conformance port, Mamba's runtime does
//! NOT yet implement first-class async generators (no `ASYNC_GENERATOR`
//! object, no `__anext__` / `asend` methods on the Cranelift JIT path —
//! verified 2026-04 in `runtime/generator.rs`). Tests are therefore written
//! to document the expected CPython behaviour but are marked `#[ignore]`
//! with a TODO pointing at AC6. When runtime support lands, remove the
//! `#[ignore]` attribute.
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6

use super::{assert_output, jit_capture};

/// AC6: `async for` drains an async generator and collects its values.
///
/// TODO(#756): unignore once async-generator runtime support lands.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6
#[test]
#[ignore = "AC6: async generators not yet wired through JIT runtime (see runtime/generator.rs)"]
fn test_async_for_collects_values() {
    let output = jit_capture(
        r#"async def ag():
    yield 1
    yield 2

async def main():
    collected = []
    async for x in ag():
        collected.append(x)
    print(collected)

# Minimal in-process driver: call the coroutine's __await__ to completion.
coro = main()
try:
    while True:
        coro.send(None)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "[1, 2]\n");
}

/// AC6: `asend` round-trips a value into the async generator, mirroring the
/// sync `send` contract from AC2.
///
/// TODO(#756): unignore once async-generator runtime support lands.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6
#[test]
#[ignore = "AC6: async generators not yet wired through JIT runtime (see runtime/generator.rs)"]
fn test_asend_round_trip() {
    let output = jit_capture(
        r#"async def ag():
    v = yield 'ready'
    yield 'got:' + v

async def main():
    g = ag()
    first = await g.asend(None)
    print(first)
    second = await g.asend('abc')
    print(second)

coro = main()
try:
    while True:
        coro.send(None)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "ready\ngot:abc\n");
}

/// AC6: `athrow` injects an exception into the suspended async yield.
///
/// TODO(#756): unignore once async-generator runtime support lands.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6
#[test]
#[ignore = "AC6: async generators not yet wired through JIT runtime (see runtime/generator.rs)"]
fn test_athrow_caught_inside() {
    let output = jit_capture(
        r#"async def ag():
    try:
        yield 1
    except ValueError:
        yield 'caught'

async def main():
    g = ag()
    print(await g.asend(None))
    print(await g.athrow(ValueError('x')))

coro = main()
try:
    while True:
        coro.send(None)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "1\ncaught\n");
}

/// AC6: `aclose` shuts the async generator down cleanly without hanging
/// the driver loop.
///
/// TODO(#756): unignore once async-generator runtime support lands.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6
#[test]
#[ignore = "AC6: async generators not yet wired through JIT runtime (see runtime/generator.rs)"]
fn test_aclose_clean_shutdown() {
    let output = jit_capture(
        r#"async def ag():
    try:
        yield 1
    except GeneratorExit:
        print('closing')
        raise

async def main():
    g = ag()
    await g.asend(None)
    await g.aclose()
    print('done')

coro = main()
try:
    while True:
        coro.send(None)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "closing\ndone\n");
}

/// AC6: `async for` terminates cleanly on StopAsyncIteration, not
/// StopIteration. A sync-generator-style return must surface the proper
/// async terminator.
///
/// TODO(#756): unignore once async-generator runtime support lands.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#AC6
#[test]
#[ignore = "AC6: async generators not yet wired through JIT runtime (see runtime/generator.rs)"]
fn test_async_generator_stopasynciteration_terminates() {
    let output = jit_capture(
        r#"async def ag():
    yield 42

async def main():
    g = ag()
    print(await g.asend(None))
    try:
        await g.asend(None)
    except StopAsyncIteration:
        print('async stop')

coro = main()
try:
    while True:
        coro.send(None)
except StopIteration:
    pass
"#,
    );
    assert_output(&output, "42\nasync stop\n");
}
