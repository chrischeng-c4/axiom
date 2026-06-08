# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "data_filter_strips_leading_slash"
# subject = "tarfile.data_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.data_filter: data_filter strips a leading absolute slash ('/etc/evil.txt' -> 'etc/evil.txt') rather than letting the path escape to root"""
import tarfile

_abs = tarfile.TarInfo("/etc/evil.txt")
_abs.size = 0
_stripped = tarfile.data_filter(_abs, "dest")
assert _stripped.name == "etc/evil.txt", f"abs strip = {_stripped.name!r}"

print("data_filter_strips_leading_slash OK")
