# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "mkdtemp__suffix_as_typed_wrong"
# subject = "tempfile.mkdtemp(suffix: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed suffix"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed suffix
# mamba-strict-type: TypeError
"""Type wall: tempfile.mkdtemp(suffix: typed); call it with the wrong type.

typeshed contract: suffix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import mkdtemp
try:
    mkdtemp(_W())  # suffix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
