# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "behavior"
# case = "module_state_tests__test_reload_module"
# subject = "cpython.test_ast.ModuleStateTests.test_reload_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ast/test_ast.py"
# status = "filled"
# ///
import gc
import sys

saved = sys.modules.get("_ast")
try:
    sys.modules.pop("_ast", None)
    import _ast as ast1

    sys.modules.pop("_ast", None)
    import _ast as ast2
finally:
    if saved is not None:
        sys.modules["_ast"] = saved

del ast1
del ast2
gc.collect()

print("ModuleStateTests::test_reload_module: ok")
