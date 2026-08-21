"""Behavior contract for third-party uvicorn package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import uvicorn  # type: ignore[import]

# Rule 1: Config accepts host/port and stores them
_cfg1 = uvicorn.Config(app=lambda: None, host="0.0.0.0", port=9000)
assert _cfg1.host == "0.0.0.0", f"host = {_cfg1.host!r}"
assert _cfg1.port == 9000, f"port = {_cfg1.port!r}"

# Rule 2: Config default values
_cfg2 = uvicorn.Config(app=lambda: None)
assert _cfg2.host == "127.0.0.1", f"default host = {_cfg2.host!r}"
assert _cfg2.port == 8000, f"default port = {_cfg2.port!r}"
assert _cfg2.reload is False, f"default reload = {_cfg2.reload!r}"
assert isinstance(_cfg2.workers, int), f"workers is int: {_cfg2.workers!r}"

# Rule 3: Config log_level default
_cfg3 = uvicorn.Config(app=lambda: None)
# log_level default is None (gets resolved later during setup)
assert _cfg3.log_level is None or isinstance(_cfg3.log_level, str), \
    f"log_level default = {_cfg3.log_level!r}"

# Rule 4: Config accepts log_level
_cfg4 = uvicorn.Config(app=lambda: None, log_level="debug")
assert _cfg4.log_level == "debug", f"log_level = {_cfg4.log_level!r}"

# Rule 5: Server wraps Config; started/should_exit are False initially
_cfg5 = uvicorn.Config(app=lambda: None)
_server5 = uvicorn.Server(_cfg5)
assert _server5.config is _cfg5, "server.config is cfg"
assert _server5.started is False, f"started = {_server5.started!r}"
assert _server5.should_exit is False, f"should_exit = {_server5.should_exit!r}"

# Rule 6: Config.bind_socket / ssl / workers attributes exist
assert hasattr(_cfg5, "bind_socket") or hasattr(_cfg5, "fd") or hasattr(_cfg5, "uds"), \
    "config has bind info"
assert hasattr(_cfg5, "ssl_certfile"), "config.ssl_certfile"
assert hasattr(_cfg5, "workers"), "config.workers"

# Rule 7: Module attributes are identity-stable
_run_ref = uvicorn.run
_config_ref = uvicorn.Config
_server_ref = uvicorn.Server
for _ in range(5):
    assert uvicorn.run is _run_ref, "run stable"
    assert uvicorn.Config is _config_ref, "Config stable"
    assert uvicorn.Server is _server_ref, "Server stable"

print("behavior OK")
