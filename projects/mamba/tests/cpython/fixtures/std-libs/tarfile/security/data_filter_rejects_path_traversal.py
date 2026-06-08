# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "security"
# case = "data_filter_rejects_path_traversal"
# subject = "tarfile.data_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: an untrusted member whose name traverses out of the destination ('../escape.txt') is refused by data_filter with OutsideDestinationError, and the exception subclasses FilterError subclasses TarError"""
import tarfile

# data_filter refuses a parent-traversal member (path escapes destination).
_esc = tarfile.TarInfo("../escape.txt")
_esc.size = 0
_raised = False
try:
    tarfile.data_filter(_esc, "dest")
except tarfile.OutsideDestinationError:
    _raised = True
assert _raised, "data_filter must reject .. traversal"

# OutsideDestinationError is a FilterError is a TarError.
assert issubclass(tarfile.OutsideDestinationError, tarfile.FilterError), "OutsideDestinationError < FilterError"
assert issubclass(tarfile.FilterError, tarfile.TarError), "FilterError < TarError"

print("data_filter_rejects_path_traversal OK")
