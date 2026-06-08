"""Surface contract for third-party alembic package.

# type-regime: monomorphic

Probes: alembic.__version__, alembic.context, alembic.op,
alembic.EnvironmentContext, alembic.config.Config.
CPython 3.12 is the oracle.
"""

import alembic  # type: ignore[import]
import alembic.config  # type: ignore[import]
import alembic.op  # type: ignore[import]
from alembic.runtime.environment import EnvironmentContext

# Core API
assert hasattr(alembic, "__version__"), "__version__"
assert hasattr(alembic, "context"), "context"
assert hasattr(alembic, "op"), "op"

# Version
assert isinstance(alembic.__version__, str), \
    f"version type = {type(alembic.__version__)!r}"

# Config class
assert hasattr(alembic.config, "Config"), "Config"
assert callable(alembic.config.Config), "Config callable"

# Config construction
_cfg = alembic.config.Config()
assert hasattr(_cfg, "get_main_option"), "cfg.get_main_option"
assert hasattr(_cfg, "set_main_option"), "cfg.set_main_option"
assert hasattr(_cfg, "get_section"), "cfg.get_section"

# Config set/get option
_cfg.set_main_option("script_location", "migrations")
_val = _cfg.get_main_option("script_location")
assert _val == "migrations", f"config option = {_val!r}"

# op module has DDL operations
assert hasattr(alembic.op, "create_table"), "op.create_table"
assert hasattr(alembic.op, "drop_table"), "op.drop_table"
assert hasattr(alembic.op, "add_column"), "op.add_column"
assert hasattr(alembic.op, "drop_column"), "op.drop_column"
assert hasattr(alembic.op, "create_index"), "op.create_index"
assert hasattr(alembic.op, "execute"), "op.execute"

# EnvironmentContext is a class
assert callable(EnvironmentContext), "EnvironmentContext callable"

# Module attributes stable
_v_ref = alembic.__version__
assert alembic.__version__ is _v_ref, "__version__ stable"
_ctx_ref = alembic.context
assert alembic.context is _ctx_ref, "context stable"
_op_ref = alembic.op
assert alembic.op is _op_ref, "op stable"
_ec_ref = EnvironmentContext
assert EnvironmentContext is _ec_ref, "EnvironmentContext stable"

print("surface OK")
