import cmath

_ledger: list[int] = []

assert cmath.__name__ == "cmath", "cmath.__name__"
_ledger.append(1)

assert abs(cmath.pi - 3.141592653589793) < 1e-10, "cmath.pi"
_ledger.append(1)

assert cmath.sqrt(-1) == 1j, "cmath.sqrt(-1) == 1j"
_ledger.append(1)

assert cmath.sqrt(complex(-4, 0)) == 2j, "cmath.sqrt(-4) == 2j"
_ledger.append(1)

assert cmath.exp(0j) == complex(1, 0), "cmath.exp(0j) == 1+0j"
_ledger.append(1)

_logval = cmath.log(cmath.e)
assert abs(_logval.real - 1.0) < 1e-10 and abs(_logval.imag) < 1e-10, "cmath.log(e).real approx 1"
_ledger.append(1)

_sinval = cmath.sin(0j)
assert abs(_sinval.real) < 1e-10 and abs(_sinval.imag) < 1e-10, "cmath.sin(0) approx 0"
_ledger.append(1)

_cosval = cmath.cos(0j)
assert abs(_cosval.real - 1.0) < 1e-10, "cmath.cos(0) approx 1"
_ledger.append(1)

assert cmath.isnan(complex(float('nan'), 0)) == True, "cmath.isnan on nan"
_ledger.append(1)

assert cmath.isnan(complex(0, 0)) == False, "cmath.isnan on 0+0j is False"
_ledger.append(1)

assert cmath.isinf(complex(float('inf'), 0)) == True, "cmath.isinf on inf"
_ledger.append(1)

assert cmath.isinf(complex(0, 0)) == False, "cmath.isinf on 0+0j is False"
_ledger.append(1)

_pol = cmath.polar(1+1j)
assert abs(_pol[0] - cmath.sqrt(2).real) < 1e-10, "polar magnitude of 1+1j is sqrt(2)"
_ledger.append(1)

assert abs(_pol[1] - (cmath.pi / 4)) < 1e-10, "polar angle of 1+1j is pi/4"
_ledger.append(1)

_rec = cmath.rect(1, 0)
assert abs(_rec.real - 1.0) < 1e-10 and abs(_rec.imag) < 1e-10, "rect(1, 0) == 1+0j"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: cmath {sum(_ledger)} asserts")
