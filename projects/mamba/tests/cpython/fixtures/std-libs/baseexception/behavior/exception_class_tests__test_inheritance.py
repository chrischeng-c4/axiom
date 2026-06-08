# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "baseexception"
# dimension = "behavior"
# case = "exception_class_tests__test_inheritance"
# subject = "cpython.test_baseexception.ExceptionClassTests.test_inheritance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_baseexception.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_baseexception.py::ExceptionClassTests::test_inheritance
"""Auto-ported test: ExceptionClassTests::test_inheritance (CPython 3.12 oracle)."""


import builtins
import os
import test
from platform import system as platform_system


def verify_instance_interface(ins):
    for attr in ("args", "__str__", "__repr__"):
        assert hasattr(ins, attr), f"{ins.__class__.__name__} missing {attr} attribute"


exc_set = set()
for object_ in builtins.__dict__.values():
    try:
        if issubclass(object_, BaseException):
            exc_set.add(object_.__name__)
    except TypeError:
        pass

test_dir = os.path.dirname(test.__file__)
inheritance_path = os.path.join(test_dir, "exception_hierarchy.txt")
with open(inheritance_path, encoding="utf-8") as inheritance_tree:
    superclass_name = inheritance_tree.readline().rstrip()
    last_exc = getattr(builtins, superclass_name)
    assert superclass_name in exc_set, f"{superclass_name} not found"
    exc_set.discard(superclass_name)
    superclasses = []
    last_depth = 0

    for exc_line in inheritance_tree:
        exc_line = exc_line.rstrip()
        depth = exc_line.rindex("\u2500")
        exc_name = exc_line[depth + 2:]
        if "(" in exc_name:
            paren_index = exc_name.index("(")
            platform_name = exc_name[paren_index + 1:-1]
            exc_name = exc_name[:paren_index - 1]
            if platform_system() != platform_name:
                exc_set.discard(exc_name)
                continue
        if "[" in exc_name:
            left_bracket = exc_name.index("[")
            exc_name = exc_name[:left_bracket - 1]

        exc = getattr(builtins, exc_name)
        if last_depth < depth:
            superclasses.append((last_depth, last_exc))
        elif last_depth > depth:
            while superclasses[-1][0] >= depth:
                superclasses.pop()

        assert issubclass(exc, superclasses[-1][1]), (
            f"{exc.__name__} is not a subclass of {superclasses[-1][1].__name__}"
        )
        try:
            verify_instance_interface(exc())
        except TypeError:
            pass
        assert exc_name in exc_set
        exc_set.discard(exc_name)
        last_exc = exc
        last_depth = depth

assert len(exc_set) == 0, f"{exc_set} not accounted for"
print("ExceptionClassTests::test_inheritance: ok")
