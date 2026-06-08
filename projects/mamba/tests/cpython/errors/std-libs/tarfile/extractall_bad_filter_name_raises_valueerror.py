# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "errors"
# case = "extractall_bad_filter_name_raises_valueerror"
# subject = "tarfile.TarFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.TarFile: extractall_bad_filter_name_raises_valueerror (errors)."""
import tarfile
import io
_buf = io.BytesIO()
tarfile.open(fileobj=_buf, mode='w').close()
_buf.seek(0)
_tf = tarfile.open(fileobj=_buf, mode='r')

_raised = False
try:
    _tf.extractall('dest', filter='nope')
except ValueError:
    _raised = True
assert _raised, "extractall_bad_filter_name_raises_valueerror: expected ValueError"
print("extractall_bad_filter_name_raises_valueerror OK")
