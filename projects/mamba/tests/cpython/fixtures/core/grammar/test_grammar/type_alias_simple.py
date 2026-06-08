# RUN: parse
# CPython 3.12 test_grammar: simple type alias (PEP 695)

# Simple type alias without type parameters
type Vector = list[float]
type Str = str
type Number = int | float
