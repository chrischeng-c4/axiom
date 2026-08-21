"""Behavior contract for builtins.complex.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import math

# Rule 1: complex() constructors
assert complex() == 0j, f"complex() = {complex()!r}"
assert complex(3) == (3+0j), f"complex(3) = {complex(3)!r}"
assert complex(3, 4) == (3+4j), f"complex(3,4) = {complex(3,4)!r}"
assert complex(1.5, -2.5) == (1.5-2.5j), f"complex(1.5,-2.5) = {complex(1.5,-2.5)!r}"

# Rule 2: j literal syntax
z = 3 + 4j
assert z.real == 3.0, f"z.real = {z.real!r}"
assert z.imag == 4.0, f"z.imag = {z.imag!r}"

# Rule 3: arithmetic
assert (1+2j) + (3+4j) == (4+6j), f"add = {(1+2j)+(3+4j)!r}"
assert (3+4j) - (1+2j) == (2+2j), f"sub = {(3+4j)-(1+2j)!r}"
assert (1+2j) * (3+4j) == (-5+10j), f"mul = {(1+2j)*(3+4j)!r}"
assert (1+0j) / (1+1j) == (0.5-0.5j), f"div = {(1+0j)/(1+1j)!r}"

# Rule 4: abs() returns magnitude (|z| = sqrt(re²+im²))
z = 3 + 4j
assert abs(z) == 5.0, f"|3+4j| = {abs(z)!r}"
assert abs(1j) == 1.0, f"|1j| = {abs(1j)!r}"
assert abs(complex(0, 0)) == 0.0, f"|0j| = {abs(complex(0,0))!r}"

# Rule 5: conjugate
z = 3 + 4j
c = z.conjugate()
assert c == (3-4j), f"conjugate = {c!r}"
assert c.real == 3.0 and c.imag == -4.0, "conjugate real/imag"

# Rule 6: complex(str) parses complex literals
assert complex("3+4j") == (3+4j), f"complex('3+4j') = {complex('3+4j')!r}"
assert complex("1j") == 1j, f"complex('1j') = {complex('1j')!r}"

# Rule 7: complex is not orderable
_raised = False
try:
    result = (1+2j) < (3+4j)
except TypeError:
    _raised = True
assert _raised, "(1+2j) < (3+4j) did not raise TypeError"

# Rule 8: equality
assert (1+2j) == (1+2j), "equality failed"
assert (1+2j) != (1+3j), "inequality failed"

# Rule 9: int/float + complex yields complex
assert 1 + 1j == (1+1j), f"int + complex = {1+1j!r}"
assert 1.5 + 1j == (1.5+1j), f"float + complex = {1.5+1j!r}"
assert type(1 + 1j) is complex, f"type(int+complex) = {type(1+1j).__name__!r}"

# Rule 10: complex division by zero raises ZeroDivisionError
_raised = False
try:
    result2 = (1+1j) / 0j
except ZeroDivisionError:
    _raised = True
assert _raised, "(1+1j)/0j did not raise ZeroDivisionError"

print("behavior OK")
