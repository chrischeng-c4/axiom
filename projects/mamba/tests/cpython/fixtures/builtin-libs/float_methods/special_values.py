# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
import math
print(math.isinf(float("inf")))
print(math.isinf(float("-inf")))
print(math.isnan(float("nan")))
print(float("inf") > 1000000)
print(float("-inf") < -1000000)
print(float("nan") == float("nan"))
