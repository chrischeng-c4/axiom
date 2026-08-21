# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "real_world"
# case = "numeric_tower_dispatch"
# subject = "numbers.Number"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_abc.py"
# status = "filled"
# ///
"""numbers.Number: a value-classifier walks the numeric tower (Integral->Real->Complex->Number) to label mixed inputs and reject non-numbers, the way a serializer or validator dispatches by abstract numeric kind"""
import numbers
from fractions import Fraction


def classify(value):
    """Label a value by its most specific numeric-tower rung.

    This mirrors how a serializer or schema validator dispatches on the
    abstract numeric kind rather than the concrete type, so a Fraction and an
    int both route through the same Rational/Integral handling.
    """
    # Most specific rung first; Integral < Rational < Real < Complex < Number.
    if isinstance(value, numbers.Integral):
        return "integral"
    if isinstance(value, numbers.Rational):
        return "rational"
    if isinstance(value, numbers.Real):
        return "real"
    if isinstance(value, numbers.Complex):
        return "complex"
    if isinstance(value, numbers.Number):
        return "number"
    return "not-a-number"


# A serializer feed of mixed inputs, including non-numeric payloads to reject.
inputs = [7, True, Fraction(3, 4), 2.5, 1 + 2j, "label", [1, 2], None]
labels = [classify(v) for v in inputs]

assert labels == [
    "integral",      # int
    "integral",      # bool subclasses int -> Integral
    "rational",      # Fraction -> Rational (not Integral)
    "real",          # float -> Real
    "complex",       # complex -> Complex
    "not-a-number",  # str
    "not-a-number",  # list
    "not-a-number",  # None
], labels

# Only the numeric inputs are accepted by a numbers.Number gate.
accepted = [v for v in inputs if isinstance(v, numbers.Number)]
assert accepted == [7, True, Fraction(3, 4), 2.5, 1 + 2j], accepted

print("numeric_tower_dispatch OK")
