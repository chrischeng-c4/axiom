use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/functools/cache_memoizes_recursive_fib.py`.
#[test]
fn test_gen_behavior_std_libs_functools_cache_memoizes_recursive_fib() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "cache_memoizes_recursive_fib"
# subject = "functools.cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cache: functools.cache memoizes a recursive fibonacci and a multi-arg adder so repeated keys return the cached value"""
import functools

# Recursive fibonacci: the body runs once per distinct n (12 distinct
# keys for fib(10): 0..10 plus the top call) thanks to memoization.
_fib_calls = 0


@functools.cache
def _fib(n: int) -> int:
    global _fib_calls
    _fib_calls += 1
    return n if n < 2 else _fib(n - 1) + _fib(n - 2)


assert _fib(10) == 55, f"fib(10) = {_fib(10)!r}"
assert _fib_calls == 11, f"fib body ran {_fib_calls!r} times"
assert _fib(10) == 55, "fib(10) cached"

_info = _fib.cache_info()
assert _info.misses == 11, f"fib misses = {_info.misses!r}"
assert _info.currsize == 11, f"fib currsize = {_info.currsize!r}"


# Multi-arg adder: repeated (a, b) keys hit the cache; the body runs once
# per distinct argument tuple.
_add_calls = 0


@functools.cache
def _add(a: int, b: int) -> int:
    global _add_calls
    _add_calls += 1
    return a + b


assert _add(2, 3) == 5, "add(2,3)"
assert _add(2, 3) == 5, "add(2,3) cached"
assert _add(4, 5) == 9, "add(4,5)"
assert _add_calls == 2, f"add body ran {_add_calls!r} times"

_ainfo = _add.cache_info()
assert _ainfo.hits == 1, f"add hits = {_ainfo.hits!r}"
assert _ainfo.misses == 2, f"add misses = {_ainfo.misses!r}"
assert _ainfo.currsize == 2, f"add currsize = {_ainfo.currsize!r}"

print("cache_memoizes_recursive_fib OK")
"###);
    assert_output(&out, r###"cache_memoizes_recursive_fib OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/cached_property_computes_once_per_instance.py`.
#[test]
fn test_gen_behavior_std_libs_functools_cached_property_computes_once_per_instance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "cached_property_computes_once_per_instance"
# subject = "functools.cached_property"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.cached_property: cached_property computes once per instance, keeps independent instances separate, allows manual overwrite, and works under inheritance"""
import functools


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.compute_calls = 0

    @functools.cached_property
    def magnitude_sq(self):
        self.compute_calls += 1
        return self.x * self.x + self.y * self.y


# Computed once per instance: repeated access reuses the stored value.
p = Point(3, 4)
assert p.magnitude_sq == 25, f"p.magnitude_sq = {p.magnitude_sq!r}"
assert p.magnitude_sq == 25, "p.magnitude_sq cached"
assert p.magnitude_sq == 25, "p.magnitude_sq still cached"
assert p.compute_calls == 1, f"p computed {p.compute_calls!r} times"

# A second instance is independent and does not affect p.
q = Point(5, 12)
assert q.magnitude_sq == 169, f"q.magnitude_sq = {q.magnitude_sq!r}"
assert q.compute_calls == 1, f"q computed {q.compute_calls!r} times"
assert p.compute_calls == 1, "p untouched by q"

# Manual overwrite: the descriptor only has __get__, so an explicit set
# stores a plain instance attribute that shadows the cached value.
p.magnitude_sq = 999
assert p.magnitude_sq == 999, f"p.magnitude_sq after set = {p.magnitude_sq!r}"


# cached_property is inherited and works on the subclass instance.
class Box:
    def __init__(self, n):
        self.n = n
        self.calls = 0

    @functools.cached_property
    def volume(self):
        self.calls += 1
        return self.n ** 3


class BigBox(Box):
    pass


b = BigBox(4)
assert b.volume == 64, f"b.volume = {b.volume!r}"
assert b.volume == 64, "b.volume cached"
assert b.calls == 1, f"b computed {b.calls!r} times"

