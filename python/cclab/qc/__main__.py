"""
Entry point for running cclab.qc as a module.

Usage:
    python -m cclab.qc [args]
"""

from .cli import main

if __name__ == "__main__":
    exit(main())
