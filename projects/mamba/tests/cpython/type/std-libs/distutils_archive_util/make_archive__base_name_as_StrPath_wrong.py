# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_archive_util"
# dimension = "type"
# case = "make_archive__base_name_as_StrPath_wrong"
# subject = "distutils.archive_util.make_archive(base_name: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/archive_util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.archive_util.make_archive(base_name: StrPath); call it with the wrong type.

typeshed contract: base_name is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.archive_util import make_archive
try:
    make_archive(_W(), "", None)  # base_name: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
