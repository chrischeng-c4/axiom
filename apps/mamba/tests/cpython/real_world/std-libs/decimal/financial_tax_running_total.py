# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "real_world"
# case = "financial_tax_running_total"
# subject = "decimal.Decimal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_decimal.py"
# status = "filled"
# ///
"""decimal.Decimal: a finance pipeline sums Decimal line items into a pre-tax subtotal, applies a 6% tax via Decimal multiply, and confirms the exact stringified subtotal/tax/total (48.49 / 2.9094 / 51.3994) with no binary-float drift"""
from decimal import Decimal

D = Decimal
# Tax rate: 6% (sales tax). Line items modelled as Decimal so the cumulative
# add/mul does not drift into binary-float error.
TAX = D("0.06")
LINE_ITEMS = [D(p) for p in ["12.50", "3.99", "1.25", "8.75", "22.00"]]

# Pre-tax total.
subtotal = D("0")
for price in LINE_ITEMS:
    subtotal = subtotal + price

# Apply tax: total = subtotal + subtotal * tax.
tax_amount = subtotal * TAX
total = subtotal + tax_amount

# 12.50 + 3.99 + 1.25 + 8.75 + 22.00 = 48.49; 48.49 * 0.06 = 2.9094;
# 48.49 + 2.9094 = 51.3994. Decimal preserves operand scale exactly.
assert str(subtotal) == "48.49", f"subtotal: got {str(subtotal)!r}, want 48.49"
assert str(tax_amount) == "2.9094", f"tax: got {str(tax_amount)!r}, want 2.9094"
assert str(total) == "51.3994", f"total: got {str(total)!r}, want 51.3994"

print("financial_tax_running_total OK")
