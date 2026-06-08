# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "real_world"
# case = "di_container_dispatch_by_signature"
# subject = "inspect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: a dependency-injection container introspects callables with isclass/isfunction/signature, reads parameter defaults and kinds, and binds a call via Signature.bind + apply_defaults to wire up a small object graph"""
import inspect

# A tiny dependency-injection container: it inspects each provider callable,
# decides how to construct it, reads parameter defaults/kinds, and binds the
# call via Signature.bind + apply_defaults to wire up a small object graph.

class Database:
    def __init__(self, dsn="sqlite://memory"):
        self.dsn = dsn

def make_logger(level="info"):
    return {"level": level}

class Service:
    def __init__(self, db, logger, retries=3):
        self.db = db
        self.logger = logger
        self.retries = retries

def construct(provider, **overrides):
    # Dispatch on the kind of callable, exactly as a DI container would.
    if inspect.isclass(provider):
        target = provider.__init__
        skip_self = True
    elif inspect.isfunction(provider):
        target = provider
        skip_self = False
    else:
        raise TypeError("unsupported provider")

    sig = inspect.signature(target)
    params = list(sig.parameters.values())
    if skip_self:
        params = params[1:]  # drop self

    # Resolve each parameter: an override wins, else its declared default.
    kwargs = {}
    for p in params:
        if p.name in overrides:
            kwargs[p.name] = overrides[p.name]
        elif p.default is not inspect.Parameter.empty:
            kwargs[p.name] = p.default
        else:
            raise LookupError(f"cannot resolve required param {p.name!r}")
    return provider(**kwargs)

# Build the graph: Database and logger come from their declared defaults.
db = construct(Database)
assert db.dsn == "sqlite://memory", db.dsn
logger = construct(make_logger, level="debug")
assert logger == {"level": "debug"}, logger

# Service has two required params (db, logger) plus a defaulted one (retries).
svc_sig = inspect.signature(Service.__init__)
svc_params = list(svc_sig.parameters.values())[1:]  # drop self
assert [p.name for p in svc_params] == ["db", "logger", "retries"], svc_params
assert svc_params[0].kind == inspect.Parameter.POSITIONAL_OR_KEYWORD
assert svc_params[2].default == 3, svc_params[2].default

service = construct(Service, db=db, logger=logger)
assert service.db is db and service.logger is logger, "wired dependencies"
assert service.retries == 3, "default retries applied"

# Bind a method call through the signature and apply defaults.
def handle(request, *, verbose=False, attempts=1):
    return (request, verbose, attempts)

hsig = inspect.signature(handle)
bound = hsig.bind("ping", verbose=True)
bound.apply_defaults()
assert bound.arguments == {"request": "ping", "verbose": True, "attempts": 1}, bound.arguments
assert handle(*bound.args, **bound.kwargs) == ("ping", True, 1), "dispatched call"

print("di_container_dispatch_by_signature OK")
