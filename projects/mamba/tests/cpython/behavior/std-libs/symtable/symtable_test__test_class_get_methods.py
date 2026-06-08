# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "behavior"
# case = "symtable_test__test_class_get_methods"
# subject = "cpython.test_symtable.SymtableTest.test_class_get_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_symtable.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: SymtableTest::test_class_get_methods (CPython 3.12 oracle)."""

import symtable
import textwrap


TEST_CODE = """
class Mine:
    instance_var = 24
    def a_method(p1, p2):
        pass
"""

TEST_COMPLEX_CLASS_CODE = """
# The following symbols are defined in ComplexClass
# without being introduced by a 'global' statement.
glob_unassigned_meth: Any
glob_unassigned_meth_pep_695: Any

glob_unassigned_async_meth: Any
glob_unassigned_async_meth_pep_695: Any

def glob_assigned_meth(): pass
def glob_assigned_meth_pep_695[T](): pass

async def glob_assigned_async_meth(): pass
async def glob_assigned_async_meth_pep_695[T](): pass

# The following symbols are defined in ComplexClass after
# being introduced by a 'global' statement (and therefore
# are not considered as local symbols of ComplexClass).
glob_unassigned_meth_ignore: Any
glob_unassigned_meth_pep_695_ignore: Any

glob_unassigned_async_meth_ignore: Any
glob_unassigned_async_meth_pep_695_ignore: Any

def glob_assigned_meth_ignore(): pass
def glob_assigned_meth_pep_695_ignore[T](): pass

async def glob_assigned_async_meth_ignore(): pass
async def glob_assigned_async_meth_pep_695_ignore[T](): pass

class ComplexClass:
    a_var = 1234
    a_genexpr = (x for x in [])
    a_lambda = lambda x: x

    type a_type_alias = int
    type a_type_alias_pep_695[T] = list[T]

    class a_class: pass
    class a_class_pep_695[T]: pass

    def a_method(self): pass
    def a_method_pep_695[T](self): pass

    async def an_async_method(self): pass
    async def an_async_method_pep_695[T](self): pass

    @classmethod
    def a_classmethod(cls): pass
    @classmethod
    def a_classmethod_pep_695[T](self): pass

    @classmethod
    async def an_async_classmethod(cls): pass
    @classmethod
    async def an_async_classmethod_pep_695[T](self): pass

    @staticmethod
    def a_staticmethod(): pass
    @staticmethod
    def a_staticmethod_pep_695[T](self): pass

    @staticmethod
    async def an_async_staticmethod(): pass
    @staticmethod
    async def an_async_staticmethod_pep_695[T](self): pass

    # These ones will be considered as methods because of the 'def' although
    # they are *not* valid methods at runtime since they are not decorated
    # with @staticmethod.
    def a_fakemethod(): pass
    def a_fakemethod_pep_695[T](): pass

    async def an_async_fakemethod(): pass
    async def an_async_fakemethod_pep_695[T](): pass

    # Check that those are still considered as methods
    # since they are not using the 'global' keyword.
    def glob_unassigned_meth(): pass
    def glob_unassigned_meth_pep_695[T](): pass

    async def glob_unassigned_async_meth(): pass
    async def glob_unassigned_async_meth_pep_695[T](): pass

    def glob_assigned_meth(): pass
    def glob_assigned_meth_pep_695[T](): pass

    async def glob_assigned_async_meth(): pass
    async def glob_assigned_async_meth_pep_695[T](): pass

    # The following are not picked as local symbols because they are not
    # visible by the class at runtime (this is equivalent to having the
    # definitions outside of the class).
    global glob_unassigned_meth_ignore
    def glob_unassigned_meth_ignore(): pass
    global glob_unassigned_meth_pep_695_ignore
    def glob_unassigned_meth_pep_695_ignore[T](): pass

    global glob_unassigned_async_meth_ignore
    async def glob_unassigned_async_meth_ignore(): pass
    global glob_unassigned_async_meth_pep_695_ignore
    async def glob_unassigned_async_meth_pep_695_ignore[T](): pass

    global glob_assigned_meth_ignore
    def glob_assigned_meth_ignore(): pass
    global glob_assigned_meth_pep_695_ignore
    def glob_assigned_meth_pep_695_ignore[T](): pass

    global glob_assigned_async_meth_ignore
    async def glob_assigned_async_meth_ignore(): pass
    global glob_assigned_async_meth_pep_695_ignore
    async def glob_assigned_async_meth_pep_695_ignore[T](): pass
"""


def find_block(block, name):
    for child in block.get_children():
        if child.get_name() == name:
            return child
    raise AssertionError(f"missing child block {name!r}")


top = symtable.symtable(TEST_CODE, "?", "exec")
Mine = find_block(top, "Mine")
assert Mine.get_methods() == ("a_method",)

top = symtable.symtable(TEST_COMPLEX_CLASS_CODE, "?", "exec")
this = find_block(top, "ComplexClass")
assert this.get_methods() == (
    "a_method",
    "a_method_pep_695",
    "an_async_method",
    "an_async_method_pep_695",
    "a_classmethod",
    "a_classmethod_pep_695",
    "an_async_classmethod",
    "an_async_classmethod_pep_695",
    "a_staticmethod",
    "a_staticmethod_pep_695",
    "an_async_staticmethod",
    "an_async_staticmethod_pep_695",
    "a_fakemethod",
    "a_fakemethod_pep_695",
    "an_async_fakemethod",
    "an_async_fakemethod_pep_695",
    "glob_unassigned_meth",
    "glob_unassigned_meth_pep_695",
    "glob_unassigned_async_meth",
    "glob_unassigned_async_meth_pep_695",
    "glob_assigned_meth",
    "glob_assigned_meth_pep_695",
    "glob_assigned_async_meth",
    "glob_assigned_async_meth_pep_695",
)


def check_body(body, expected_methods):
    indented = textwrap.indent(body, " " * 4)
    top = symtable.symtable(f"class A:\n{indented}", "?", "exec")
    this = find_block(top, "A")
    assert this.get_methods() == expected_methods


genexprs = (
    "x = (x for x in [])",
    "x = (x async for x in [])",
    "genexpr = (x for x in [])",
    "genexpr = (x async for x in [])",
)

for gen in genexprs:
    check_body(gen, ())
    check_body("\n".join((gen, "genexpr = 1")), ())
    check_body("\n".join(("genexpr = 1", gen)), ())

for paramlist in ("()", "(x)", "(x, y)", "(z: T)"):
    for func in (
        f"def genexpr{paramlist}:pass",
        f"async def genexpr{paramlist}:pass",
        f"def genexpr[T]{paramlist}:pass",
        f"async def genexpr[T]{paramlist}:pass",
    ):
        check_body(func, ("genexpr",))
        for gen in genexprs:
            check_body("\n".join((gen, func)), ("genexpr",))
            check_body("\n".join((func, gen)), ("genexpr",))

print("SymtableTest::test_class_get_methods: ok")
