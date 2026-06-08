---
change: mamba-py312-p0
group: py312-conformance
date: 2026-03-10
---

# Requirements

## Conformance Test Harness (#752)

Build a test harness that runs `.py` snippets in both CPython 3.12 and mamba, comparing stdout, stderr, exit code, and exception type. Integrate with `cargo test` (snapshot/golden-file). Support expected-failure annotations for known divergences. This is the foundation for all conformance testing.

## MbValue Arithmetic & Comparison (#753)

Verify NaN-boxed MbValue operations match CPython 3.12:
- int/float/complex arithmetic (+, -, *, /, //, %, **, unary -)
- IEEE 754 edge cases (inf, nan, -0.0)
- Mixed-type promotion (int+float, int+complex, float+complex)
- Comparison operators across types (==, !=, <, >, <=, >=)
- Truthiness (bool(0), bool(""), bool([]), bool(None), etc.)
- round(), abs(), pow(), divmod() edge cases

## Object Model (#754)

Verify mamba object model matches CPython 3.12:
- Class creation with single/multiple inheritance
- C3 MRO linearization
- Descriptor protocol (__get__, __set__, __delete__)
- Properties (@property, @x.setter, @x.deleter)
- Metaclass (class Meta(type):, __init_subclass__)
- __slots__, super() (zero-arg and explicit), __new__ vs __init__ ordering
- Attribute lookup order: instance -> class -> bases -> __getattr__

## Builtins Verification (#758)

Verify all Python builtins match CPython 3.12. 108 tests exist but need conformance verification:
- Numeric: int(), float(), complex(), round(), abs(), pow(), divmod()
- Sequence: len(), range(), sorted(), reversed(), enumerate(), zip(), map(), filter()
- String: str(), repr(), format(), chr(), ord(), ascii()
- Type: type(), isinstance(), issubclass(), callable(), hasattr(), getattr(), setattr()
- I/O: print(), input(), open()
- Other: id(), hash(), all(), any(), min(), max(), sum(), iter(), next(), exec(), eval()
