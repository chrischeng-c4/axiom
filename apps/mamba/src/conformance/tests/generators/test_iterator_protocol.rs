//! Iterator protocol conformance — user-defined __iter__ / __next__ (R6).
//!
//! Ported from CPython 3.12 Lib/test/test_iter.py @ tag v3.12.0 subset
//! (TestCase.test_iter_class, test_iter_next).
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6

use super::{assert_output, jit_capture};

/// R6: user class with __iter__ returning self and __next__ raising
/// StopIteration integrates with the `for` statement without over-advancing.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_user_iter_class_for_loop() {
    let output = jit_capture(
        r#"class Counter:
    def __init__(self, limit):
        self.limit = limit
        self.n = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.n >= self.limit:
            raise StopIteration
        val = self.n
        self.n += 1
        return val

for x in Counter(4):
    print(x)
"#,
    );
    assert_output(&output, "0\n1\n2\n3\n");
}

/// R6: user iterator works in list() comprehension — no over-advance, no
/// leaked StopIteration.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_user_iter_class_in_list_constructor() {
    let output = jit_capture(
        r#"class Range3:
    def __init__(self):
        self.n = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.n >= 3:
            raise StopIteration
        self.n += 1
        return self.n * 10

print(list(Range3()))
"#,
    );
    assert_output(&output, "[10, 20, 30]\n");
}

/// R6: manual iteration via next() on a user iterator raises StopIteration
/// after exhaustion.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_user_iter_class_manual_next() {
    let output = jit_capture(
        r#"class Two:
    def __init__(self):
        self.n = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.n >= 2:
            raise StopIteration
        self.n += 1
        return self.n

it = Two()
print(next(it))
print(next(it))
try:
    next(it)
except StopIteration:
    print('stopped')
"#,
    );
    assert_output(&output, "1\n2\nstopped\n");
}

/// R6: iterator protocol interacts with sum() — numeric __next__.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_user_iter_class_in_sum() {
    let output = jit_capture(
        r#"class N:
    def __init__(self, limit):
        self.limit = limit
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= self.limit:
            raise StopIteration
        self.i += 1
        return self.i

print(sum(N(5)))
"#,
    );
    assert_output(&output, "15\n");
}

/// R6: separate __iter__() helper object (not self) — iter() returns a
/// fresh iterator each time, enabling re-iteration.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_iter_returns_helper_object() {
    let output = jit_capture(
        r#"class Iter:
    def __init__(self, data):
        self.data = data
        self.i = 0
    def __iter__(self):
        return self
    def __next__(self):
        if self.i >= len(self.data):
            raise StopIteration
        val = self.data[self.i]
        self.i += 1
        return val

class Collection:
    def __init__(self, data):
        self.data = data
    def __iter__(self):
        return Iter(self.data)

c = Collection([1, 2, 3])
for v in c:
    print(v)
# Re-iterate — must work because __iter__ returns a fresh Iter each call.
for v in c:
    print(v)
"#,
    );
    assert_output(&output, "1\n2\n3\n1\n2\n3\n");
}

/// R6: StopIteration instance with a .value attribute is still legal (the
/// value is ignored by plain iteration but may be consumed by `yield from`).
/// This guards against accidental over-advance when the exception carries a
/// payload.
///
/// @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md#R6
#[test]
fn test_stopiteration_with_value_ends_iteration_cleanly() {
    let output = jit_capture(
        r#"class Once:
    def __init__(self):
        self.done = False
    def __iter__(self):
        return self
    def __next__(self):
        if self.done:
            raise StopIteration('payload')
        self.done = True
        return 42

collected = []
for v in Once():
    collected.append(v)
print(collected)
"#,
    );
    assert_output(&output, "[42]\n");
}
