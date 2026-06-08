# Language conformance: exception full coverage (R4.8).
# BaseException tree, except subclass matching, raise from,
# __cause__/__context__/__traceback__
# ExceptionGroup/except*: xfail (see #755)

# --- BaseException hierarchy ---
print(issubclass(Exception, BaseException))
print(issubclass(ValueError, Exception))
print(issubclass(TypeError, Exception))
print(issubclass(KeyboardInterrupt, BaseException))
print(issubclass(SystemExit, BaseException))

# --- except subclass matching (isinstance against MRO) ---
def match_exception(exc: BaseException) -> str:
    try:
        raise exc
    except ValueError:
        return "ValueError"
    except LookupError:
        return "LookupError"
    except Exception:
        return "Exception"
    except BaseException:
        return "BaseException"

print(match_exception(ValueError("v")))
print(match_exception(KeyError("k")))     # KeyError < LookupError
print(match_exception(TypeError("t")))
print(match_exception(RuntimeError("r")))

# --- raise from (explicit chaining) ---
def explicit_chain() -> None:
    try:
        raise ValueError("original")
    except ValueError as e:
        raise RuntimeError("wrapped") from e

try:
    explicit_chain()
except RuntimeError as e:
    print(f"RuntimeError: {e}")
    print(f"__cause__: {e.__cause__}")
    print(f"__suppress_context__: {e.__suppress_context__}")

# --- Implicit chaining (__context__) ---
def implicit_chain() -> None:
    try:
        raise ValueError("first")
    except ValueError:
        raise RuntimeError("second")

try:
    implicit_chain()
except RuntimeError as e:
    print(f"RuntimeError: {e}")
    print(f"__context__ type: {type(e.__context__).__name__}")
    print(f"__suppress_context__: {e.__suppress_context__}")

# --- raise from None (suppress context) ---
def suppressed_chain() -> None:
    try:
        raise ValueError("hidden")
    except ValueError:
        raise RuntimeError("clean") from None

try:
    suppressed_chain()
except RuntimeError as e:
    print(f"RuntimeError: {e}")
    print(f"__cause__: {e.__cause__}")
    print(f"__suppress_context__: {e.__suppress_context__}")

# --- Exception with args ---
e = TypeError("bad type", 42)
print(e.args)

# --- finally always runs ---
def with_finally(raise_exc: bool) -> str:
    try:
        if raise_exc:
            raise ValueError("err")
        return "ok"
    finally:
        print("finally ran")

print(with_finally(False))
try:
    with_finally(True)
except ValueError:
    print("caught ValueError")

# --- ExceptionGroup/except*: xfail (see #755) ---
# # mamba-xfail: ExceptionGroup not implemented (see #755)
# try:
#     raise ExceptionGroup("eg", [ValueError("v"), TypeError("t")])
# except* ValueError as eg:
#     print("caught ValueError group")
