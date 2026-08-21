# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "comparable_version_ordering"
# subject = "platform._comparable_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._comparable_version: _comparable_version sorts version strings: numeric segments compare numerically, separators (. _ - +) normalize, and the pre-release ladder is dev<alpha<beta<candidate<final<post"""
import platform

V = platform._comparable_version

# Equal strings compare equal; numeric segments sort numerically.
assert V("1.2.3") == V("1.2.3"), "identical strings equal"
assert V("8.02") == V("8.02"), "leading zero kept stable"
assert V("1.2.3") < V("1.2.10"), "10 sorts after 3 numerically"
assert V("0.9") < V("2.2"), "major number dominates"

# Mixed separators (. _ - +) normalize to the same key.
assert V("1.2.3.4") == V("1_2-3+4"), "separators normalize"

# Pre-release ladder: dev < alpha/a < beta/b < candidate/rc < final < post.
assert V("1.2dev") < V("1.2alpha"), "dev before alpha"
assert V("1.2alpha") < V("1.2beta"), "alpha before beta"
assert V("1.2a") < V("1.2b"), "a before b"
assert V("1.2b") < V("1.2c"), "b before c"
assert V("1.2c") < V("1.2rc"), "c before rc"
assert V("1.2rc") < V("1.2.0"), "rc before final release"
assert V("1.2.0") < V("1.2pl"), "final before post-level"

# More-specific version sorts after its prefix.
assert V("1.2") < V("1.2.1"), "1.2 before 1.2.1"
assert V("0.4") < V("0.4.0"), "bare before zero-padded"

print("comparable_version_ordering OK")
