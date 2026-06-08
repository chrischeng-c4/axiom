# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# exceptions catch patterns broad

# basic catch
try:
    raise ValueError("bad")
except ValueError as e:
    print("caught:", e)

# catch by base class
try:
    raise ValueError("oops")
except Exception as e:
    print("as exception:", e)

# multiple except blocks
def run(x):
    try:
        if x == 1:
            raise ValueError("v")
        elif x == 2:
            raise TypeError("t")
        elif x == 3:
            raise KeyError("k")
        return "ok"
    except ValueError as e:
        return "got value: " + str(e)
    except TypeError as e:
        return "got type: " + str(e)
    except KeyError as e:
        return "got key: " + str(e.args[0])

print(run(0))
print(run(1))
print(run(2))
print(run(3))

# bare except
def bare(x):
    try:
        if x < 0:
            raise RuntimeError("neg")
        return x * 2
    except:
        return -1

print(bare(5))
print(bare(-1))

# finally always runs
def with_finally(x):
    log = []
    try:
        log.append("try")
        if x < 0:
            raise ValueError("neg")
        log.append("after-check")
    except ValueError:
        log.append("caught")
    finally:
        log.append("finally")
    return log

print(with_finally(5))
print(with_finally(-1))

# nested try
def nested(x):
    try:
        try:
            if x == 0:
                raise ValueError("inner")
            return x
        except ValueError:
            raise TypeError("rethrown")
    except TypeError as e:
        return str(e)

print(nested(5))
print(nested(0))

# catch tuple
def multi(x):
    try:
        if x == 1:
            raise ValueError("v")
        if x == 2:
            raise KeyError("k")
        return "none"
    except (ValueError, KeyError) as e:
        return "multi: " + str(e.args[0])

print(multi(0))
print(multi(1))
print(multi(2))