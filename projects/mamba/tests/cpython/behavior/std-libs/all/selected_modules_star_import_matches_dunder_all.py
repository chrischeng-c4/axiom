# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "all"
# dimension = "behavior"
# case = "selected_modules_star_import_matches_dunder_all"
# subject = "cpython.test___all__.AllTest.test_all"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test___all__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Selected stdlib modules export exactly their __all__ names on star import."""
import importlib

MODULES = [
    "calendar",
    "cmd",
    "codecs",
    "collections",
    "copy",
    "dataclasses",
    "enum",
    "functools",
]

for modname in MODULES:
    module = importlib.import_module(modname)
    all_names = module.__all__
    assert len(set(all_names)) == len(all_names), modname

    namespace = {}
    exec(f"from {modname} import *", namespace)
    for helper_name in ["__builtins__", "__annotations__", "__warningregistry__"]:
        namespace.pop(helper_name, None)

    exported = set(namespace)
    expected = set(all_names)
    assert exported == expected, (modname, sorted(exported ^ expected))

print("selected_modules_star_import_matches_dunder_all OK")
