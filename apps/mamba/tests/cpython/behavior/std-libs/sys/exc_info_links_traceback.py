# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "exc_info_links_traceback"
# subject = "sys.exc_info"
# kind = "semantic"
# xfail = "mamba e.__traceback__ is None / exc_info linkage incomplete (repo-memory project_mamba_module_exec_del_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: exc_info() returns (ValueError, the caught instance, the instance's __traceback__) for a live exception"""
import sys


def _raise():
    raise ValueError(42)


try:
    _raise()
except ValueError as _e:
    _t, _v, _tb = sys.exc_info()
    assert _t is ValueError, f"exc_info[0] = {_t!r}"
    assert _v is _e, "exc_info[1] is the caught instance"
    assert _tb is _e.__traceback__, "exc_info[2] is e.__traceback__"
print("exc_info_links_traceback OK")
