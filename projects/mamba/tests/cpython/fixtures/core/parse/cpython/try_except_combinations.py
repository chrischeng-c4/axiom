# RUN: parse
# Try/except/else/finally combination tests (#576)

# --- minimal try/except ---
try:
    pass
except:
    pass

# --- try/except with type ---
try:
    pass
except Exception:
    pass

# --- try/except with as ---
try:
    pass
except Exception as e:
    pass

# --- try/except/else ---
try:
    x = 1
except Exception:
    x = 0
else:
    y = x + 1

# --- try/except/finally ---
try:
    x = 1
except Exception:
    x = 0
finally:
    z = True

# --- try/except/else/finally ---
try:
    x = 1
except Exception:
    x = 0
else:
    y = x + 1
finally:
    z = True

# --- try/finally (no except) ---
try:
    x = 1
finally:
    pass

# --- multiple except clauses ---
try:
    x = 1
except ValueError:
    pass
except TypeError:
    pass
except (KeyError, IndexError):
    pass
except Exception as e:
    pass

# --- multiple except + else + finally ---
try:
    x = int("42")
except ValueError:
    x = 0
except TypeError:
    x = -1
else:
    y = x * 2
finally:
    done = True

# --- nested try ---
try:
    try:
        x = 1
    except ValueError:
        pass
    finally:
        pass
except Exception:
    pass
finally:
    pass

# --- try in loop ---
for i in range(5):
    try:
        x = 10 / i
    except ZeroDivisionError:
        x = 0
    else:
        y = x * 2
    finally:
        pass

# --- try in while ---
count = 0
while count < 3:
    try:
        count += 1
        if count == 2:
            raise ValueError
    except ValueError:
        continue
    else:
        pass
    finally:
        pass

# --- try with break ---
for i in range(10):
    try:
        if i == 5:
            break
    except:
        pass
    finally:
        pass

# --- try with return ---
def func_with_try():
    try:
        return 1
    except:
        return 0
    finally:
        pass

# --- try with yield ---
def gen_with_try():
    try:
        yield 1
    except:
        yield 0
    finally:
        yield -1

# --- try in with ---
class Ctx:
    def __enter__(self):
        return self
    def __exit__(self, *a):
        pass

with Ctx():
    try:
        pass
    except:
        pass

# --- try in class ---
class MyClass:
    try:
        import json
    except ImportError:
        json = None

# --- try in if ---
if True:
    try:
        pass
    except:
        pass
else:
    try:
        pass
    except:
        pass

# --- chained except with as ---
try:
    raise ValueError("test")
except ValueError as e1:
    try:
        raise TypeError("inner") from e1
    except TypeError as e2:
        pass

# --- try with raise ---
try:
    raise RuntimeError("test")
except RuntimeError:
    raise

# --- try with raise from ---
try:
    raise ValueError("original")
except ValueError as orig:
    raise RuntimeError("wrapped") from orig

# --- try with raise from None ---
try:
    raise ValueError("noisy")
except ValueError:
    raise RuntimeError("clean") from None

# --- bare except after typed excepts ---
try:
    pass
except ValueError:
    pass
except TypeError:
    pass
except:
    pass

# --- except with tuple of exception types ---
try:
    pass
except (ValueError, TypeError, KeyError, IndexError):
    pass

# --- try/except in lambda body (not possible — use wrapper) ---
def safe(f, default=None):
    try:
        return f()
    except:
        return default

# --- deeply nested try ---
try:
    try:
        try:
            try:
                raise ValueError
            except ValueError:
                raise TypeError
        except TypeError:
            raise KeyError
    except KeyError:
        raise RuntimeError
except RuntimeError:
    pass

# --- try with assert ---
try:
    assert False, "test assertion"
except AssertionError:
    pass

# --- try with complex finally ---
resource = None
try:
    resource = "acquired"
    raise ValueError
except ValueError:
    pass
finally:
    if resource is not None:
        resource = None

# --- multiple try blocks in sequence ---
try:
    a = 1
except:
    a = 0

try:
    b = 2
except:
    b = 0

try:
    c = a + b
except:
    c = -1