print("cached_property_computes_once_per_instance OK")
"###);
    assert_output(&out, r###"cached_property_computes_once_per_instance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/lru_cache_bare_and_unbounded_forms.py`.
#[test]
fn test_gen_behavior_std_libs_functools_lru_cache_bare_and_unbounded_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_bare_and_unbounded_forms"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: the bare @lru_cache form and @lru_cache(maxsize=None) both cache; unbounded grows currsize with each distinct key"""
import functools

# Bare decorator form: `@lru_cache` with no parens still memoizes and
# defaults to maxsize=128.
_bare_calls = 0


@functools.lru_cache
def _double(n: int) -> int:
    global _bare_calls
    _bare_calls += 1
    return n * 2


assert _double(5) == 10, "double(5)"
assert _double(5) == 10, "double(5) cached"
assert _double(10) == 20, "double(10)"
assert _bare_calls == 2, f"bare body ran {_bare_calls!r} times"

_bare_info = _double.cache_info()
assert _bare_info.hits == 1, f"bare hits = {_bare_info.hits!r}"
assert _bare_info.misses == 2, f"bare misses = {_bare_info.misses!r}"
assert _bare_info.maxsize == 128, f"bare maxsize = {_bare_info.maxsize!r}"


# Unbounded form: maxsize=None never evicts, so currsize grows with each
# distinct key.
@functools.lru_cache(maxsize=None)
def _triple(n: int) -> int:
    return n * 3


for i in range(20):
    _triple(i)

_un_info = _triple.cache_info()
assert _un_info.maxsize is None, f"unbounded maxsize = {_un_info.maxsize!r}"
assert _un_info.currsize == 20, f"unbounded currsize = {_un_info.currsize!r}"

print("lru_cache_bare_and_unbounded_forms OK")
"###);
    assert_output(&out, r###"lru_cache_bare_and_unbounded_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/lru_cache_caches_and_counts_hits_misses.py`.
#[test]
fn test_gen_behavior_std_libs_functools_lru_cache_caches_and_counts_hits_misses() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_caches_and_counts_hits_misses"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache memoizes by args so the body runs once per distinct key, and cache_info tracks hits/misses"""
import functools

# The body runs once per distinct argument; repeated keys are served
# from the cache and counted as hits.
_call_count = 0


@functools.lru_cache(maxsize=8)
def _square(n: int) -> int:
    global _call_count
    _call_count += 1
    return n * n


assert _square(3) == 9, "square(3)"
assert _square(3) == 9, "square(3) cached"
assert _square(4) == 16, "square(4)"
assert _square(4) == 16, "square(4) cached"
assert _call_count == 2, f"body ran {_call_count!r} times"

_info = _square.cache_info()
assert _info.hits == 2, f"hits = {_info.hits!r}"
assert _info.misses == 2, f"misses = {_info.misses!r}"
assert _info.currsize == 2, f"currsize = {_info.currsize!r}"

print("lru_cache_caches_and_counts_hits_misses OK")
"###);
    assert_output(&out, r###"lru_cache_caches_and_counts_hits_misses OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/lru_cache_clear_resets_state.py`.
#[test]
fn test_gen_behavior_std_libs_functools_lru_cache_clear_resets_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_clear_resets_state"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: cache_clear zeroes hits/misses/currsize on a previously populated cache"""
import functools

@functools.lru_cache(maxsize=8)
def _square(n: int) -> int:
    return n * n


# Populate the cache so hits/misses/currsize are all non-zero.
_square(3)
_square(3)
_square(4)
_before = _square.cache_info()
assert _before.hits == 1, f"hits before clear = {_before.hits!r}"
assert _before.misses == 2, f"misses before clear = {_before.misses!r}"
assert _before.currsize == 2, f"currsize before clear = {_before.currsize!r}"

# cache_clear() drops every entry and zeroes the counters.
_square.cache_clear()
_after = _square.cache_info()
assert _after.hits == 0, f"hits after clear = {_after.hits!r}"
assert _after.misses == 0, f"misses after clear = {_after.misses!r}"
assert _after.currsize == 0, f"currsize after clear = {_after.currsize!r}"

print("lru_cache_clear_resets_state OK")
"###);
    assert_output(&out, r###"lru_cache_clear_resets_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/lru_cache_maxsize_evicts_lru.py`.
#[test]
fn test_gen_behavior_std_libs_functools_lru_cache_maxsize_evicts_lru() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "lru_cache_maxsize_evicts_lru"
# subject = "functools.lru_cache"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.lru_cache: lru_cache(maxsize=N) bounds currsize at N and evicts the least-recently-used entry"""
import functools

# maxsize=2 holds at most two entries. square(3) evicts the LRU entry
# square(1), so the later square(1) is a miss that recomputes the body.
_calls = 0


@functools.lru_cache(maxsize=2)
def _square(n: int) -> int:
    global _calls
    _calls += 1
    return n * n


assert _square(1) == 1, "square(1)"
assert _square(2) == 4, "square(2)"
assert _square(3) == 9, "square(3) evicts square(1)"
assert _square(1) == 1, "square(1) recomputed after eviction"
assert _calls == 4, f"body ran {_calls!r} times (no hits)"

_info = _square.cache_info()
assert _info.maxsize == 2, f"maxsize = {_info.maxsize!r}"
assert _info.currsize == 2, f"currsize bounded at 2 = {_info.currsize!r}"
assert _info.hits == 0, f"hits = {_info.hits!r}"
assert _info.misses == 4, f"misses = {_info.misses!r}"

print("lru_cache_maxsize_evicts_lru OK")
"###);
    assert_output(&out, r###"lru_cache_maxsize_evicts_lru OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_binds_positional_and_keyword.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_binds_positional_and_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_binds_positional_and_keyword"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial binds leading positional args and default keywords; call-time positionals append and call-time kwargs win on conflict"""
import functools


def capture(*args, **kw):
    return (args, kw)


# Bound positionals lead; call-time positionals append; call-time kwargs win.
p = functools.partial(capture, 1, 2, a=10, b=20)
assert p(3, 4, b=30, c=40) == ((1, 2, 3, 4), {"a": 10, "b": 30, "c": 40}), (
    f"merged call = {p(3, 4, b=30, c=40)!r}"
)


# A bound leading positional, exercised over a 3-arg function.
def _add(a, b, c):
    return a + b + c


assert functools.partial(_add, 5)(1, 2) == 8, "partial(add, 5)(1, 2)"
assert functools.partial(_add, 5, 6)(7) == 18, "partial(add, 5, 6)(7)"


# A bound default keyword.
def _greet(name, greeting="hi"):
    return f"{greeting} {name}"


assert functools.partial(_greet, greeting="hello")("Alice") == "hello Alice", "partial kw"

print("partial_binds_positional_and_keyword OK")
"###);
    assert_output(&out, r###"partial_binds_positional_and_keyword OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_copy_shares_members.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_copy_shares_members() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_copy_shares_members"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: copy.copy(partial) preserves func and shares the same args/keywords objects"""
import copy
import functools


def capture(*args, **kw):
    return (args, kw)


c = functools.partial(capture, ["asdf"], bar=[True])
c_copy = copy.copy(c)
assert c_copy.func is c.func, "copy shares func"
assert c_copy.args is c.args, "copy shares args"
assert c_copy.keywords is c.keywords, "copy shares keywords"

print("partial_copy_shares_members OK")
"###);
    assert_output(&out, r###"partial_copy_shares_members OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_exposes_func_args_keywords.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_exposes_func_args_keywords() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_exposes_func_args_keywords"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial.func/.args/.keywords expose the captured callable and bound arguments"""
import functools


def capture(*args, **kw):
    return (args, kw)


p = functools.partial(capture, 1, 2, a=10, b=20)
assert p.func is capture, "partial.func"
assert p.args == (1, 2), f"partial.args = {p.args!r}"
assert p.keywords == {"a": 10, "b": 20}, f"partial.keywords = {p.keywords!r}"

# A keyword-only binding leaves args empty.
q = functools.partial(max, 0)
assert q.func is max, "partial.func builtin"
assert q.args == (0,), f"partial.args = {q.args!r}"

print("partial_exposes_func_args_keywords OK")
"###);
    assert_output(&out, r###"partial_exposes_func_args_keywords OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_nesting_flattens.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_nesting_flattens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_nesting_flattens"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: a partial of a partial flattens: func resolves to the base callable and the merged signature equals a single flat partial"""
import functools


def capture(*args, **kw):
    return (args, kw)


def signature(part):
    return (part.func, part.args, part.keywords)


inner = functools.partial(capture, "asdf")
nested = functools.partial(inner, bar=True)
flat = functools.partial(capture, "asdf", bar=True)

assert nested.func is capture, "nested partial flattened func"
assert signature(nested) == signature(flat), "nested == flat signature"

print("partial_nesting_flattens OK")
"###);
    assert_output(&out, r###"partial_nesting_flattens OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_repr_names_type_and_args.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_repr_names_type_and_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_repr_names_type_and_args"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: repr(partial(...)) is module-qualified and reflects the captured positional args"""
import functools


def capture(*args, **kw):
    return (args, kw)


assert repr(functools.partial(capture)) == f"functools.partial({capture!r})", "repr bare"
assert repr(functools.partial(capture, 7)) == f"functools.partial({capture!r}, 7)", (
    "repr with positional"
)

print("partial_repr_names_type_and_args OK")
"###);
    assert_output(&out, r###"partial_repr_names_type_and_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partial_wraps_bound_and_unbound_methods.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partial_wraps_bound_and_unbound_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partial_wraps_bound_and_unbound_methods"
# subject = "functools.partial"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partial: partial wraps both an unbound (str.join) and a bound (''.join) method as the underlying callable"""
import functools

data = [str(i) for i in range(10)]

# Unbound method: the separator is the bound first positional.
join_unbound = functools.partial(str.join, "")
assert join_unbound(data) == "0123456789", "unbound str.join"

# Bound method: the partial wraps an already-bound callable.
join_bound = functools.partial("".join)
assert join_bound(data) == "0123456789", "bound ''.join"

print("partial_wraps_bound_and_unbound_methods OK")
"###);
    assert_output(&out, r###"partial_wraps_bound_and_unbound_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/partialmethod_binds_in_class_body.py`.
#[test]
fn test_gen_behavior_std_libs_functools_partialmethod_binds_in_class_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "partialmethod_binds_in_class_body"
# subject = "functools.partialmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.partialmethod: partialmethod binds a positional-only argument when declared in a class body"""
import functools


# partialmethod binds a leading positional-only argument inside the class
# body, so add_one(2) calls add(self, 1, 2).
class Adder:
    def add(self, a, b, /):
        return a + b

    add_one = functools.partialmethod(add, 1)


assert Adder().add_one(2) == 3, "partialmethod positional-only"
assert Adder().add_one(40) == 41, "partialmethod reused"


# partialmethod(None, ...) with a non-callable first arg raises TypeError
# at class definition time.
try:

    class Bad:
        m = functools.partialmethod(None, 1)

    raise AssertionError("expected TypeError for partialmethod(None, ...)")
except TypeError:
    pass

print("partialmethod_binds_in_class_body OK")
"###);
    assert_output(&out, r###"partialmethod_binds_in_class_body OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/reduce_fold_with_and_without_initial.py`.
#[test]
fn test_gen_behavior_std_libs_functools_reduce_fold_with_and_without_initial() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "reduce_fold_with_and_without_initial"
# subject = "functools.reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: fold a binary op over a list with and without an initial seed (sum, product, max), and a single-element / empty+initial short-circuit that never calls func"""
import functools

# Fold without an initial seed.
assert functools.reduce(lambda a, b: a + b, [1, 2, 3, 4]) == 10, "reduce sum"
assert functools.reduce(lambda a, b: a * b, [1, 2, 3, 4]) == 24, "reduce product"
assert functools.reduce(max, [3, 1, 4, 1, 5, 9]) == 9, "reduce max"

# Fold with an initial seed.
assert functools.reduce(lambda a, b: a + b, [1, 2, 3], 100) == 106, "reduce with initial"
assert functools.reduce(lambda a, b: a + b, [], 42) == 42, "empty + initial returns seed"

# A single-element sequence returns that element without calling func.
assert functools.reduce(42, "1") == "1", "single element skips func"
# An empty sequence with an initial likewise never calls func.
assert functools.reduce(42, "", "1") == "1", "empty + initial skips func"

print("reduce_fold_with_and_without_initial OK")
"###);
    assert_output(&out, r###"reduce_fold_with_and_without_initial OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/reduce_iterates_getitem_sequence_protocol.py`.
#[test]
fn test_gen_behavior_std_libs_functools_reduce_iterates_getitem_sequence_protocol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "reduce_iterates_getitem_sequence_protocol"
# subject = "functools.reduce"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.reduce: reduce drives an arbitrary __getitem__ sequence (IndexError stops), with and without an initial, including the empty custom sequence"""
import functools


class Squares:
    """A sequence-protocol object: __getitem__ raising IndexError to stop."""

    def __init__(self, count):
        self.count = count
        self.sofar = []

    def __getitem__(self, i):
        if not 0 <= i < self.count:
            raise IndexError
        while len(self.sofar) <= i:
            n = len(self.sofar)
            self.sofar.append(n * n)
        return self.sofar[i]


def _add(x, y):
    return x + y


# reduce iterates via __getitem__, not just over built-in lists.
assert functools.reduce(_add, Squares(10)) == 285, "reduce over custom seq"
assert functools.reduce(_add, Squares(10), 0) == 285, "reduce custom seq + initial"
assert functools.reduce(_add, Squares(0), 0) == 0, "reduce empty custom seq"

print("reduce_iterates_getitem_sequence_protocol OK")
"###);
    assert_output(&out, r###"reduce_iterates_getitem_sequence_protocol OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/singledispatch_dispatches_on_first_arg_type.py`.
#[test]
fn test_gen_behavior_std_libs_functools_singledispatch_dispatches_on_first_arg_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "singledispatch_dispatches_on_first_arg_type"
# subject = "functools.singledispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatch: singledispatch picks the implementation by the first argument's type via explicit register, decorator register, .dispatch, MRO, ABC, and annotation/union registration"""
import collections.abc
import functools
import typing


# Explicit register(type, impl): unregistered types fall back to base.
@functools.singledispatch
def describe(obj):
    return "base"


def _describe_int(i):
    return "integer"


describe.register(int, _describe_int)
assert describe("str") == "base", "str -> base"
assert describe(1) == "integer", "int -> integer"
assert describe([1, 2]) == "base", "list -> base"


# Decorator form @g.register(type) plus .dispatch lookups.
@functools.singledispatch
def kind(obj):
    return "default"


@kind.register(int)
def _kind_int(i):
    return f"int {i}"


assert kind("") == "default", "empty str default"
assert kind(12) == "int 12", "int dispatched"
assert kind.dispatch(int) is _kind_int, "dispatch(int)"
assert kind.dispatch(object) is kind.dispatch(str), "unregistered -> base impl"


# MRO resolution: D(C, B) with A and B registered prefers B over A.
@functools.singledispatch
def label(obj):
    return "base"


class A:
    pass


class C(A):
    pass


class B(A):
    pass


class D(C, B):
    pass


label.register(A, lambda o: "A")
label.register(B, lambda o: "B")
assert label(A()) == "A", "A -> A"
assert label(B()) == "B", "B -> B"
assert label(C()) == "A", "C inherits A"
assert label(D()) == "B", "D(C,B) prefers B"


# ABC registration: concrete types match the most specific abstract base.
@functools.singledispatch
def abc_kind(obj):
    return "base"


abc_kind.register(collections.abc.Sequence, lambda o: "sequence")
abc_kind.register(collections.abc.MutableSequence, lambda o: "mutableseq")
assert abc_kind((1, 2)) == "sequence", "tuple -> sequence"
assert abc_kind([1, 2]) == "mutableseq", "list -> mutableseq"


# Annotation-based register: the type comes from the parameter annotation.
@functools.singledispatch
def via_ann(arg):
    return "base"


@via_ann.register
def _(arg: collections.abc.Mapping):
    return "mapping"


assert via_ann(None) == "base", "None -> base"
assert via_ann({"a": 1}) == "mapping", "dict -> mapping"


# Union annotations (typing.Union and X | Y) both register.
@functools.singledispatch
def uni(arg):
    return "default"


@uni.register
def _(arg: typing.Union[str, bytes]):
    return "union"


@uni.register
def _(arg: int | float):
    return "uniontype"


assert uni([]) == "default", "list default"
assert uni("") == "union", "str union"
assert uni(b"") == "union", "bytes union"
assert uni(1) == "uniontype", "int uniontype"
assert uni(1.0) == "uniontype", "float uniontype"

print("singledispatch_dispatches_on_first_arg_type OK")
"###);
    assert_output(&out, r###"singledispatch_dispatches_on_first_arg_type OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/singledispatchmethod_dispatches_with_classmethod_staticmethod.py`.
#[test]
fn test_gen_behavior_std_libs_functools_singledispatchmethod_dispatches_with_classmethod_staticmethod() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "singledispatchmethod_dispatches_with_classmethod_staticmethod"
# subject = "functools.singledispatchmethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatchmethod: singledispatchmethod dispatches a method by its first non-self argument and stacks with classmethod/staticmethod, threading cls/self correctly"""
import functools


# Plain method dispatch: each registered type sets a different value, and
# only the receiving instance is mutated.
class Recorder:
    @functools.singledispatchmethod
    def handle(self, arg):
        self.arg = "base"

    @handle.register(int)
    def _(self, arg):
        self.arg = "int"

    @handle.register(str)
    def _(self, arg):
        self.arg = "str"


a = Recorder()
a.handle(0)
assert a.arg == "int", "int -> int"

fresh = Recorder()
assert not hasattr(fresh, "arg"), "untouched instance has no arg"

a.handle("x")
assert a.arg == "str", "str -> str"

a.handle(0.0)
assert a.arg == "base", "float -> base"


# classmethod stacking: dispatch works and cls is threaded through.
class Factory:
    def __init__(self, tag):
        self.tag = tag

    @functools.singledispatchmethod
    @classmethod
    def make(cls, arg):
        return cls("base")

    @make.register(int)
    @classmethod
    def _(cls, arg):
        return cls("int")

    @make.register(str)
    @classmethod
    def _(cls, arg):
        return cls("str")


assert Factory.make(0).tag == "int", "classmethod int"
assert Factory.make("").tag == "str", "classmethod str"
assert Factory.make(0.0).tag == "base", "classmethod float"


# staticmethod stacking: no self/cls is passed to the implementations.
class Checker:
    @functools.singledispatchmethod
    @staticmethod
    def check(arg):
        return "base"

    @check.register(int)
    @staticmethod
    def _(arg):
        return isinstance(arg, int)

    @check.register(str)
    @staticmethod
    def _(arg):
        return isinstance(arg, str)


assert Checker.check(0) is True, "staticmethod int"
assert Checker.check("") is True, "staticmethod str"
assert Checker.check(0.0) == "base", "staticmethod float"

print("singledispatchmethod_dispatches_with_classmethod_staticmethod OK")
"###);
    assert_output(&out, r###"singledispatchmethod_dispatches_with_classmethod_staticmethod OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/total_ordering_derives_from_any_seed_op.py`.
#[test]
fn test_gen_behavior_std_libs_functools_total_ordering_derives_from_any_seed_op() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_derives_from_any_seed_op"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering seeded from __lt__ or __ge__ derives the other three ordering operators consistently, including equal-value boundaries"""
import functools


# total_ordering can seed from any one ordering op; whichever is provided,
# the other three are derived consistently.
@functools.total_ordering
class FromLt:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return self.v == other.v

    def __lt__(self, other):
        return self.v < other.v


@functools.total_ordering
class FromGe:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return self.v == other.v

    def __ge__(self, other):
        return self.v >= other.v


for cls in (FromLt, FromGe):
    lo, hi = cls(1), cls(2)
    assert lo < hi, f"{cls.__name__}: lt"
    assert lo <= hi, f"{cls.__name__}: le"
    assert hi > lo, f"{cls.__name__}: gt"
    assert hi >= lo, f"{cls.__name__}: ge"
    assert lo <= cls(1), f"{cls.__name__}: le equal"
    assert hi >= cls(2), f"{cls.__name__}: ge equal"
    assert not (lo < cls(1)), f"{cls.__name__}: lt equal is False"
    assert not (hi > cls(2)), f"{cls.__name__}: gt equal is False"

print("total_ordering_derives_from_any_seed_op OK")
"###);
    assert_output(&out, r###"total_ordering_derives_from_any_seed_op OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/total_ordering_does_not_override_inherited.py`.
#[test]
fn test_gen_behavior_std_libs_functools_total_ordering_does_not_override_inherited() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_does_not_override_inherited"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: total_ordering does not overwrite rich comparisons inherited from a base (an int subclass keeps int's ordering)"""
import functools


# total_ordering does not overwrite ordering ops already inherited from a
# base; subclassing int keeps int's own rich comparisons.
@functools.total_ordering
class MyInt(int):
    pass


assert MyInt(1) < MyInt(2), "int subclass lt"
assert MyInt(2) > MyInt(1), "int subclass gt"
assert MyInt(2) >= MyInt(2), "int subclass ge equal"
assert MyInt(1) <= MyInt(1), "int subclass le equal"

# The inherited comparison methods are int's, not synthesized stand-ins.
assert MyInt.__lt__ is int.__lt__, "lt not overwritten"
assert MyInt.__gt__ is int.__gt__, "gt not overwritten"

print("total_ordering_does_not_override_inherited OK")
"###);
    assert_output(&out, r###"total_ordering_does_not_override_inherited OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/total_ordering_propagates_notimplemented.py`.
#[test]
fn test_gen_behavior_std_libs_functools_total_ordering_propagates_notimplemented() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "total_ordering_propagates_notimplemented"
# subject = "functools.total_ordering"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.total_ordering: when the seed comparison returns NotImplemented, the derived operators also return NotImplemented rather than guessing"""
import functools


# When the seed op returns NotImplemented (here for a foreign type), the
# derived ops propagate NotImplemented instead of guessing a result.
@functools.total_ordering
class OnlyLt:
    def __init__(self, v):
        self.v = v

    def __eq__(self, other):
        return isinstance(other, OnlyLt) and self.v == other.v

    def __lt__(self, other):
        if isinstance(other, OnlyLt):
            return self.v < other.v
        return NotImplemented


obj = OnlyLt(1)
assert obj.__le__(1) is NotImplemented, "le passthrough NotImplemented"
assert obj.__gt__(1) is NotImplemented, "gt passthrough NotImplemented"
assert obj.__ge__(1) is NotImplemented, "ge passthrough NotImplemented"

# For same-type operands the derived ops still produce real results.
assert obj < OnlyLt(2), "lt same type"
assert obj <= OnlyLt(1), "le same type equal"

print("total_ordering_propagates_notimplemented OK")
"###);
    assert_output(&out, r###"total_ordering_propagates_notimplemented OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/update_wrapper_custom_assigned_updated_tuples.py`.
#[test]
fn test_gen_behavior_std_libs_functools_update_wrapper_custom_assigned_updated_tuples() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "update_wrapper_custom_assigned_updated_tuples"
# subject = "functools.update_wrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: custom assigned tuples skip a missing source attribute and custom updated tuples merge mapping attributes in place"""
import functools


def src():
    pass


def wrapper():
    pass


# A missing assigned attribute on the source is simply skipped; an updated
# mapping attribute is merged in place (src has none, so it stays empty).
wrapper.dict_attr = {}
functools.update_wrapper(wrapper, src, assigned=("attr",), updated=("dict_attr",))
assert "attr" not in wrapper.__dict__, "missing assigned attr skipped"
assert wrapper.dict_attr == {}, "updated dict stays empty (src has none)"


# An updated attribute that is missing on the wrapper raises AttributeError.
del wrapper.dict_attr
try:
    functools.update_wrapper(
        wrapper, src, assigned=("attr",), updated=("dict_attr",)
    )
    raise AssertionError("expected AttributeError for missing updated attr")
except AttributeError:
    pass


# An updated attribute that is not a mapping also raises AttributeError
# (an int has no .update()).
wrapper.dict_attr = 1
try:
    functools.update_wrapper(
        wrapper, src, assigned=("attr",), updated=("dict_attr",)
    )
    raise AssertionError("expected AttributeError for non-mapping updated attr")
except AttributeError:
    pass

print("update_wrapper_custom_assigned_updated_tuples OK")
"###);
    assert_output(&out, r###"update_wrapper_custom_assigned_updated_tuples OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/update_wrapper_from_builtin_resets_annotations.py`.
#[test]
fn test_gen_behavior_std_libs_functools_update_wrapper_from_builtin_resets_annotations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "update_wrapper_from_builtin_resets_annotations"
# subject = "functools.update_wrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.update_wrapper: update_wrapper from a builtin (max, type) copies name/doc and resets __annotations__ to empty"""
import functools


# Updating from the `max` builtin copies its name/doc and resets the
# wrapper's annotations to an empty dict.
def from_builtin():
    pass


functools.update_wrapper(from_builtin, max)
assert from_builtin.__name__ == "max", f"name = {from_builtin.__name__!r}"
assert from_builtin.__doc__.startswith("max("), f"doc = {from_builtin.__doc__!r}"
assert from_builtin.__annotations__ == {}, "annotations reset"


# Updating from the `type` builtin also yields empty annotations and an
# empty __type_params__ tuple.
def from_type(*args):
    pass


functools.update_wrapper(from_type, type)
assert from_type.__name__ == "type", f"name = {from_type.__name__!r}"
assert from_type.__annotations__ == {}, "type annotations"
assert from_type.__type_params__ == (), "type __type_params__ empty"

print("update_wrapper_from_builtin_resets_annotations OK")
"###);
    assert_output(&out, r###"update_wrapper_from_builtin_resets_annotations OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/functools/wraps_copies_name_and_doc.py`.
#[test]
fn test_gen_behavior_std_libs_functools_wraps_copies_name_and_doc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "wraps_copies_name_and_doc"
# subject = "functools.wraps"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.wraps: @wraps copies __name__ and __doc__ from the wrapped function to the wrapper and sets __wrapped__"""
import functools


# @wraps copies __name__/__doc__ from the wrapped function onto the
# wrapper and records the original under __wrapped__.
def _log(fn):
    @functools.wraps(fn)
    def _w(*args, **kwargs):
        return fn(*args, **kwargs)

    return _w


@_log
def _target(x: int) -> int:
    """Target docstring."""
    return x * 2


assert _target.__name__ == "_target", f"__name__ = {_target.__name__!r}"
assert _target.__doc__ == "Target docstring.", f"__doc__ = {_target.__doc__!r}"
assert _target.__wrapped__ is not None, "__wrapped__ missing"
assert _target(21) == 42, "wrapper still calls through"

print("wraps_copies_name_and_doc OK")
"###);
    assert_output(&out, r###"wraps_copies_name_and_doc OK
"###);
}
