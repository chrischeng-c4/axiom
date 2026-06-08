# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "code"
# dimension = "behavior"
# case = "code_constructor_roundtrip"
# subject = "types.CodeType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_code.py"
# status = "filled"
# ///
"""types.CodeType: rebuilding a CodeType from an existing code object's full field list (the 18-arg 3.12 constructor) preserves co_name and co_argcount"""
import types


def sample(a, b, *, z=1, w=2):
    x = a + b
    return x


co = sample.__code__
CodeType = type(co)
rebuilt = CodeType(
    co.co_argcount, co.co_posonlyargcount, co.co_kwonlyargcount,
    co.co_nlocals, co.co_stacksize, co.co_flags, co.co_code,
    co.co_consts, co.co_names, co.co_varnames, co.co_filename,
    co.co_name, co.co_qualname, co.co_firstlineno, co.co_linetable,
    co.co_exceptiontable, co.co_freevars, co.co_cellvars,
)
assert rebuilt.co_name == co.co_name, "constructor round-trip preserves name"
assert rebuilt.co_argcount == co.co_argcount, "round-trip preserves argcount"

print("code_constructor_roundtrip OK")
