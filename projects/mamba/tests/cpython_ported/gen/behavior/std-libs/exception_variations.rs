use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested
"""Auto-ported test: ExceptStarTestCases::test_nested (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_finally = False
hit_inner_except = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except* BaseException:
        hit_inner_except = True
    finally:
        hit_inner_finally = True
finally:
    hit_finally = True

assert hit_inner_except

assert hit_inner_finally

assert hit_finally
print("ExceptStarTestCases::test_nested: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested_else.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_else"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_else
"""Auto-ported test: ExceptStarTestCases::test_nested_else (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except* BaseException:
        hit_inner_except = True
    else:
        hit_inner_else = True
    raise Exception('outer exception')
except* BaseException:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptStarTestCases::test_nested_else: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested_else_mixed1.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested_else_mixed1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_else_mixed1"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_else_mixed1"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_else_mixed1
"""Auto-ported test: ExceptStarTestCases::test_nested_else_mixed1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except* BaseException:
        hit_inner_except = True
    else:
        hit_inner_else = True
    raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptStarTestCases::test_nested_else_mixed1: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested_else_mixed1: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested_else_mixed2.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested_else_mixed2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_else_mixed2"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_else_mixed2"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_else_mixed2
"""Auto-ported test: ExceptStarTestCases::test_nested_else_mixed2 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
    raise Exception('outer exception')
except* BaseException:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptStarTestCases::test_nested_else_mixed2: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested_else_mixed2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested_mixed1.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested_mixed1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_mixed1"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_mixed1"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_mixed1
"""Auto-ported test: ExceptStarTestCases::test_nested_mixed1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
hit_inner_except = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except* BaseException:
        hit_inner_except = True
    finally:
        hit_inner_finally = True
except:
    hit_except = True
finally:
    hit_finally = True

assert hit_inner_except

assert hit_inner_finally

assert not hit_except

assert hit_finally
print("ExceptStarTestCases::test_nested_mixed1: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested_mixed1: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_nested_mixed2.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_nested_mixed2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_nested_mixed2"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_nested_mixed2"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_nested_mixed2
"""Auto-ported test: ExceptStarTestCases::test_nested_mixed2 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
hit_inner_except = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
    finally:
        hit_inner_finally = True
except* BaseException:
    hit_except = True
finally:
    hit_finally = True

assert hit_inner_except

assert hit_inner_finally

assert not hit_except

assert hit_finally
print("ExceptStarTestCases::test_nested_mixed2: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_nested_mixed2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except
"""Auto-ported test: ExceptStarTestCases::test_try_except (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    raise Exception('ahoy!')
except* BaseException:
    hit_except = True

assert hit_except
print("ExceptStarTestCases::test_try_except: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_else.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_else"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_else
"""Auto-ported test: ExceptStarTestCases::test_try_except_else (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
try:
    raise Exception('foo!')
except* BaseException:
    hit_except = True
else:
    hit_else = True

assert not hit_else

assert hit_except
print("ExceptStarTestCases::test_try_except_else: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_else_finally.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_else_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_else_finally"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_else_finally"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_else_finally
"""Auto-ported test: ExceptStarTestCases::test_try_except_else_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
hit_finally = False
try:
    raise Exception('nyaa!')
except* BaseException:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally

assert not hit_else
print("ExceptStarTestCases::test_try_except_else_finally: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_else_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_else_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_else_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_else_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_else_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_else_finally_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_except_else_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
hit_finally = False
try:
    pass
except* BaseException:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_except

assert hit_finally

assert hit_else
print("ExceptStarTestCases::test_try_except_else_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_else_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_else_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_else_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_else_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_else_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_else_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_except_else_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
try:
    pass
except* BaseException:
    hit_except = True
else:
    hit_else = True

assert not hit_except

assert hit_else
print("ExceptStarTestCases::test_try_except_else_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_else_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_finally.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_finally"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_finally"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_finally
"""Auto-ported test: ExceptStarTestCases::test_try_except_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    raise Exception('yarr!')
except* BaseException:
    hit_except = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally
print("ExceptStarTestCases::test_try_except_finally: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_finally_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_except_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    pass
except* BaseException:
    hit_except = True
finally:
    hit_finally = True

assert not hit_except

assert hit_finally
print("ExceptStarTestCases::test_try_except_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_except_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_except_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_except_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_except_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_except_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_except_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    pass
except* BaseException:
    hit_except = True

assert not hit_except
print("ExceptStarTestCases::test_try_except_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_except_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_star_test_cases__test_try_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_star_test_cases__test_try_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_star_test_cases__test_try_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptStarTestCases.test_try_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptStarTestCases::test_try_finally_no_exception
"""Auto-ported test: ExceptStarTestCases::test_try_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_finally = False
try:
    pass
finally:
    hit_finally = True

assert hit_finally
print("ExceptStarTestCases::test_try_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptStarTestCases::test_try_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested
"""Auto-ported test: ExceptTestCases::test_nested (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_finally = False
hit_inner_except = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
    finally:
        hit_inner_finally = True
finally:
    hit_finally = True

assert hit_inner_except

assert hit_inner_finally

assert hit_finally
print("ExceptTestCases::test_nested: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested_else.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_else"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_else
"""Auto-ported test: ExceptTestCases::test_nested_else (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
    raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_else: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested_exception_in_else.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested_exception_in_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_else"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_else
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_else (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        pass
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
        raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_else: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested_exception_in_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested_exception_in_except.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested_exception_in_except() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_except"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_except"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_except
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_except (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
        raise Exception('outer exception')
    else:
        hit_inner_else = True
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert hit_inner_except

assert not hit_inner_else

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_except: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested_exception_in_except: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested_exception_in_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested_exception_in_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_finally_no_exception
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
hit_inner_finally = False
try:
    try:
        pass
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
    finally:
        hit_inner_finally = True
        raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_inner_except

assert hit_inner_else

assert hit_inner_finally

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested_exception_in_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_nested_exception_in_finally_with_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_nested_exception_in_finally_with_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_nested_exception_in_finally_with_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_nested_exception_in_finally_with_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_nested_exception_in_finally_with_exception
"""Auto-ported test: ExceptTestCases::test_nested_exception_in_finally_with_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_else = False
hit_finally = False
hit_except = False
hit_inner_except = False
hit_inner_else = False
hit_inner_finally = False
try:
    try:
        raise Exception('inner exception')
    except:
        hit_inner_except = True
    else:
        hit_inner_else = True
    finally:
        hit_inner_finally = True
        raise Exception('outer exception')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert hit_inner_except

assert not hit_inner_else

assert hit_inner_finally

assert not hit_else

assert hit_finally

assert hit_except
print("ExceptTestCases::test_nested_exception_in_finally_with_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_nested_exception_in_finally_with_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except
"""Auto-ported test: ExceptTestCases::test_try_except (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    raise Exception('ahoy!')
except:
    hit_except = True

assert hit_except
print("ExceptTestCases::test_try_except: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_else.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_else() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_else"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_else"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_else
"""Auto-ported test: ExceptTestCases::test_try_except_else (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
try:
    raise Exception('foo!')
except:
    hit_except = True
else:
    hit_else = True

assert not hit_else

assert hit_except
print("ExceptTestCases::test_try_except_else: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_else: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_else_finally.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_else_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_else_finally"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_else_finally"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_else_finally
"""Auto-ported test: ExceptTestCases::test_try_except_else_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
hit_finally = False
try:
    raise Exception('nyaa!')
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally

assert not hit_else
print("ExceptTestCases::test_try_except_else_finally: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_else_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_else_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_else_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_else_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_else_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_else_finally_no_exception
"""Auto-ported test: ExceptTestCases::test_try_except_else_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
hit_finally = False
try:
    pass
except:
    hit_except = True
else:
    hit_else = True
finally:
    hit_finally = True

assert not hit_except

assert hit_finally

assert hit_else
print("ExceptTestCases::test_try_except_else_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_else_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_else_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_else_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_else_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_else_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_else_no_exception
"""Auto-ported test: ExceptTestCases::test_try_except_else_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_else = False
try:
    pass
except:
    hit_except = True
else:
    hit_else = True

assert not hit_except

assert hit_else
print("ExceptTestCases::test_try_except_else_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_else_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_finally.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_finally() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_finally"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_finally"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_finally
"""Auto-ported test: ExceptTestCases::test_try_except_finally (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    raise Exception('yarr!')
except:
    hit_except = True
finally:
    hit_finally = True

assert hit_except

assert hit_finally
print("ExceptTestCases::test_try_except_finally: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_finally: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_finally_no_exception
"""Auto-ported test: ExceptTestCases::test_try_except_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
hit_finally = False
try:
    pass
except:
    hit_except = True
finally:
    hit_finally = True

assert not hit_except

assert hit_finally
print("ExceptTestCases::test_try_except_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_finally_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_except_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_except_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_except_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_except_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_except_no_exception
"""Auto-ported test: ExceptTestCases::test_try_except_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_except = False
try:
    pass
except:
    hit_except = True

assert not hit_except
print("ExceptTestCases::test_try_except_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_except_no_exception: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/exception_variations/except_test_cases__test_try_finally_no_exception.py`.
#[test]
fn test_gen_behavior_std_libs_exception_variations_except_test_cases__test_try_finally_no_exception() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "exception_variations"
# dimension = "behavior"
# case = "except_test_cases__test_try_finally_no_exception"
# subject = "cpython.test_exception_variations.ExceptTestCases.test_try_finally_no_exception"
# kind = "semantic"
# mem_carveout = ""
# source = "Lib/test/test_exception_variations.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_exception_variations.py::ExceptTestCases::test_try_finally_no_exception
"""Auto-ported test: ExceptTestCases::test_try_finally_no_exception (CPython 3.12 oracle)."""


import unittest


# --- test body ---
hit_finally = False
try:
    pass
finally:
    hit_finally = True

assert hit_finally
print("ExceptTestCases::test_try_finally_no_exception: ok")
"###);
    assert_output(&out, r###"ExceptTestCases::test_try_finally_no_exception: ok
"###);
}
