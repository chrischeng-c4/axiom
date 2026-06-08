# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# #1691: zero-arg scalar constructors emit the CPython-default value
# without invoking the runtime's 1-arg dispatch (which would trip the
# Cranelift verifier with a 0-arg call against a 1-arg signature).
print(bool())
print(int())
print(float())
print(repr(str()))
print(type(bool()).__name__)
print(type(int()).__name__)
print(type(float()).__name__)
print(type(str()).__name__)
