# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""Builtin function and bound-method __qualname__ (CPython 3.12 oracle).

Builtins, static type methods, and methods bound to instances all expose
a stable dotted __qualname__. Confirms the introspection contract.
"""

import time

# Plain builtin function: bare name.
assert len.__qualname__ == "len"
assert time.time.__qualname__ == "time"

# Static / class methods on a type: "Type.method".
assert dict.fromkeys.__qualname__ == "dict.fromkeys"
assert str.maketrans.__qualname__ == "str.maketrans"
assert bytes.maketrans.__qualname__ == "bytes.maketrans"

# Methods reached through an instance still report the owning type.
assert [1, 2, 3].append.__qualname__ == "list.append"
assert {"foo": "bar"}.pop.__qualname__ == "dict.pop"

# __name__ drops the type prefix that __qualname__ keeps.
assert dict.fromkeys.__name__ == "fromkeys"
assert dict.fromkeys.__qualname__.endswith(".fromkeys")

print("builtin_qualname OK")
