# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filterwarnings_filters_by_category"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filterwarnings("ignore", category=DeprecationWarning) drops DeprecationWarning while a concurrent UserWarning is still captured"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.filterwarnings("ignore", category=DeprecationWarning)
    warnings.warn("user", UserWarning)
    warnings.warn("deprecated", DeprecationWarning)
    cats = [w.category for w in recorded]
    assert UserWarning in cats, f"UserWarning captured: {cats!r}"
    assert DeprecationWarning not in cats, f"DeprecationWarning ignored: {cats!r}"

print("filterwarnings_filters_by_category OK")
