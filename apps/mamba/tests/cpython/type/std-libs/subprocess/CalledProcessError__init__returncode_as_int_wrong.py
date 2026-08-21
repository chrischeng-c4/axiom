# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "CalledProcessError__init__returncode_as_int_wrong"
# subject = "subprocess.CalledProcessError.__init__(returncode: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.CalledProcessError.__init__(returncode: int); call it with the wrong type.

typeshed contract: returncode is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from subprocess import CalledProcessError
try:
    CalledProcessError("not_an_int", None)  # returncode: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
