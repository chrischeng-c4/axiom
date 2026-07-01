# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_upload"
# dimension = "type"
# case = "upload__upload_file__command_as_str_wrong"
# subject = "distutils.command.upload.upload.upload_file(command: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/upload.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.upload.upload.upload_file(command: str); call it with the wrong type.

typeshed contract: command is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.command.upload import upload
obj = object.__new__(upload)
try:
    obj.upload_file(12345, "", "")  # command: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
