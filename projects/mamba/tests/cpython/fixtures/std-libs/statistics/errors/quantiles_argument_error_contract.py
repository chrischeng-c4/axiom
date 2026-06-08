# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "quantiles_argument_error_contract"
# subject = "statistics.quantiles"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: quantiles n<1 and one-element data are StatisticsError, a non-int n and a stray positional cut-count are TypeError, and an unknown method is a ValueError"""
from statistics import quantiles, StatisticsError

# n<1 and one-element data are StatisticsError; a non-int n and a stray
# positional cut-count are TypeError; an unknown method is a ValueError.
for bad, exc in [(lambda: quantiles([10, 20, 30], n=0), StatisticsError),
                 (lambda: quantiles([10, 20, 30], n=-1), StatisticsError),
                 (lambda: quantiles([10], n=4), StatisticsError),
                 (lambda: quantiles([10, 20, 30], n=1.5), TypeError),
                 (lambda: quantiles([10, 20, 30], 4), TypeError),
                 (lambda: quantiles([10, 20, 30], method="X"), ValueError)]:
    _raised = False
    try:
        bad()
    except exc:
        _raised = True
    assert _raised, exc.__name__

print("quantiles_argument_error_contract OK")
