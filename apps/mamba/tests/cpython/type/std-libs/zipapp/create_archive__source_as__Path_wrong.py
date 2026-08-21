# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "type"
# case = "create_archive__source_as__Path_wrong"
# subject = "zipapp.create_archive(source: _Path)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipapp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipapp.create_archive(source: _Path); call it with the wrong type.

typeshed contract: source is _Path. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipapp import create_archive
try:
    create_archive(_W())  # source: _Path <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
