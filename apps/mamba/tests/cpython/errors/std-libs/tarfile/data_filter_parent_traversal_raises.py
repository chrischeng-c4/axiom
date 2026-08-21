# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "data_filter_parent_traversal_raises"
# subject = "tarfile.data_filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: data_filter_parent_traversal_raises (errors)."""
import tarfile
_esc = tarfile.TarInfo('../escape.txt')
_esc.size = 0

_raised = False
try:
    tarfile.data_filter(_esc, 'dest')
except tarfile.OutsideDestinationError:
    _raised = True
assert _raised, "data_filter_parent_traversal_raises: expected tarfile.OutsideDestinationError"
print("data_filter_parent_traversal_raises OK")
