# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "type"
# case = "TarInfo__create_pax_global_header__pax_headers_as_Mapping_wrong"
# subject = "tarfile.TarInfo.create_pax_global_header(pax_headers: Mapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pax_headers"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tarfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pax_headers
# mamba-strict-type: TypeError
"""Type wall: tarfile.TarInfo.create_pax_global_header(pax_headers: Mapping); call it with the wrong type.

typeshed contract: pax_headers is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tarfile import TarInfo
try:
    TarInfo.create_pax_global_header(_W())  # pax_headers: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
