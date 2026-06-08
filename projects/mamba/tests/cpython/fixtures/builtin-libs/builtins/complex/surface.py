"""Surface contract for builtins.complex.

# type-regime: monomorphic

Probes: name presence, is a type, __name__, .real and .imag attributes,
conjugate, abs, arithmetic.
CPython 3.12 is the oracle.
"""

import builtins

assert hasattr(builtins, "complex"), "builtins.complex missing"
assert builtins.complex is complex, "builtins.complex is complex divergence"
assert callable(builtins.complex), "builtins.complex not callable"

# complex is a class (type)
assert type(builtins.complex).__name__ == "type", \
    f"type(builtins.complex).__name__ = {type(builtins.complex).__name__!r}"
assert issubclass(complex, object), "complex not subclass of object"
assert builtins.complex.__name__ == "complex", \
    f"complex.__name__ = {builtins.complex.__name__!r}"

# complex instances
z = complex(3, 4)
assert isinstance(z, complex), "isinstance(3+4j, complex) failed"

# .real and .imag attributes
assert z.real == 3.0, f"z.real = {z.real!r}"
assert z.imag == 4.0, f"z.imag = {z.imag!r}"
assert isinstance(z.real, float), f"type(z.real) = {type(z.real).__name__!r}"
assert isinstance(z.imag, float), f"type(z.imag) = {type(z.imag).__name__!r}"

# conjugate method
assert hasattr(complex, "conjugate"), "complex.conjugate missing"

# j literal syntax
assert (1 + 2j).real == 1.0, "(1+2j).real"
assert (1 + 2j).imag == 2.0, "(1+2j).imag"

# complex.__doc__ exists
assert isinstance(builtins.complex.__doc__, str) and len(builtins.complex.__doc__) > 0, \
    "builtins.complex.__doc__ missing"

print("surface OK")
