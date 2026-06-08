"""Surface contract for third-party charset_normalizer package.

# type-regime: monomorphic

Probes: charset_normalizer.from_bytes, charset_normalizer.from_fp,
charset_normalizer.from_path, charset_normalizer.detect,
charset_normalizer.CharsetMatch, charset_normalizer.CharsetMatches.
CPython 3.12 is the oracle.
"""

import charset_normalizer
import io

# Core API
assert hasattr(charset_normalizer, "from_bytes"), "from_bytes"
assert hasattr(charset_normalizer, "from_fp"), "from_fp"
assert hasattr(charset_normalizer, "from_path"), "from_path"
assert hasattr(charset_normalizer, "detect"), "detect"
assert hasattr(charset_normalizer, "CharsetMatch"), "CharsetMatch"
assert hasattr(charset_normalizer, "CharsetMatches"), "CharsetMatches"
assert hasattr(charset_normalizer, "is_binary"), "is_binary"

# from_bytes returns CharsetMatches
_result = charset_normalizer.from_bytes(b"hello world")
assert isinstance(_result, charset_normalizer.CharsetMatches), \
    f"from_bytes type = {type(_result)!r}"

# best() returns CharsetMatch or None
_best = _result.best()
assert _best is None or isinstance(_best, charset_normalizer.CharsetMatch), \
    f"best type = {type(_best)!r}"

# CharsetMatch attributes
if _best is not None:
    assert hasattr(_best, "encoding"), "best.encoding"
    assert hasattr(_best, "encoding_aliases"), "best.encoding_aliases"
    assert hasattr(_best, "percent_chaos"), "best.percent_chaos"
    assert hasattr(_best, "percent_coherence"), "best.percent_coherence"
    assert hasattr(_best, "languages"), "best.languages"
    assert hasattr(_best, "raw"), "best.raw"

# from_fp works with file-like object
_fp = io.BytesIO(b"hello world from file")
_result_fp = charset_normalizer.from_fp(_fp)
assert isinstance(_result_fp, charset_normalizer.CharsetMatches), \
    f"from_fp type = {type(_result_fp)!r}"

# detect returns dict with 'encoding', 'confidence', 'language'
_detected = charset_normalizer.detect(b"hello world")
assert isinstance(_detected, dict), f"detect type = {type(_detected)!r}"
assert "encoding" in _detected, "detected.encoding"
assert "confidence" in _detected, "detected.confidence"
assert "language" in _detected, "detected.language"

# is_binary returns bool
assert isinstance(charset_normalizer.is_binary(b"hello"), bool), "is_binary returns bool"
assert isinstance(charset_normalizer.is_binary(b"\x00\x01\x02"), bool), "is_binary binary"

# Module attributes are identity-stable
_fb_ref = charset_normalizer.from_bytes
assert charset_normalizer.from_bytes is _fb_ref, "from_bytes stable"
_det_ref = charset_normalizer.detect
assert charset_normalizer.detect is _det_ref, "detect stable"

print("surface OK")
