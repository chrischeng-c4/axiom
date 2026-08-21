# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "real_world"
# case = "mojibake_recovery_pipeline"
# subject = "codecs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs: a text-ingest pipeline reads a mixed-encoding corpus: utf-8-sig source files (BOM-prefixed) are decoded via a StreamReader, latin-1 bytes are recovered, and a base64_codec transport layer round-trips the normalized text"""
import codecs

import io

# Synthetic mixed-encoding corpus: source files arrive as utf-8-sig (a BOM
# prefix common in Windows-exported CSV/text) and a legacy latin-1 stream.
SOURCES = [
    "café résumé",
    "naïve façade",
    "ABC¡∀XYZ",
    "Москва 北京 東京",
]

# Stage 1: ingest utf-8-sig source bytes through a StreamReader; the BOM
# must be stripped transparently so the recovered text is clean.
recovered = []
for text in SOURCES:
    raw = text.encode("utf-8-sig")
    reader = codecs.getreader("utf-8-sig")(io.BytesIO(raw))
    recovered.append(reader.read())
assert recovered == SOURCES, f"utf-8-sig ingest = {recovered!r}"

# Stage 2: a legacy latin-1 byte blob is recovered to text, then re-encoded
# to utf-8 for the normalized store.
legacy_latin1 = "Crème brûlée".encode("latin-1")
decoded = legacy_latin1.decode("latin-1")
assert decoded == "Crème brûlée"
normalized = codecs.encode(decoded, "utf-8")
assert codecs.decode(normalized, "utf-8") == decoded

# Stage 3: a base64_codec transport layer wraps each normalized record for
# safe transmission, then round-trips it back without loss.
store = "\n".join(recovered + [decoded])
payload = store.encode("utf-8")
transported = codecs.encode(payload, "base64_codec")
assert isinstance(transported, bytes)
assert codecs.decode(transported, "base64_codec") == payload

# Stage 4: aggregate a per-record byte tally to prove the pipeline ran end
# to end over every record.
total = sum(len(codecs.encode(r, "utf-8")) for r in recovered)
assert total > 0, "byte tally accumulated"

print("mojibake_recovery_pipeline OK")
