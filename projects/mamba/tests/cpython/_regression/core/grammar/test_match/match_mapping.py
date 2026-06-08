# RUN: parse
# CPython 3.12 test_match: mapping (dict) patterns

# Simple mapping pattern
match {"action": "move", "x": 1, "y": 2}:
    case {"action": "move", "x": x, "y": y}:
        pass

# Partial matching (extra keys allowed)
match {"name": "Alice", "age": 30, "city": "NYC"}:
    case {"name": name}:
        pass

# Rest capture with **
match {"a": 1, "b": 2, "c": 3}:
    case {"a": a, **rest}:
        pass

# Nested mapping
match {"user": {"name": "Bob", "role": "admin"}}:
    case {"user": {"name": name, "role": "admin"}}:
        pass

# Combined with literals
match {"type": "error", "code": 404}:
    case {"type": "error", "code": 404}:
        pass
    case {"type": "error", "code": code}:
        pass
    case _:
        pass
