# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "type"
# case = "ftpcp__source_as_FTP_wrong"
# subject = "ftplib.ftpcp(source: FTP)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ftplib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ftplib.ftpcp(source: FTP); call it with the wrong type.

typeshed contract: source is FTP. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ftplib import ftpcp
try:
    ftpcp(_W(), "", None)  # source: FTP <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
