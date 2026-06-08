# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "symtable"
# dimension = "behavior"
# case = "symtable_test__test_bytes"
# subject = "cpython.test_symtable.SymtableTest.test_bytes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_symtable.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_symtable.py::SymtableTest::test_bytes
"""Auto-ported test: SymtableTest::test_bytes (CPython 3.12 oracle)."""


import textwrap
import symtable
import unittest


'\nTest the API of the symtable module.\n'

TEST_CODE = '\nimport sys\n\nglob = 42\nsome_var = 12\nsome_non_assigned_global_var: int\nsome_assigned_global_var = 11\n\nclass Mine:\n    instance_var = 24\n    def a_method(p1, p2):\n        pass\n\ndef spam(a, b, *var, **kw):\n    global bar\n    global some_assigned_global_var\n    some_assigned_global_var = 12\n    bar = 47\n    some_var = 10\n    x = 23\n    glob\n    def internal():\n        return x\n    def other_internal():\n        nonlocal some_var\n        some_var = 3\n        return some_var\n    return internal\n\ndef foo():\n    pass\n\ndef namespace_test(): pass\ndef namespace_test(): pass\n\ntype Alias = int\ntype GenericAlias[T] = list[T]\n\ndef generic_spam[T](a):\n    pass\n\nclass GenericMine[T: int]:\n    pass\n'

TEST_COMPLEX_CLASS_CODE = "\n# The following symbols are defined in ComplexClass\n# without being introduced by a 'global' statement.\nglob_unassigned_meth: Any\nglob_unassigned_meth_pep_695: Any\n\nglob_unassigned_async_meth: Any\nglob_unassigned_async_meth_pep_695: Any\n\ndef glob_assigned_meth(): pass\ndef glob_assigned_meth_pep_695[T](): pass\n\nasync def glob_assigned_async_meth(): pass\nasync def glob_assigned_async_meth_pep_695[T](): pass\n\n# The following symbols are defined in ComplexClass after\n# being introduced by a 'global' statement (and therefore\n# are not considered as local symbols of ComplexClass).\nglob_unassigned_meth_ignore: Any\nglob_unassigned_meth_pep_695_ignore: Any\n\nglob_unassigned_async_meth_ignore: Any\nglob_unassigned_async_meth_pep_695_ignore: Any\n\ndef glob_assigned_meth_ignore(): pass\ndef glob_assigned_meth_pep_695_ignore[T](): pass\n\nasync def glob_assigned_async_meth_ignore(): pass\nasync def glob_assigned_async_meth_pep_695_ignore[T](): pass\n\nclass ComplexClass:\n    a_var = 1234\n    a_genexpr = (x for x in [])\n    a_lambda = lambda x: x\n\n    type a_type_alias = int\n    type a_type_alias_pep_695[T] = list[T]\n\n    class a_class: pass\n    class a_class_pep_695[T]: pass\n\n    def a_method(self): pass\n    def a_method_pep_695[T](self): pass\n\n    async def an_async_method(self): pass\n    async def an_async_method_pep_695[T](self): pass\n\n    @classmethod\n    def a_classmethod(cls): pass\n    @classmethod\n    def a_classmethod_pep_695[T](self): pass\n\n    @classmethod\n    async def an_async_classmethod(cls): pass\n    @classmethod\n    async def an_async_classmethod_pep_695[T](self): pass\n\n    @staticmethod\n    def a_staticmethod(): pass\n    @staticmethod\n    def a_staticmethod_pep_695[T](self): pass\n\n    @staticmethod\n    async def an_async_staticmethod(): pass\n    @staticmethod\n    async def an_async_staticmethod_pep_695[T](self): pass\n\n    # These ones will be considered as methods because of the 'def' although\n    # they are *not* valid methods at runtime since they are not decorated\n    # with @staticmethod.\n    def a_fakemethod(): pass\n    def a_fakemethod_pep_695[T](): pass\n\n    async def an_async_fakemethod(): pass\n    async def an_async_fakemethod_pep_695[T](): pass\n\n    # Check that those are still considered as methods\n    # since they are not using the 'global' keyword.\n    def glob_unassigned_meth(): pass\n    def glob_unassigned_meth_pep_695[T](): pass\n\n    async def glob_unassigned_async_meth(): pass\n    async def glob_unassigned_async_meth_pep_695[T](): pass\n\n    def glob_assigned_meth(): pass\n    def glob_assigned_meth_pep_695[T](): pass\n\n    async def glob_assigned_async_meth(): pass\n    async def glob_assigned_async_meth_pep_695[T](): pass\n\n    # The following are not picked as local symbols because they are not\n    # visible by the class at runtime (this is equivalent to having the\n    # definitions outside of the class).\n    global glob_unassigned_meth_ignore\n    def glob_unassigned_meth_ignore(): pass\n    global glob_unassigned_meth_pep_695_ignore\n    def glob_unassigned_meth_pep_695_ignore[T](): pass\n\n    global glob_unassigned_async_meth_ignore\n    async def glob_unassigned_async_meth_ignore(): pass\n    global glob_unassigned_async_meth_pep_695_ignore\n    async def glob_unassigned_async_meth_pep_695_ignore[T](): pass\n\n    global glob_assigned_meth_ignore\n    def glob_assigned_meth_ignore(): pass\n    global glob_assigned_meth_pep_695_ignore\n    def glob_assigned_meth_pep_695_ignore[T](): pass\n\n    global glob_assigned_async_meth_ignore\n    async def glob_assigned_async_meth_ignore(): pass\n    global glob_assigned_async_meth_pep_695_ignore\n    async def glob_assigned_async_meth_pep_695_ignore[T](): pass\n"

def find_block(block, name):
    for ch in block.get_children():
        if ch.get_name() == name:
            return ch


# --- test body ---
top = symtable.symtable(TEST_CODE, '?', 'exec')
Mine = find_block(top, 'Mine')
a_method = find_block(Mine, 'a_method')
spam = find_block(top, 'spam')
internal = find_block(spam, 'internal')
other_internal = find_block(spam, 'other_internal')
foo = find_block(top, 'foo')
Alias = find_block(top, 'Alias')
GenericAlias = find_block(top, 'GenericAlias')
GenericAlias_inner = find_block(GenericAlias, 'GenericAlias')
generic_spam = find_block(top, 'generic_spam')
generic_spam_inner = find_block(generic_spam, 'generic_spam')
GenericMine = find_block(top, 'GenericMine')
GenericMine_inner = find_block(GenericMine, 'GenericMine')
T = find_block(GenericMine, 'T')
top = symtable.symtable(TEST_CODE.encode('utf8'), '?', 'exec')

assert find_block(top, 'Mine') is not None
code = b'# -*- coding: iso8859-15 -*-\nclass \xb4: pass\n'
top = symtable.symtable(code, '?', 'exec')

assert find_block(top, 'Ž') is not None
print("SymtableTest::test_bytes: ok")
