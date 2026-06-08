# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `complex(real=0, imag=0)` — Python complex constructor.
# Was undefined-name (#1256 long-tail tracker, sub-priority 3).
# Wired through a `mb_complex(real, imag)` runtime that returns a
# heap Complex object; the lower-pass adapts the 0- and 1-arg forms
# (`complex()` → `mb_complex(0, None)`, `complex(real)` →
# `mb_complex(real, None)`).

# Zero-arg form defaults to 0+0j.
print(complex())                # 0j

# 1-arg form: imag defaults to 0.
print(complex(5))               # (5+0j)
print(complex(0))               # 0j
print(complex(-3))              # (-3+0j)

# 2-arg form: integer real + imag.
print(complex(1, 2))            # (1+2j)
print(complex(0, 1))            # 1j
print(complex(0, 0))            # 0j
print(complex(0, -1))           # -1j
print(complex(1, 0))            # (1+0j)
print(complex(-1, 2))           # (-1+2j)
print(complex(2, -3))           # (2-3j)

# Float operands round-trip with .0 stripped (matches CPython repr).
print(complex(1.5, 2.5))        # (1.5+2.5j)
print(complex(0.0, 0.5))        # 0.5j

# Bool is treated as int subtype.
print(complex(True, False))     # (1+0j)
print(complex(False, True))     # 1j

# Field access — `.real` and `.imag` are exposed as attributes (floats).
c = complex(3, 4)
print(c.real, c.imag)           # 3.0 4.0
print(complex(0, 7).imag)       # 7.0
print(complex(-2, 0).real)      # -2.0

# `repr()` matches `print` form.
print(repr(complex(1, 2)))      # (1+2j)
print(repr(complex(0, 1)))      # 1j
print(repr(complex()))          # 0j

# Container repr threads through `__repr__` per element.
print([complex(1, 2), complex(0, 1)])
# [(1+2j), 1j]
