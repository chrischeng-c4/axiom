# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "a_s_t__tests__test_classattrs"
# subject = "cpython.test_ast.AST_Tests.test_classattrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import ast

x = ast.Constant()
if x._fields != ("value", "kind"):
    raise AssertionError(x._fields)

try:
    x.value
except AttributeError:
    pass
else:
    raise AssertionError("ast.Constant().value should be missing")

x = ast.Constant(42)
if x.value != 42:
    raise AssertionError(x.value)

try:
    x.lineno
except AttributeError:
    pass
else:
    raise AssertionError("ast.Constant(42).lineno should be missing")

try:
    x.foobar
except AttributeError:
    pass
else:
    raise AssertionError("ast.Constant(42).foobar should be missing")

x = ast.Constant(lineno=2)
if x.lineno != 2:
    raise AssertionError(x.lineno)

x = ast.Constant(42, lineno=0)
if x.lineno != 0:
    raise AssertionError(x.lineno)
if x._fields != ("value", "kind"):
    raise AssertionError(x._fields)
if x.value != 42:
    raise AssertionError(x.value)

try:
    ast.Constant(1, None, 2)
except TypeError:
    pass
else:
    raise AssertionError("extra positional argument should fail")

try:
    ast.Constant(1, None, 2, lineno=0)
except TypeError:
    pass
else:
    raise AssertionError("extra positional argument with keyword should fail")

if ast.Constant(1, foo="bar").foo != "bar":
    raise AssertionError("foo")

try:
    ast.Constant(1, value=2)
except TypeError:
    pass
else:
    raise AssertionError("duplicate value should fail")

if ast.Constant(42).value != 42:
    raise AssertionError("int")
if ast.Constant(4.25).value != 4.25:
    raise AssertionError("float")
if ast.Constant(4.25j).value != 4.25j:
    raise AssertionError("complex")
if ast.Constant("42").value != "42":
    raise AssertionError("str")
if ast.Constant(b"42").value != b"42":
    raise AssertionError("bytes")
if ast.Constant(True).value is not True:
    raise AssertionError("true")
if ast.Constant(False).value is not False:
    raise AssertionError("false")
if ast.Constant(None).value is not None:
    raise AssertionError("none")
if ast.Constant(...).value is not ...:
    raise AssertionError("ellipsis")

print("AST_Tests::test_classattrs: ok")
