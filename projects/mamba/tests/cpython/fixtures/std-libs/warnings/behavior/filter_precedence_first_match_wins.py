# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "filter_precedence_first_match_wins"
# subject = "warnings.filterwarnings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filters are an ordered list; append=False prepends so a front "ignore" wins over a later "error", suppressing the warning"""
import warnings

with warnings.catch_warnings():
    warnings.resetwarnings()
    warnings.filterwarnings("error", append=True)
    warnings.filterwarnings("ignore", append=False)
    assert warnings.filters[0][0] == "ignore", f"front = {warnings.filters[0][0]!r}"
    # The prepended "ignore" takes precedence, so warn() is suppressed.
    with warnings.catch_warnings(record=True) as recorded:
        # Re-install the same ordering inside the inner snapshot.
        warnings.resetwarnings()
        warnings.filterwarnings("error", append=True)
        warnings.filterwarnings("ignore", append=False)
        warnings.warn("masked by ignore", UserWarning)
        assert len(recorded) == 0, f"ignore wins: {len(recorded)!r}"

print("filter_precedence_first_match_wins OK")
