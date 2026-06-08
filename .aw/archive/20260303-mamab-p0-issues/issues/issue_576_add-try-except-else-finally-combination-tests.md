---
number: 576
title: "Add try/except/else/finally combination tests"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #576 — Add try/except/else/finally combination tests

## Context
The try statement has many valid combinations and nesting patterns.

## Test cases
```python
# Basic forms
try:
    pass
except:
    pass

try:
    pass
except Exception:
    pass

try:
    pass
except Exception as e:
    pass

try:
    pass
except (TypeError, ValueError):
    pass

try:
    pass
except (TypeError, ValueError) as e:
    pass

# Multiple except clauses
try:
    pass
except TypeError:
    pass
except ValueError:
    pass
except Exception:
    pass

# With else
try:
    pass
except Exception:
    pass
else:
    pass

# With finally
try:
    pass
finally:
    pass

try:
    pass
except Exception:
    pass
finally:
    pass

# Full form
try:
    pass
except TypeError:
    pass
except ValueError:
    pass
else:
    pass
finally:
    pass

# except* (PEP 654)
try:
    pass
except* TypeError:
    pass
except* ValueError:
    pass

try:
    pass
except* (TypeError, ValueError) as eg:
    pass

# Nested try
try:
    try:
        pass
    except Inner:
        pass
except Outer:
    pass

# Try in various scopes
def f():
    try:
        return 1
    except:
        return 2
    finally:
        cleanup()

class C:
    try:
        import optional
    except ImportError:
        optional = None

# Raise in except
try:
    pass
except Exception as e:
    raise RuntimeError("wrapped") from e

# Bare raise
try:
    pass
except:
    raise

# Try with complex body
try:
    with open('f') as fh:
        for line in fh:
            if condition:
                break
except (IOError, OSError) as e:
    pass
```

## Task
Create `tests/fixtures/parse/edge_cases/try_combinations.py` with `# RUN: parse`.
