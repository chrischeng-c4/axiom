# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# try/finally semantics broad

# basic try/finally with exception
def f1():
    try:
        raise ValueError("x")
    except ValueError:
        print("caught")
    finally:
        print("final 1")

f1()

# try/except/else/finally - normal path (finally runs; else runs when no exception)
def f2():
    try:
        x = 1
    except Exception:
        print("except 2")
    else:
        print("else 2")
    finally:
        print("final 2")

f2()

# try/except/else/finally - exception path (no else)
def f3():
    try:
        raise RuntimeError("y")
    except RuntimeError:
        print("except 3")
    else:
        print("else 3 (NOT)")
    finally:
        print("final 3")

f3()

# nested try
def f4():
    try:
        try:
            raise ValueError("inner")
        finally:
            print("inner finally")
    except ValueError:
        print("outer caught")

f4()

# finally with return-like side effect
def f5():
    result = []
    try:
        result.append("try")
        raise Exception("boom")
    except Exception:
        result.append("except")
    finally:
        result.append("finally")
    return result

print(f5())

# multiple except types
def f6(v):
    try:
        if v == 1:
            raise ValueError
        if v == 2:
            raise TypeError
        if v == 3:
            raise RuntimeError
    except ValueError:
        return "value"
    except TypeError:
        return "type"
    except Exception:
        return "other"

print(f6(1))
print(f6(2))
print(f6(3))

# re-raise
def f7():
    try:
        try:
            raise ValueError("origin")
        except ValueError:
            print("caught once")
            raise
    except ValueError as e:
        print("caught twice:", e)

f7()