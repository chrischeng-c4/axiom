# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_metaclass_keywords"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: a callable metaclass receives (name, bases, ns, **kwds) and non-'metaclass' keywords flow through as kwargs"""
import types


# A callable metaclass receives (name, bases, ns, **kwds); extra kwds flow
# through. The non-'metaclass' keyword 'x' arrives as a keyword arg.
def meta_func(name, bases, ns, **kw):
    return (name, bases, ns, kw)


res = types.new_class("X", (int, object), dict(metaclass=meta_func, x=0))
assert res == ("X", (int, object), {}, {"x": 0})

print("new_class_metaclass_keywords OK")
