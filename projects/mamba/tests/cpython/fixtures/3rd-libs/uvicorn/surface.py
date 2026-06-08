"""Surface contract for third-party uvicorn package.

# type-regime: monomorphic

Probes: uvicorn.run, uvicorn.Config, uvicorn.Server, uvicorn.main.
CPython 3.12 is the oracle.
"""

import uvicorn
import uvicorn.config
import uvicorn.main

# Core API
assert hasattr(uvicorn, "run"), "run"
assert hasattr(uvicorn, "Config"), "Config"
assert hasattr(uvicorn, "Server"), "Server"
assert callable(uvicorn.run), "run callable"

# Config
_cfg = uvicorn.Config(app=lambda: None, host="127.0.0.1", port=8000)
assert hasattr(_cfg, "host"), "config.host"
assert hasattr(_cfg, "port"), "config.port"
assert hasattr(_cfg, "reload"), "config.reload"
assert hasattr(_cfg, "workers"), "config.workers"
assert hasattr(_cfg, "log_level"), "config.log_level"
assert _cfg.host == "127.0.0.1", f"host = {_cfg.host!r}"
assert _cfg.port == 8000, f"port = {_cfg.port!r}"

# Server
_server = uvicorn.Server(_cfg)
assert hasattr(_server, "config"), "server.config"
assert hasattr(_server, "serve"), "server.serve"
assert hasattr(_server, "startup"), "server.startup"
assert hasattr(_server, "shutdown"), "server.shutdown"
assert hasattr(_server, "started"), "server.started"
assert hasattr(_server, "should_exit"), "server.should_exit"

# Module attributes stable
_run_ref = uvicorn.run
assert uvicorn.run is _run_ref, "run stable"
_config_ref = uvicorn.Config
assert uvicorn.Config is _config_ref, "Config stable"
_server_ref = uvicorn.Server
assert uvicorn.Server is _server_ref, "Server stable"

print("surface OK")
