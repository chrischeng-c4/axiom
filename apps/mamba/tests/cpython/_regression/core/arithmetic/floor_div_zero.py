# Arithmetic conformance: floor division by zero raises ZeroDivisionError (R2).
# Tests `//` operator with zero divisor and non-regression for normal cases.

# TC-2.1: Integer floor division by zero
try:
    x = 10 // 0
except ZeroDivisionError:
    print("caught int")

# TC-2.2: Float floor division by zero
try:
    y = 10.0 // 0.0
except ZeroDivisionError:
    print("caught float")

# TC-2.3: Normal floor division (non-regression)
print(7 // 2)

# TC-2.4: Negative floor division (non-regression)
print(-7 // 2)
