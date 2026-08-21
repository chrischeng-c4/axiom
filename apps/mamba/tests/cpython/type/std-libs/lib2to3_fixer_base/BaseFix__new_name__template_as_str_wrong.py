# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_fixer_base"
# dimension = "type"
# case = "BaseFix__new_name__template_as_str_wrong"
# subject = "lib2to3.fixer_base.BaseFix.new_name(template: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/fixer_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.fixer_base.BaseFix.new_name(template: str); call it with the wrong type.

typeshed contract: template is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.fixer_base import BaseFix
obj = object.__new__(BaseFix)
try:
    obj.new_name(12345)  # template: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
