# RUN: parse
# CPython-derived: PEP 654 exception group syntax (#560)

# --- basic except* ---
try:
    pass
except* TypeError as eg:
    pass

# --- except* with tuple of exceptions ---
try:
    pass
except* (ValueError, KeyError) as eg:
    pass

# --- multiple except* handlers ---
try:
    pass
except* TypeError as eg:
    handle_type_errors(eg)
except* ValueError as eg:
    handle_value_errors(eg)
except* KeyError as eg:
    handle_key_errors(eg)

# --- except* with complex body ---
try:
    risky_operation()
except* TypeError as eg:
    for e in eg.exceptions:
        log(e)
    count = len(eg.exceptions)
    if count > 3:
        escalate(eg)

# --- except* with tuple and complex body ---
try:
    pass
except* (OSError, IOError) as eg:
    for e in eg.exceptions:
        if isinstance(e, OSError):
            handle_os(e)
        else:
            handle_io(e)

# --- nested try/except* ---
try:
    try:
        pass
    except* ValueError as inner_eg:
        process(inner_eg)
except* TypeError as outer_eg:
    handle(outer_eg)

# --- ExceptionGroup construction ---
eg = ExceptionGroup("multiple errors", [
    ValueError("bad value"),
    TypeError("bad type"),
    KeyError("missing key"),
])

# --- nested ExceptionGroup construction ---
eg = ExceptionGroup("outer", [
    ValueError("val"),
    ExceptionGroup("inner", [
        TypeError("type1"),
        TypeError("type2"),
    ]),
])

# --- BaseExceptionGroup construction ---
beg = BaseExceptionGroup("base errors", [
    KeyboardInterrupt(),
    SystemExit(1),
])

# --- ExceptionGroup with single exception ---
eg = ExceptionGroup("single", [ValueError("only one")])

# --- except* without as clause ---
try:
    pass
except* TypeError:
    pass
except* ValueError:
    pass

# --- except* with re-raise ---
try:
    pass
except* ValueError as eg:
    raise

# --- except* with raise new ---
try:
    pass
except* ValueError as eg:
    raise RuntimeError("converted") from eg

# --- except* with raise ExceptionGroup ---
try:
    pass
except* TypeError as eg:
    raise ExceptionGroup("wrapped", eg.exceptions)

# --- try/except* with else ---
try:
    result = compute()
except* ValueError as eg:
    result = default
else:
    process(result)

# --- try/except* with finally ---
try:
    acquire_resource()
except* ValueError as eg:
    handle_error(eg)
finally:
    release_resource()

# --- try/except* with else and finally ---
try:
    result = compute()
except* ValueError as eg:
    result = fallback
except* TypeError as eg:
    result = other_fallback
else:
    log_success(result)
finally:
    cleanup()

# --- except* with multiple exception types per handler ---
try:
    pass
except* (ValueError, TypeError, ArithmeticError) as eg:
    handle_all(eg)
except* (OSError, IOError) as eg:
    handle_io(eg)

# --- except* accessing exception group attributes ---
try:
    pass
except* ValueError as eg:
    msg = eg.message
    excs = eg.exceptions
    sub = eg.subgroup(TypeError)
    derived = eg.derive(lambda excs: ExceptionGroup("new", excs))

# --- except* with conditional logic ---
try:
    pass
except* ValueError as eg:
    if len(eg.exceptions) == 1:
        handle_single(eg.exceptions[0])
    else:
        handle_multiple(eg)

# --- raise ExceptionGroup directly ---
raise ExceptionGroup("test", [ValueError("a"), TypeError("b")])

# --- except* with walrus ---
try:
    pass
except* ValueError as eg:
    if (count := len(eg.exceptions)) > 0:
        report(count)

# --- nested function with except* ---
def outer():
    def inner():
        try:
            pass
        except* ValueError as eg:
            pass
    inner()

# --- async function with except* ---
async def async_handler():
    try:
        await risky_async()
    except* ValueError as eg:
        await handle_errors(eg)
    except* TypeError as eg:
        await convert_errors(eg)

# --- except* in loop ---
for i in range(10):
    try:
        process(i)
    except* ValueError as eg:
        log(i, eg)
        continue

# --- except* with match ---
try:
    pass
except* ValueError as eg:
    match len(eg.exceptions):
        case 0:
            pass
        case 1:
            handle_one(eg.exceptions[0])
        case _:
            handle_many(eg)
