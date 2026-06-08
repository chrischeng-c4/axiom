# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filterwarnings_filters_by_message_regex"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filterwarnings("ignore", message=".*skip_me.*") drops messages matching the regex while non-matching messages are kept"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.filterwarnings("ignore", message=".*skip_me.*")
    warnings.warn("skip_me warning", UserWarning)
    warnings.warn("keep_me warning", UserWarning)
    msgs = [str(w.message) for w in recorded]
    assert not any("skip_me" in m for m in msgs), f"skip_me filtered: {msgs!r}"
    assert any("keep_me" in m for m in msgs), f"keep_me kept: {msgs!r}"

print("filterwarnings_filters_by_message_regex OK")
