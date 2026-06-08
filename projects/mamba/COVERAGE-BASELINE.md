# Mamba Coverage Baseline

Frozen llvm-cov baseline for `projects/mamba/` — first cut of #1885 child task #1.

**Source commit**: `6edea62d5` (project-mamba @ 2026-05-18T14:22:40Z, post-#2422 merge)
**Tool**: `cargo llvm-cov --package mamba --lib --summary-only`
**Toolchain**: stable-aarch64-apple-darwin

## Totals

| Axis | Total | Missed | Coverage |
|------|-------|--------|----------|
| Regions | 217868 | 67577 | **68.98%** |
| Functions | 12068 | 3171 | **73.72%** |
| Lines | 103517 | 31979 | **69.11%** |

## Per-file (regions / functions / lines)

| File | Regions | Region % | Function % | Line % |
|------|---------|----------|------------|--------|
| projects/mamba/src/bench/mod.rs | 763 | 57.14% | 58.62% | 71.50% |
| projects/mamba/src/codegen/cranelift/aot.rs | 601 | 96.01% | 44.44% | 97.13% |
| projects/mamba/src/codegen/cranelift/jit.rs | 3561 | 50.15% | 60.56% | 55.39% |
| projects/mamba/src/codegen/cranelift/marshal.rs | 181 | 58.01% | 100.00% | 72.63% |
| projects/mamba/src/codegen/cranelift/mod.rs | 3345 | 26.28% | 60.94% | 37.90% |
| projects/mamba/src/codegen/cranelift/perf_map.rs | 177 | 93.79% | 93.75% | 96.77% |
| projects/mamba/src/codegen/llvm.rs | 1051 | 72.31% | 87.18% | 73.17% |
| projects/mamba/src/codegen/mod.rs | 70 | 90.00% | 80.00% | 98.08% |
| projects/mamba/src/config/schema.rs | 479 | 93.74% | 97.22% | 92.45% |
| projects/mamba/src/conformance/mod.rs | 940 | 74.26% | 85.19% | 72.82% |
| projects/mamba/src/conformance/pytest_runner.rs | 2345 | 55.82% | 51.30% | 56.55% |
| projects/mamba/src/diagnostic/mod.rs | 20 | 90.00% | 100.00% | 88.89% |
| projects/mamba/src/driver/config.rs | 295 | 98.31% | 100.00% | 98.18% |
| projects/mamba/src/driver/mod.rs | 1245 | 64.90% | 70.18% | 68.35% |
| projects/mamba/src/driver/module_graph.rs | 749 | 84.38% | 78.00% | 83.38% |
| projects/mamba/src/driver/repl.rs | 554 | 61.73% | 55.17% | 56.51% |
| projects/mamba/src/error.rs | 25 | 80.00% | 100.00% | 84.21% |
| projects/mamba/src/ffi/c_parser.rs | 900 | 97.67% | 100.00% | 98.90% |
| projects/mamba/src/ffi/c_types.rs | 251 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/ffi/cbindgen.rs | 179 | 73.74% | 93.75% | 78.12% |
| projects/mamba/src/ffi/memory.rs | 95 | 98.95% | 100.00% | 98.85% |
| projects/mamba/src/ffi/safety.rs | 375 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/ffi/stub_gen.rs | 431 | 93.74% | 100.00% | 96.14% |
| projects/mamba/src/ffi/type_map.rs | 594 | 96.63% | 100.00% | 96.60% |
| projects/mamba/src/hir/mod.rs | 340 | 92.65% | 100.00% | 96.71% |
| projects/mamba/src/lexer/indent.rs | 115 | 96.52% | 83.33% | 91.89% |
| projects/mamba/src/lexer/mod.rs | 38 | 81.58% | 66.67% | 72.73% |
| projects/mamba/src/lexer/token.rs | 2020 | 77.52% | 100.00% | 84.28% |
| projects/mamba/src/lower/ast_to_hir.rs | 7054 | 68.05% | 64.63% | 72.29% |
| projects/mamba/src/lower/hir_to_mir.rs | 9840 | 31.91% | 57.78% | 34.50% |
| projects/mamba/src/mir/mod.rs | 241 | 95.85% | 100.00% | 100.00% |
| projects/mamba/src/parser/expr.rs | 1792 | 72.49% | 98.68% | 77.45% |
| projects/mamba/src/parser/expr_compound.rs | 1062 | 78.06% | 100.00% | 87.50% |
| projects/mamba/src/parser/mod.rs | 476 | 82.77% | 86.11% | 79.39% |
| projects/mamba/src/parser/pattern.rs | 1240 | 80.56% | 91.23% | 83.69% |
| projects/mamba/src/parser/stmt.rs | 1546 | 71.28% | 98.36% | 78.46% |
| projects/mamba/src/parser/stmt_compound.rs | 1982 | 72.96% | 97.30% | 81.02% |
| projects/mamba/src/parser/type_expr.rs | 539 | 70.13% | 96.00% | 78.51% |
| projects/mamba/src/pkgmgr/cache.rs | 608 | 83.22% | 77.78% | 81.37% |
| projects/mamba/src/pkgmgr/http.rs | 1488 | 84.48% | 77.78% | 81.44% |
| projects/mamba/src/pkgmgr/installer/archive.rs | 282 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/installer/layout.rs | 188 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/installer/mod.rs | 343 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/installer/record.rs | 162 | 62.35% | 83.33% | 53.64% |
| projects/mamba/src/pkgmgr/installer/scripts.rs | 180 | 68.33% | 60.00% | 66.06% |
| projects/mamba/src/pkgmgr/installer/uninstall.rs | 143 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/json_api.rs | 200 | 98.50% | 100.00% | 98.88% |
| projects/mamba/src/pkgmgr/lockfile/invalidate.rs | 130 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/lockfile/mod.rs | 144 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/lockfile/parse.rs | 52 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/lockfile/serialize.rs | 53 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/pep440.rs | 402 | 90.80% | 96.30% | 91.86% |
| projects/mamba/src/pkgmgr/resolver/graph.rs | 5 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/resolver/mod.rs | 158 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/pkgmgr/resolver/pubgrub_glue.rs | 111 | 56.76% | 41.67% | 47.30% |
| projects/mamba/src/pkgmgr/resolver/requirement.rs | 192 | 86.46% | 81.82% | 86.96% |
| projects/mamba/src/pkgmgr/resolver/specifier.rs | 245 | 89.39% | 86.67% | 89.47% |
| projects/mamba/src/pkgmgr/simple_api.rs | 562 | 88.97% | 96.15% | 92.90% |
| projects/mamba/src/pkgmgr/types.rs | 3 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/resolve/pass.rs | 3112 | 72.33% | 86.60% | 85.23% |
| projects/mamba/src/resolve/scope.rs | 301 | 99.00% | 95.45% | 97.69% |
| projects/mamba/src/runtime/async_rt.rs | 382 | 84.55% | 85.19% | 82.30% |
| projects/mamba/src/runtime/async_task.rs | 726 | 64.19% | 78.05% | 60.61% |
| projects/mamba/src/runtime/bigint_ops.rs | 655 | 81.22% | 87.80% | 78.66% |
| projects/mamba/src/runtime/builtins.rs | 10088 | 53.91% | 72.05% | 52.92% |
| projects/mamba/src/runtime/bytes_ops.rs | 2540 | 65.79% | 74.13% | 64.33% |
| projects/mamba/src/runtime/class.rs | 14435 | 55.17% | 69.63% | 50.85% |
| projects/mamba/src/runtime/closure.rs | 1485 | 83.03% | 80.83% | 79.36% |
| projects/mamba/src/runtime/dict_ops.rs | 2758 | 76.76% | 89.78% | 74.02% |
| projects/mamba/src/runtime/exception.rs | 3539 | 93.50% | 89.81% | 90.32% |
| projects/mamba/src/runtime/file_io.rs | 854 | 86.30% | 87.18% | 80.80% |
| projects/mamba/src/runtime/gc.rs | 1220 | 91.56% | 95.83% | 96.23% |
| projects/mamba/src/runtime/generator.rs | 1320 | 35.23% | 44.72% | 35.73% |
| projects/mamba/src/runtime/integer_handle_registry.rs | 37 | 40.54% | 66.67% | 42.86% |
| projects/mamba/src/runtime/iter.rs | 2672 | 56.10% | 71.14% | 52.49% |
| projects/mamba/src/runtime/list_ops.rs | 3235 | 64.11% | 76.83% | 63.09% |
| projects/mamba/src/runtime/mod.rs | 373 | 96.78% | 100.00% | 94.52% |
| projects/mamba/src/runtime/module.rs | 2981 | 85.44% | 88.21% | 83.35% |
| projects/mamba/src/runtime/output.rs | 129 | 96.12% | 92.86% | 95.83% |
| projects/mamba/src/runtime/rc.rs | 916 | 93.23% | 98.48% | 92.67% |
| projects/mamba/src/runtime/registry_bridge.rs | 217 | 0.00% | 0.00% | 0.00% |
| projects/mamba/src/runtime/set_ops.rs | 2287 | 90.47% | 91.67% | 87.51% |
| projects/mamba/src/runtime/stdlib/abc_mod.rs | 305 | 93.11% | 93.55% | 93.29% |
| projects/mamba/src/runtime/stdlib/aiofiles_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/aiohttp_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/alembic_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/argparse_mod.rs | 401 | 84.54% | 95.45% | 80.11% |
| projects/mamba/src/runtime/stdlib/array_mod.rs | 1865 | 75.39% | 87.94% | 77.75% |
| projects/mamba/src/runtime/stdlib/ast_mod.rs | 357 | 75.91% | 59.26% | 77.60% |
| projects/mamba/src/runtime/stdlib/asyncio_mod.rs | 161 | 43.48% | 36.36% | 43.08% |
| projects/mamba/src/runtime/stdlib/atexit_mod.rs | 181 | 85.64% | 68.18% | 87.37% |
| projects/mamba/src/runtime/stdlib/attrs_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/azure_core_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/azure_identity_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/azure_keyvault_secrets_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/azure_storage_blob_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/base64_mod.rs | 934 | 40.36% | 43.14% | 35.94% |
| projects/mamba/src/runtime/stdlib/bdb_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/binascii_mod.rs | 389 | 83.03% | 84.62% | 82.02% |
| projects/mamba/src/runtime/stdlib/bisect_mod.rs | 340 | 80.88% | 85.71% | 83.83% |
| projects/mamba/src/runtime/stdlib/boto3_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/botocore_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/builtins_mod.rs | 1281 | 34.27% | 36.96% | 42.32% |
| projects/mamba/src/runtime/stdlib/bz2_mod.rs | 343 | 91.25% | 95.24% | 92.00% |
| projects/mamba/src/runtime/stdlib/calendar_mod.rs | 1183 | 85.71% | 86.52% | 86.61% |
| projects/mamba/src/runtime/stdlib/celery_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/certifi_mod.rs | 125 | 77.60% | 75.00% | 76.67% |
| projects/mamba/src/runtime/stdlib/cgi_mod.rs | 489 | 69.12% | 61.29% | 67.71% |
| projects/mamba/src/runtime/stdlib/charset_normalizer_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/cmath_mod.rs | 882 | 76.42% | 77.97% | 78.25% |
| projects/mamba/src/runtime/stdlib/code_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/codecs_mod.rs | 1289 | 92.40% | 88.79% | 92.25% |
| projects/mamba/src/runtime/stdlib/collections_mod.rs | 2040 | 50.54% | 50.40% | 48.64% |
| projects/mamba/src/runtime/stdlib/colorsys_mod.rs | 568 | 84.86% | 95.24% | 89.55% |
| projects/mamba/src/runtime/stdlib/compileall_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/concurrent_futures_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/configparser_mod.rs | 729 | 84.64% | 86.11% | 80.61% |
| projects/mamba/src/runtime/stdlib/contextlib_mod.rs | 155 | 74.84% | 63.64% | 73.61% |
| projects/mamba/src/runtime/stdlib/contextvars_mod.rs | 103 | 87.38% | 75.00% | 80.00% |
| projects/mamba/src/runtime/stdlib/copy_mod.rs | 518 | 83.20% | 71.43% | 79.36% |
| projects/mamba/src/runtime/stdlib/cryptography_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/csv_mod.rs | 980 | 74.08% | 58.46% | 69.53% |
| projects/mamba/src/runtime/stdlib/dataclasses_mod.rs | 524 | 96.56% | 100.00% | 94.58% |
| projects/mamba/src/runtime/stdlib/datetime_mod.rs | 1270 | 52.99% | 47.19% | 56.66% |
| projects/mamba/src/runtime/stdlib/dbm_mod.rs | 56 | 78.57% | 40.00% | 65.38% |
| projects/mamba/src/runtime/stdlib/decimal_mod.rs | 437 | 77.57% | 76.74% | 75.12% |
| projects/mamba/src/runtime/stdlib/dev_tools_mod.rs | 566 | 95.23% | 73.91% | 92.86% |
| projects/mamba/src/runtime/stdlib/difflib_mod.rs | 516 | 92.05% | 94.59% | 91.58% |
| projects/mamba/src/runtime/stdlib/dis_mod.rs | 326 | 71.17% | 48.15% | 68.94% |
| projects/mamba/src/runtime/stdlib/email_mod.rs | 333 | 73.87% | 60.00% | 76.29% |
| projects/mamba/src/runtime/stdlib/encodings_mod.rs | 277 | 92.06% | 86.96% | 94.55% |
| projects/mamba/src/runtime/stdlib/enum_mod.rs | 572 | 94.41% | 96.43% | 88.75% |
| projects/mamba/src/runtime/stdlib/errno_mod.rs | 842 | 98.22% | 98.15% | 98.13% |
| projects/mamba/src/runtime/stdlib/fastapi_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/filecmp_mod.rs | 682 | 89.74% | 84.85% | 87.15% |
| projects/mamba/src/runtime/stdlib/fileinput_mod.rs | 70 | 52.86% | 18.18% | 46.00% |
| projects/mamba/src/runtime/stdlib/flask_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/fnmatch_mod.rs | 766 | 63.58% | 83.87% | 60.29% |
| projects/mamba/src/runtime/stdlib/fractions_mod.rs | 1259 | 54.41% | 58.89% | 57.19% |
| projects/mamba/src/runtime/stdlib/functools_mod.rs | 1638 | 41.27% | 48.78% | 44.40% |
| projects/mamba/src/runtime/stdlib/future_mod.rs | 160 | 99.38% | 100.00% | 99.03% |
| projects/mamba/src/runtime/stdlib/gc_mod.rs | 215 | 80.00% | 61.90% | 80.17% |
| projects/mamba/src/runtime/stdlib/getopt_mod.rs | 844 | 88.63% | 92.31% | 87.01% |
| projects/mamba/src/runtime/stdlib/getpass_mod.rs | 64 | 46.88% | 28.57% | 47.06% |
| projects/mamba/src/runtime/stdlib/glob_mod.rs | 691 | 88.57% | 88.24% | 88.45% |
| projects/mamba/src/runtime/stdlib/google_api_core_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/google_cloud_pubsub_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/google_cloud_storage_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/googleapis_common_protos_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/graphlib_mod.rs | 1066 | 88.65% | 86.11% | 86.00% |
| projects/mamba/src/runtime/stdlib/grpcio_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/grpclib_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/gunicorn_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/gzip_mod.rs | 441 | 91.84% | 96.00% | 92.99% |
| projects/mamba/src/runtime/stdlib/hashlib_mod.rs | 843 | 71.29% | 77.92% | 71.14% |
| projects/mamba/src/runtime/stdlib/heapq_mod.rs | 537 | 88.83% | 94.44% | 88.76% |
| projects/mamba/src/runtime/stdlib/hmac_mod.rs | 1068 | 75.37% | 86.96% | 76.38% |
| projects/mamba/src/runtime/stdlib/html_parser_mod.rs | 246 | 86.18% | 78.95% | 86.43% |
| projects/mamba/src/runtime/stdlib/http_cookiejar_mod.rs | 156 | 62.18% | 52.63% | 59.78% |
| projects/mamba/src/runtime/stdlib/http_cookies_mod.rs | 136 | 80.88% | 68.75% | 78.48% |
| projects/mamba/src/runtime/stdlib/http_mod.rs | 1645 | 72.52% | 69.41% | 73.76% |
| projects/mamba/src/runtime/stdlib/httpx_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/hypothesis_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/idlelib_mod.rs | 249 | 78.71% | 47.62% | 84.31% |
| projects/mamba/src/runtime/stdlib/idna_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/importlib_mod.rs | 232 | 81.03% | 90.00% | 85.32% |
| projects/mamba/src/runtime/stdlib/inspect_mod.rs | 251 | 72.11% | 78.57% | 74.40% |
| projects/mamba/src/runtime/stdlib/io_mod.rs | 740 | 76.22% | 80.49% | 71.39% |
| projects/mamba/src/runtime/stdlib/ipaddress_mod.rs | 736 | 47.96% | 43.40% | 45.33% |
| projects/mamba/src/runtime/stdlib/itertools_mod.rs | 2036 | 58.20% | 59.80% | 58.58% |
| projects/mamba/src/runtime/stdlib/jinja2_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/jmespath_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/json_mod.rs | 1684 | 60.15% | 63.79% | 59.92% |
| projects/mamba/src/runtime/stdlib/jsonschema_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/keyword_mod.rs | 201 | 90.05% | 94.12% | 89.66% |
| projects/mamba/src/runtime/stdlib/kombu_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/linecache_mod.rs | 437 | 86.73% | 91.11% | 87.65% |
| projects/mamba/src/runtime/stdlib/locale_mod.rs | 261 | 91.57% | 92.00% | 90.74% |
| projects/mamba/src/runtime/stdlib/logging_mod.rs | 410 | 79.51% | 79.49% | 78.06% |
| projects/mamba/src/runtime/stdlib/long_tail2_mod.rs | 566 | 95.23% | 36.36% | 95.29% |
| projects/mamba/src/runtime/stdlib/long_tail3_mod.rs | 653 | 95.87% | 66.67% | 96.25% |
| projects/mamba/src/runtime/stdlib/long_tail4_mod.rs | 485 | 95.26% | 71.43% | 95.53% |
| projects/mamba/src/runtime/stdlib/long_tail_mod.rs | 336 | 92.86% | 77.78% | 94.02% |
| projects/mamba/src/runtime/stdlib/lzma_mod.rs | 462 | 94.16% | 95.24% | 94.56% |
| projects/mamba/src/runtime/stdlib/main_mod.rs | 71 | 97.18% | 100.00% | 96.88% |
| projects/mamba/src/runtime/stdlib/marshmallow_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/math_mod.rs | 1632 | 52.94% | 68.29% | 51.57% |
| projects/mamba/src/runtime/stdlib/mimetypes_mod.rs | 530 | 71.70% | 75.00% | 74.05% |
| projects/mamba/src/runtime/stdlib/mock_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/mod.rs | 196 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/runtime/stdlib/msgpack_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/multiprocessing_mod.rs | 69 | 78.26% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/netrc_mod.rs | 730 | 89.59% | 93.55% | 86.86% |
| projects/mamba/src/runtime/stdlib/ntpath_mod.rs | 1481 | 48.89% | 48.48% | 51.86% |
| projects/mamba/src/runtime/stdlib/numbers_mod.rs | 197 | 92.89% | 90.91% | 96.43% |
| projects/mamba/src/runtime/stdlib/operator_mod.rs | 596 | 61.58% | 61.54% | 58.20% |
| projects/mamba/src/runtime/stdlib/orjson_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/os_mod.rs | 1195 | 62.09% | 56.52% | 60.21% |
| projects/mamba/src/runtime/stdlib/pathlib_mod.rs | 1225 | 77.96% | 75.26% | 76.91% |
| projects/mamba/src/runtime/stdlib/pickle_mod.rs | 709 | 92.67% | 92.50% | 94.72% |
| projects/mamba/src/runtime/stdlib/platform_mod.rs | 183 | 93.99% | 86.96% | 93.02% |
| projects/mamba/src/runtime/stdlib/posix_mod.rs | 551 | 70.42% | 57.14% | 66.04% |
| projects/mamba/src/runtime/stdlib/posixpath_mod.rs | 1339 | 52.88% | 44.94% | 56.55% |
| projects/mamba/src/runtime/stdlib/pprint_mod.rs | 197 | 54.31% | 63.64% | 57.14% |
| projects/mamba/src/runtime/stdlib/protobuf_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/psycopg_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/pydantic_core_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/pydantic_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/pyopenssl_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/pytest_asyncio_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/pytest_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/queue_mod.rs | 548 | 83.21% | 77.55% | 81.29% |
| projects/mamba/src/runtime/stdlib/quopri_mod.rs | 453 | 86.98% | 93.10% | 84.82% |
| projects/mamba/src/runtime/stdlib/random_mod.rs | 1593 | 53.67% | 43.92% | 53.27% |
| projects/mamba/src/runtime/stdlib/re_mod.rs | 2015 | 72.56% | 74.71% | 68.90% |
| projects/mamba/src/runtime/stdlib/readline_mod.rs | 801 | 70.54% | 55.56% | 80.12% |
| projects/mamba/src/runtime/stdlib/redis_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/reprlib_mod.rs | 400 | 79.25% | 81.25% | 76.65% |
| projects/mamba/src/runtime/stdlib/requests_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/s3transfer_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/secrets_mod.rs | 585 | 89.91% | 92.68% | 87.50% |
| projects/mamba/src/runtime/stdlib/selectors_mod.rs | 284 | 78.52% | 56.67% | 72.00% |
| projects/mamba/src/runtime/stdlib/shlex_mod.rs | 322 | 94.72% | 96.77% | 95.24% |
| projects/mamba/src/runtime/stdlib/shutil_mod.rs | 734 | 80.79% | 83.72% | 81.84% |
| projects/mamba/src/runtime/stdlib/signal_mod.rs | 547 | 69.84% | 68.18% | 71.26% |
| projects/mamba/src/runtime/stdlib/site_mod.rs | 84 | 72.62% | 28.57% | 63.41% |
| projects/mamba/src/runtime/stdlib/socket_mod.rs | 575 | 82.26% | 85.42% | 83.60% |
| projects/mamba/src/runtime/stdlib/sqlalchemy_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/sqlite3_mod.rs | 608 | 93.26% | 97.92% | 90.53% |
| projects/mamba/src/runtime/stdlib/ssl_mod.rs | 321 | 80.69% | 22.22% | 66.12% |
| projects/mamba/src/runtime/stdlib/starlette_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/stat_mod.rs | 355 | 74.37% | 79.17% | 80.32% |
| projects/mamba/src/runtime/stdlib/statistics_mod.rs | 2083 | 75.90% | 78.03% | 77.42% |
| projects/mamba/src/runtime/stdlib/string_constants_mod.rs | 201 | 68.66% | 55.56% | 63.10% |
| projects/mamba/src/runtime/stdlib/stringprep_mod.rs | 513 | 82.85% | 72.41% | 80.50% |
| projects/mamba/src/runtime/stdlib/struct_mod.rs | 1209 | 71.88% | 87.04% | 72.90% |
| projects/mamba/src/runtime/stdlib/subprocess_mod.rs | 777 | 93.18% | 95.74% | 91.84% |
| projects/mamba/src/runtime/stdlib/sys_mod.rs | 798 | 81.83% | 47.92% | 72.81% |
| projects/mamba/src/runtime/stdlib/sysconfig_mod.rs | 275 | 14.18% | 11.76% | 20.16% |
| projects/mamba/src/runtime/stdlib/tarfile_mod.rs | 199 | 80.40% | 84.62% | 78.89% |
| projects/mamba/src/runtime/stdlib/tempfile_mod.rs | 260 | 78.08% | 57.69% | 69.86% |
| projects/mamba/src/runtime/stdlib/test_mod.rs | 452 | 90.93% | 93.48% | 90.67% |
| projects/mamba/src/runtime/stdlib/textwrap_mod.rs | 315 | 84.44% | 90.00% | 86.21% |
| projects/mamba/src/runtime/stdlib/thirdparty_shells_mod.rs | 23 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/runtime/stdlib/threading_mod.rs | 1283 | 96.49% | 97.27% | 96.68% |
| projects/mamba/src/runtime/stdlib/time_mod.rs | 1398 | 93.42% | 94.90% | 93.56% |
| projects/mamba/src/runtime/stdlib/token_mod.rs | 200 | 86.00% | 92.86% | 86.54% |
| projects/mamba/src/runtime/stdlib/tokenize_mod.rs | 509 | 68.37% | 65.00% | 64.88% |
| projects/mamba/src/runtime/stdlib/tomllib_mod.rs | 322 | 76.40% | 86.67% | 71.83% |
| projects/mamba/src/runtime/stdlib/traceback_mod.rs | 754 | 91.91% | 92.65% | 92.00% |
| projects/mamba/src/runtime/stdlib/tracemalloc_mod.rs | 230 | 72.61% | 65.00% | 73.28% |
| projects/mamba/src/runtime/stdlib/types_mod.rs | 491 | 93.08% | 98.21% | 94.38% |
| projects/mamba/src/runtime/stdlib/typing_extensions_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/typing_mod.rs | 113 | 72.57% | 75.00% | 80.33% |
| projects/mamba/src/runtime/stdlib/unicodedata_mod.rs | 223 | 82.96% | 80.00% | 80.72% |
| projects/mamba/src/runtime/stdlib/unittest_mock_mod.rs | 421 | 92.16% | 90.00% | 86.60% |
| projects/mamba/src/runtime/stdlib/unittest_mod.rs | 694 | 79.25% | 76.32% | 78.04% |
| projects/mamba/src/runtime/stdlib/urllib3_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/urllib_error_mod.rs | 341 | 90.32% | 84.62% | 88.89% |
| projects/mamba/src/runtime/stdlib/uu_mod.rs | 657 | 76.86% | 61.36% | 75.56% |
| projects/mamba/src/runtime/stdlib/uuid_mod.rs | 807 | 77.32% | 75.00% | 72.94% |
| projects/mamba/src/runtime/stdlib/uvicorn_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/warnings_mod.rs | 870 | 80.69% | 78.33% | 83.85% |
| projects/mamba/src/runtime/stdlib/weakref_mod.rs | 718 | 85.93% | 78.05% | 84.82% |
| projects/mamba/src/runtime/stdlib/webbrowser_mod.rs | 164 | 57.32% | 40.00% | 53.03% |
| projects/mamba/src/runtime/stdlib/werkzeug_mod.rs | 70 | 77.14% | 33.33% | 62.50% |
| projects/mamba/src/runtime/stdlib/wsgiref_mod.rs | 186 | 85.48% | 60.00% | 83.33% |
| projects/mamba/src/runtime/stdlib/xml_mod.rs | 925 | 74.38% | 70.45% | 71.82% |
| projects/mamba/src/runtime/stdlib/xmlrpc_mod.rs | 116 | 81.90% | 60.00% | 85.53% |
| projects/mamba/src/runtime/stdlib/zipfile_mod.rs | 303 | 84.49% | 88.24% | 79.84% |
| projects/mamba/src/runtime/stdlib/zlib_mod.rs | 315 | 91.43% | 96.00% | 92.06% |
| projects/mamba/src/runtime/string_ops.rs | 6895 | 63.25% | 78.44% | 59.72% |
| projects/mamba/src/runtime/symbols.rs | 1750 | 99.26% | 87.50% | 99.68% |
| projects/mamba/src/runtime/tokio_exec.rs | 320 | 90.94% | 69.57% | 91.25% |
| projects/mamba/src/runtime/tuple_ops.rs | 1381 | 75.67% | 79.27% | 70.82% |
| projects/mamba/src/runtime/value.rs | 1132 | 98.59% | 98.21% | 98.47% |
| projects/mamba/src/source/mod.rs | 119 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/source/span.rs | 89 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/surface.rs | 1540 | 87.21% | 80.77% | 82.67% |
| projects/mamba/src/types/builtins.rs | 788 | 96.83% | 94.44% | 95.64% |
| projects/mamba/src/types/check.rs | 1585 | 61.96% | 72.97% | 61.73% |
| projects/mamba/src/types/check_expr.rs | 1640 | 33.17% | 40.00% | 35.88% |
| projects/mamba/src/types/check_stmt.rs | 980 | 20.71% | 13.64% | 22.41% |
| projects/mamba/src/types/context.rs | 715 | 100.00% | 100.00% | 100.00% |
| projects/mamba/src/types/generic.rs | 1090 | 99.17% | 100.00% | 99.55% |
| projects/mamba/src/types/protocol.rs | 588 | 98.81% | 100.00% | 98.04% |
| projects/mamba/src/types/ty.rs | 238 | 100.00% | 100.00% | 100.00% |

## How to reproduce

```bash
rustup component add llvm-tools-preview
LLVM_COV="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n "s/host: //p")/bin/llvm-cov" \
LLVM_PROFDATA="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n "s/host: //p")/bin/llvm-profdata" \
  cargo llvm-cov --package mamba --lib --summary-only
```

## Next child tasks

- [ ] CI artifact: emit `cargo llvm-cov --json` from GHA, attach as workflow artifact.
- [ ] PR gate: reject merges that drop total region% by more than 0.5pp (`coverage-allowed-drop` label opt-out).
- [ ] Untested file enumeration: `projects/mamba/COVERAGE-GAPS.md` — auto-sorted list of files <100% with uncovered line count.
