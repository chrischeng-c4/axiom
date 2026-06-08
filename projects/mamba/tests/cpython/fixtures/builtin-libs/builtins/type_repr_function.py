# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# type() / repr() must recognise function-pointer values, not fall
# through to the "unknown" / empty-string default. Mamba previously
# returned `type(foo).__name__ == "unknown"` and `repr(foo) == ""` for
# any TAG_FUNC value (def-defined functions, builtin function refs).
# Address bytes vary per run, so we assert the structural shape only.

def foo(): pass

print(type(foo).__name__)              # function
print(repr(foo).startswith("<function foo at 0x")) # True
print(repr(foo).endswith(">"))           # True
print(callable(foo))                     # True
