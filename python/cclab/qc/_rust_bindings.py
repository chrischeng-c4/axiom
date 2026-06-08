"""
Rust bindings helper module.

Provides a centralized place to import the Rust extension (_probe) with
fallback when the extension isn't available. This allows pytest to
collect tests even when the Rust extension hasn't been built.
"""

try:
    from .. import _probe as qc  # type: ignore
    TestType = qc.TestType
    TestMeta = qc.TestMeta
except (ImportError, AttributeError):
    qc = None
    TestType = None
    TestMeta = None

__all__ = ['qc', 'TestType', 'TestMeta']
