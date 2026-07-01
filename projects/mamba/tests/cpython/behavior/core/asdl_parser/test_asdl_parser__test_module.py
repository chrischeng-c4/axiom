# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "asdl_parser"
# dimension = "behavior"
# case = "test_asdl_parser__test_module"
# subject = "cpython.test_asdl_parser.TestAsdlParser.test_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asdl_parser.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_asdl_parser.py::TestAsdlParser::test_module
"""Auto-ported test: TestAsdlParser::test_module (CPython 3.12 oracle)."""


import importlib.machinery
import importlib.util
import os
import sys
import sysconfig


if not sysconfig.is_python_build():
    print("TestAsdlParser::test_module: skipped, installed Python build")
    raise SystemExit(0)

src_base = os.path.dirname(os.path.dirname(os.path.dirname(__file__)))
parser_dir = os.path.join(src_base, "Parser")

sys.path.insert(0, parser_dir)
try:
    loader = importlib.machinery.SourceFileLoader("asdl", os.path.join(parser_dir, "asdl.py"))
    spec = importlib.util.spec_from_loader("asdl", loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    parsed = module.parse(os.path.join(parser_dir, "Python.asdl"))
    assert module.check(parsed), "Module validation failed"
finally:
    del sys.path[0]

assert parsed.name == "Python", parsed.name
assert "stmt" in parsed.types, parsed.types.keys()
assert "expr" in parsed.types, parsed.types.keys()
assert "mod" in parsed.types, parsed.types.keys()

print("TestAsdlParser::test_module: ok")
