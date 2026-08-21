# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_refactor"
# dimension = "type"
# case = "get_fixers_from_package__pkg_name_as_str_wrong"
# subject = "lib2to3.refactor.get_fixers_from_package(pkg_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/refactor.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.refactor.get_fixers_from_package(pkg_name: str); call it with the wrong type.

typeshed contract: pkg_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.refactor import get_fixers_from_package
try:
    get_fixers_from_package(12345)  # pkg_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
