---
id: implementation
type: change_implementation
change_id: enhancement-resolver-conditional-exports-import-require-browse
---

# Implementation

## Summary

*(auto-generated baseline from git diff)*

## Changed Files

```
M	.claude/skills/score-monitor-idle/SKILL.md
M	.aw/issues/closed/bug-dual-mambaconfig-structs-driver-config-rs-vs-confi.md
M	.aw/issues/closed/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
M	.aw/issues/closed/enhancement-add-datetime-utilities-parsing-timezone-arithmetic.md
M	.aw/issues/closed/enhancement-add-general-purpose-model-validation-pydantic-like.md
M	.aw/issues/closed/enhancement-add-missing-ml-algorithms.md
M	.aw/issues/closed/enhancement-add-retry-backoff-utilities.md
M	.aw/issues/closed/enhancement-add-sqlalchemy-compatible-declarative-orm-layer.md
M	.aw/issues/closed/enhancement-add-statistical-annotations-and-dual-axis-support.md
M	.aw/issues/closed/enhancement-add-yaml-toml-parsing-and-dotenv-loading.md
M	.aw/issues/closed/enhancement-audio-processing-and-file-i-o.md
M	.aw/issues/closed/enhancement-cacheutils-extended-lri-thresholdcounter-cachedpro.md
M	.aw/issues/closed/enhancement-cclab-runtime-core-shared-runtime-layer-for-cclab.md
M	.aw/issues/closed/enhancement-dictutils-orderedmultidict-frozendict-onetoone-sub.md
M	.aw/issues/closed/enhancement-export-basemodel-field-from-cclab-schema-not-just.md
M	.aw/issues/closed/enhancement-expose-dl-layers-and-optimizers-to-python.md
M	.aw/issues/closed/enhancement-expose-existing-rust-ml-models-to-python.md
M	.aw/issues/closed/enhancement-filesystem-storage-impl-of-generalized-artifactsto.md
M	.aw/issues/closed/enhancement-fileutils-atomic-save-fileperms-mkdir-p-iter-find.md
M	.aw/issues/closed/enhancement-formatutils-tokenize-format-str-get-format-args-de.md
M	.aw/issues/closed/enhancement-implement-png-pdf-export.md
M	.aw/issues/closed/enhancement-issue-authoring-notation-agent-for-rough-idea-well.md
M	.aw/issues/closed/enhancement-issuesource-batch-fetch-webhook-receiver-for-issue.md
M	.aw/issues/closed/enhancement-iterutils-extended-bucketize-same-split-remap-get.md
M	.aw/issues/closed/enhancement-jsonutils-jsonliterator-reverse-iter-lines-cclab-s.md
M	.aw/issues/closed/enhancement-mathutils-clamp-bits-bit-string-array-cclab-sci.md
M	.aw/issues/closed/enhancement-onnx-runtime-inference-engine-eliminate-ml-depende.md
M	.aw/issues/closed/enhancement-pathutils-augpath-shrinkuser-expandpath-cclab-util.md
M	.aw/issues/closed/enhancement-platform-adapter-format-conversion-comment-create.md
M	.aw/issues/closed/enhancement-pyo3-native-modules-not-loadable-so-build-system-b.md
M	.aw/issues/closed/enhancement-queueutils-heappriorityqueue-sortedpriorityqueue-c.md
M	.aw/issues/closed/enhancement-remove-silent-importerror-swallowing-in-init-py-fi.md
M	.aw/issues/closed/enhancement-setutils-indexedset-with-ordering-indexing-and-set.md
M	.aw/issues/closed/enhancement-statsutils-stats-class-histogram-trimean-kurtosis.md
M	.aw/issues/closed/enhancement-strutils-slugify-case-conversion-pluralize-asciify.md
M	.aw/issues/closed/enhancement-tableutils-table-type-with-to-text-to-html-from-di.md
M	.aw/issues/closed/enhancement-tbutils-tracebackinfo-parsedexception-contextualca.md
M	.aw/issues/closed/enhancement-timeutils-extended-daterange-isoparse-parse-timede.md
M	.aw/issues/closed/enhancement-tracking-rust-utility-crate-boltons-inspired-batte.md
M	.aw/issues/closed/enhancement-typeutils-make-sentinel-classproperty-get-all-subc.md
M	.aw/issues/closed/enhancement-update-python-stubs-and-expose-missing-chart-types.md
M	.aw/issues/closed/enhancement-urlutils-url-class-find-all-links-parse-qsl-quote.md
M	.aw/issues/closed/enhancement-video-codec-and-container-format-support.md
M	.aw/issues/closed/epic-align-cclab-python-api-to-ecosystem-conventions.md
M	.aw/issues/closed/epic-align-with-cclab-sdd-as-cloud-sdd-runtime.md
M	.aw/issues/closed/epic-conductor-rewires-onto-arsenal-crates-extends-1049.md
M	.aw/issues/closed/epic-create-projects-score-local-cli-sdd-show-case.md
M	.aw/issues/closed/epic-decompose-cclab-sdd-library-multi-kind-generalizat.md
M	.aw/issues/closed/epic-design-mamba-concurrency-semantics-no-gil-thread-s.md
M	.aw/issues/closed/epic-domain-first-monorepo-layout-libs-apps-platform.md
M	.aw/issues/closed/epic-formal-notation-for-non-code-artifact-kinds-brd-pr.md
M	.aw/issues/closed/epic-py3-12-conformance-tracking-session-progress.md
M	.aw/issues/closed/epic-refactor-to-thin-shell-extract-all-logic-to-cclab.md
M	.aw/issues/closed/epic-sdd-post-change-operations-workflow-monitor-detect.md
M	.aw/issues/closed/epic-sdd-pre-change-planning-workflow-goals-issues.md
M	.aw/issues/closed/epic-tracking-c-library-ecosystem-compatibility-strateg.md
M	.aw/issues/closed/epic-tracking-mamba-language-features-tooling-completen.md
M	.aw/issues/closed/epic-tracking-mamba-package-manager-uv-like.md
M	.aw/issues/closed/epic-tracking-mamba-py3-12-conformance-test-coverage.md
M	.aw/issues/closed/epic-tracking-mamba-stdlib-native-rust-implementation.md
M	.aw/issues/closed/epic-user-facing-documentation-generation-project-type.md
M	.aw/issues/closed/refactor-extract-api-cli-commands-into-cclab-api-cli.md
M	.aw/issues/closed/refactor-extract-kv-cli-commands-into-cclab-kv-cli.md
M	.aw/issues/closed/refactor-extract-qc-cli-commands-into-cclab-qc-cli.md
M	.aw/issues/closed/refactor-extract-queue-cli-commands-into-cclab-queue-cli.md
M	.aw/issues/closed/refactor-extract-razer-cli-commands-into-cclab-razer-cli.md
M	.aw/issues/closed/refactor-extract-sdd-cloud-agents-into-separate-crate.md
D	.aw/issues/open/bug-score-init-missing-5-skill-templates-handoff-takeo.md
D	.aw/issues/open/enhancement-add-advanced-filtering-for-issues-priority-labels.md
D	.aw/issues/open/enhancement-add-global-toast-notification-system-mui-snackbar.md
D	.aw/issues/open/enhancement-add-interaction-diagrams-for-fe-be-async-flows-sse.md
D	.aw/issues/open/enhancement-add-jet-test-cli-command-invoke-playwright-from-pr.md
D	.aw/issues/open/enhancement-add-mark-namespace-and-raises-to-cclab-qc.md
D	.aw/issues/open/enhancement-add-native-stdlib-ast-abstract-syntax-tree.md
D	.aw/issues/open/enhancement-add-native-stdlib-bdb-debugger-framework.md
D	.aw/issues/open/enhancement-add-native-stdlib-binascii-binary-ascii-conversion.md
D	.aw/issues/open/enhancement-add-native-stdlib-builtins-built-in-objects-module.md
D	.aw/issues/open/enhancement-add-native-stdlib-cmd-line-oriented-command-interp.md
D	.aw/issues/open/enhancement-add-native-stdlib-codecs-codec-registry-and-base-c.md
D	.aw/issues/open/enhancement-add-native-stdlib-collections-abc-abstract-base-cl.md
D	.aw/issues/open/enhancement-add-native-stdlib-colorsys-color-system-conversion.md
D	.aw/issues/open/enhancement-add-native-stdlib-concurrent-futures-async-executi.md
D	.aw/issues/open/enhancement-add-native-stdlib-contextvars-context-local-state.md
D	.aw/issues/open/enhancement-add-native-stdlib-ctypes-foreign-function-interfac.md
D	.aw/issues/open/enhancement-add-native-stdlib-curses-terminal-handling-for-cha.md
D	.aw/issues/open/enhancement-add-native-stdlib-dbm-interfaces-to-unix-databases.md
D	.aw/issues/open/enhancement-add-native-stdlib-dis-bytecode-disassembler.md
D	.aw/issues/open/enhancement-add-native-stdlib-doctest-test-interactive-example.md
D	.aw/issues/open/enhancement-add-native-stdlib-email-email-handling-package.md
D	.aw/issues/open/enhancement-add-native-stdlib-ensurepip-bootstrap-pip-installe.md
D	.aw/issues/open/enhancement-add-native-stdlib-faulthandler-dump-python-traceba.md
D	.aw/issues/open/enhancement-add-native-stdlib-fcntl-posix-file-control.md
D	.aw/issues/open/enhancement-add-native-stdlib-filecmp-file-and-directory-compa.md
D	.aw/issues/open/enhancement-add-native-stdlib-fileinput-iterate-over-lines-fro.md
D	.aw/issues/open/enhancement-add-native-stdlib-fnmatch-unix-filename-pattern-ma.md
D	.aw/issues/open/enhancement-add-native-stdlib-ftplib-ftp-protocol-client.md
D	.aw/issues/open/enhancement-add-native-stdlib-future-future-statement-definiti.md
D	.aw/issues/open/enhancement-add-native-stdlib-gc-garbage-collector-interface.md
D	.aw/issues/open/enhancement-add-native-stdlib-getopt-c-style-option-parser.md
D	.aw/issues/open/enhancement-add-native-stdlib-getpass-portable-password-input.md
D	.aw/issues/open/enhancement-add-native-stdlib-gettext-internationalization-ser.md
D	.aw/issues/open/enhancement-add-native-stdlib-graphlib-topological-sorting.md
D	.aw/issues/open/enhancement-add-native-stdlib-grp-group-database-posix.md
D	.aw/issues/open/enhancement-add-native-stdlib-imaplib-imap4-protocol-client.md
D	.aw/issues/open/enhancement-add-native-stdlib-importlib-import-machinery.md
D	.aw/issues/open/enhancement-add-native-stdlib-ipaddress-ipv4-ipv6-manipulation.md
D	.aw/issues/open/enhancement-add-native-stdlib-keyword-test-whether-string-is-a.md
D	.aw/issues/open/enhancement-add-native-stdlib-linecache-random-access-to-text.md
D	.aw/issues/open/enhancement-add-native-stdlib-mailbox-manipulate-mailboxes-in.md
D	.aw/issues/open/enhancement-add-native-stdlib-main-top-level-script-environmen.md
D	.aw/issues/open/enhancement-add-native-stdlib-mimetypes-map-filenames-to-mime.md
D	.aw/issues/open/enhancement-add-native-stdlib-mmap-memory-mapped-file-support.md
D	.aw/issues/open/enhancement-add-native-stdlib-modulefinder-find-modules-used-b.md
D	.aw/issues/open/enhancement-add-native-stdlib-multiprocessing-process-based-pa.md
D	.aw/issues/open/enhancement-add-native-stdlib-netrc-netrc-file-processing.md
D	.aw/issues/open/enhancement-add-native-stdlib-pdb-python-debugger.md
D	.aw/issues/open/enhancement-add-native-stdlib-pickletools-tools-for-pickle-dev.md
D	.aw/issues/open/enhancement-add-native-stdlib-pipes-interface-to-shell-pipelin.md
D	.aw/issues/open/enhancement-add-native-stdlib-pkgutil-package-extension-utilit.md
D	.aw/issues/open/enhancement-add-native-stdlib-plistlib-apple-plist-file-handli.md
D	.aw/issues/open/enhancement-add-native-stdlib-poplib-pop3-protocol-client.md
D	.aw/issues/open/enhancement-add-native-stdlib-posix-posix-system-calls.md
D	.aw/issues/open/enhancement-add-native-stdlib-posixpath-posix-pathname-manipul.md
D	.aw/issues/open/enhancement-add-native-stdlib-profile-cprofile-deterministic-p.md
D	.aw/issues/open/enhancement-add-native-stdlib-pstats-profiling-statistics.md
D	.aw/issues/open/enhancement-add-native-stdlib-pty-pseudo-terminal-utilities-po.md
D	.aw/issues/open/enhancement-add-native-stdlib-pwd-password-database-posix.md
D	.aw/issues/open/enhancement-add-native-stdlib-pyclbr-python-module-browser-sup.md
D	.aw/issues/open/enhancement-add-native-stdlib-quopri-encode-and-decode-mime-qu.md
D	.aw/issues/open/enhancement-add-native-stdlib-readline-gnu-readline-interface.md
D	.aw/issues/open/enhancement-add-native-stdlib-reprlib-alternate-repr-implement.md
D	.aw/issues/open/enhancement-add-native-stdlib-resource-resource-usage-posix.md
D	.aw/issues/open/enhancement-add-native-stdlib-rlcompleter-completion-function.md
D	.aw/issues/open/enhancement-add-native-stdlib-runpy-locating-and-running-pytho.md
D	.aw/issues/open/enhancement-add-native-stdlib-sched-event-scheduler.md
D	.aw/issues/open/enhancement-add-native-stdlib-select-i-o-completion-waiting.md
D	.aw/issues/open/enhancement-add-native-stdlib-selectors-high-level-i-o-multipl.md
D	.aw/issues/open/enhancement-add-native-stdlib-shelve-python-object-persistence.md
D	.aw/issues/open/enhancement-add-native-stdlib-site-site-specific-configuration.md
D	.aw/issues/open/enhancement-add-native-stdlib-smtplib-smtp-protocol-client.md
D	.aw/issues/open/enhancement-add-native-stdlib-socketserver-network-server-fram.md
D	.aw/issues/open/enhancement-add-native-stdlib-ssl-tls-ssl-wrapper-for-socket-o.md
D	.aw/issues/open/enhancement-add-native-stdlib-stat-interpreting-stat-results.md
D	.aw/issues/open/enhancement-add-native-stdlib-stringprep-internet-string-prepa.md
D	.aw/issues/open/enhancement-add-native-stdlib-sysconfig-access-python-s-config.md
D	.aw/issues/open/enhancement-add-native-stdlib-tabnanny-detection-of-ambiguous.md
D	.aw/issues/open/enhancement-add-native-stdlib-termios-posix-style-tty-control.md
D	.aw/issues/open/enhancement-add-native-stdlib-test-regression-test-support.md
D	.aw/issues/open/enhancement-add-native-stdlib-timeit-measure-execution-time.md
D	.aw/issues/open/enhancement-add-native-stdlib-token-constants-for-parse-tree-n.md
D	.aw/issues/open/enhancement-add-native-stdlib-tokenize-tokenizer-for-python-so.md
D	.aw/issues/open/enhancement-add-native-stdlib-tomllib-toml-parser-pep-680.md
D	.aw/issues/open/enhancement-add-native-stdlib-tracemalloc-trace-memory-allocat.md
D	.aw/issues/open/enhancement-add-native-stdlib-tty-terminal-control-functions.md
D	.aw/issues/open/enhancement-add-native-stdlib-types-dynamic-type-creation-util.md
D	.aw/issues/open/enhancement-add-native-stdlib-urllib-url-handling-modules.md
D	.aw/issues/open/enhancement-add-native-stdlib-venv-virtual-environment-creatio.md
D	.aw/issues/open/enhancement-add-native-stdlib-wave-read-and-write-wav-files.md
D	.aw/issues/open/enhancement-add-native-stdlib-webbrowser-convenient-web-browse.md
D	.aw/issues/open/enhancement-add-native-stdlib-wsgiref-wsgi-utilities-and-refer.md
D	.aw/issues/open/enhancement-add-native-stdlib-xmlrpc-xml-rpc-client-and-server.md
D	.aw/issues/open/enhancement-add-native-stdlib-zipapp-manage-executable-python.md
D	.aw/issues/open/enhancement-add-native-stdlib-zipimport-import-modules-from-zi.md
D	.aw/issues/open/enhancement-add-native-stdlib-zoneinfo-iana-time-zone-support.md
D	.aw/issues/open/enhancement-add-pagination-controls-to-all-list-pages.md
D	.aw/issues/open/enhancement-add-project-paths-codegen-path-mapping-in-config-t.md
D	.aw/issues/open/enhancement-add-real-time-updates-via-sse-websocket-for-pipeli.md
D	.aw/issues/open/enhancement-add-trace-screenshot-defaults-to-e2e-playwright-co.md
D	.aw/issues/open/enhancement-all-support-control-from-x-import-exports.md
D	.aw/issues/open/enhancement-artifact-kind-tabs-on-projectdetail-issues-changes.md
D	.aw/issues/open/enhancement-async-features-async-for-async-with-async-generato.md
D	.aw/issues/open/enhancement-auth-ui-login-page-protected-routes-user-menu.md
D	.aw/issues/open/enhancement-authorization-middleware-role-based-route-guards.md
D	.aw/issues/open/enhancement-brd-notation-v0-dogfood-score-and-conductor.md
D	.aw/issues/open/enhancement-c-extension-compatibility-load-cpython-c-extension.md
D	.aw/issues/open/enhancement-collection-operations-2-3x-slower-than-cpython.md
D	.aw/issues/open/enhancement-comments-schema-with-author-threading.md
D	.aw/issues/open/enhancement-compile-builtin-compile-source-to-code-object.md
D	.aw/issues/open/enhancement-complete-missing-state-machine-specs-for-changes-a.md
D	.aw/issues/open/enhancement-conductor-be-fe-project-invite-flow-link-based.md
D	.aw/issues/open/enhancement-contribution-audit-fields-created-by-triggered-by.md
D	.aw/issues/open/enhancement-create-cclab-pipeline-dag-visualization-component.md
D	.aw/issues/open/enhancement-create-cclab-pipeline-package-dag-visualization-no.md
D	.aw/issues/open/enhancement-create-cclab-search-component.md
D	.aw/issues/open/enhancement-create-cclab-spec-viewer-package-markdown-mermaid.md
D	.aw/issues/open/enhancement-define-7-artifact-kinds-in-cclab-sdd-vocabulary.md
D	.aw/issues/open/enhancement-define-workflow-dags-in-yaml-project-spec-gen-chan.md
D	.aw/issues/open/enhancement-deprecate-standalone-artifacts-page-artifacts-are.md
D	.aw/issues/open/enhancement-design-mockup-notation-external-tool-reference-pro.md
D	.aw/issues/open/enhancement-error-diagnostics-quality-rich-compiler-messages-w.md
D	.aw/issues/open/enhancement-establish-codegen-pipeline-from-cclab-agkit-schema.md
D	.aw/issues/open/enhancement-export-router-class-from-cclab-api.md
D	.aw/issues/open/enhancement-expose-local-compute-modules-as-mcp-tools-lint-typ.md
D	.aw/issues/open/enhancement-extend-mock-backend-to-support-full-user-journey-d.md
D	.aw/issues/open/enhancement-extract-plan-viewer-dashboard-ui-into-packages-ccl.md
D	.aw/issues/open/enhancement-flow-sensitive-taint-analysis-replace-pattern-matc.md
D	.aw/issues/open/enhancement-frontend-codegen-wireframe-component-design-token.md
D	.aw/issues/open/enhancement-gen-code-gen-diff-gen-parse-spec-driven-code-gener.md
D	.aw/issues/open/enhancement-generalize-cclab-agent-specstore-trait-with-kind-i.md
D	.aw/issues/open/enhancement-generator-state-machine-rewrite-5x-slower-than-cpy.md
D	.aw/issues/open/enhancement-gevent-greenlet-compatibility-coroutine-based-conc.md
D	.aw/issues/open/enhancement-grpc-compatibility-grpcio-protobuf-support.md
D	.aw/issues/open/enhancement-impl-phase-must-verify-code-against-spec-not-just.md
D	.aw/issues/open/enhancement-implement-change-spec-gen-workflow-and-status-stat.md
D	.aw/issues/open/enhancement-implement-stage-should-auto-run-lint-type-check-as.md
D	.aw/issues/open/enhancement-incremental-compilation-and-module-caching.md
D	.aw/issues/open/enhancement-index-server-scoped-toolchain-binding-auto-discove.md
D	.aw/issues/open/enhancement-inline-editing-add-visual-edit-mode-indicator-and.md
D	.aw/issues/open/enhancement-interprocedural-pdg-impact-slice-taint-must-cross.md
D	.aw/issues/open/enhancement-investigation-spike-workflow-for-exploratory-debug.md
D	.aw/issues/open/enhancement-issue-detail-wire-start-agent-button-to-pipeline-t.md
D	.aw/issues/open/enhancement-language-server-protocol-lsp-for-mamba.md
D	.aw/issues/open/enhancement-lineage-ui-frontend-graph-navigating-artifact-inpu.md
D	.aw/issues/open/enhancement-list-dict-generator-class-runtime-data-structure-o.md
D	.aw/issues/open/enhancement-package-json-exports-subpath-resolution.md
D	.aw/issues/open/enhancement-pep-695-type-parameter-syntax-full-generics-suppor.md
D	.aw/issues/open/enhancement-per-kind-state-machine-on-top-of-cclab-queue-workf.md
D	.aw/issues/open/enhancement-pg-artifactstore-impl-migrate-issuedb-projectspecd.md
D	.aw/issues/open/enhancement-prd-notation-v0-dogfood-score-and-conductor.md
D	.aw/issues/open/enhancement-programmatic-dev-server-api-for-test-harness-integ.md
D	.aw/issues/open/enhancement-project-members-management-page.md
D	.aw/issues/open/enhancement-project-settings-add-member-management-ui.md
D	.aw/issues/open/enhancement-prompts-should-guide-agents-to-use-local-compute-t.md
D	.aw/issues/open/enhancement-py3-12-conformance-data-structure-ops-list-dict-se.md
D	.aw/issues/open/enhancement-py3-12-conformance-exception-hierarchy.md
D	.aw/issues/open/enhancement-py3-12-conformance-gc-cycle-detection-reference-co.md
D	.aw/issues/open/enhancement-py3-12-conformance-generator-iterator-protocol.md
D	.aw/issues/open/enhancement-py3-12-conformance-stdlib-api-signatures-return-ty.md
D	.aw/issues/open/enhancement-real-module-import-system.md
D	.aw/issues/open/enhancement-repl-interactive-mamba-shell.md
D	.aw/issues/open/enhancement-replace-self-written-pipeline-executor-with-cclab.md
D	.aw/issues/open/enhancement-runbook-slo-alert-postmortem-notation-compose-prom.md
D	.aw/issues/open/enhancement-spec-code-alignment-tracking-version-code-drift-de.md
D	.aw/issues/open/enhancement-test-generation-from-requirementplus-specs.md
D	.aw/issues/open/enhancement-type-3-arg-dynamic-class-creation-type-name-bases.md
D	.aw/issues/open/enhancement-unified-import-graph-for-ts-rust-parity-with-pytho.md
D	.aw/issues/open/enhancement-upgrade-fillback-canonical-structure-agent-friendl.md
D	.aw/issues/open/enhancement-user-identity-model-session-auth.md
D	.aw/issues/open/enhancement-wasm-playground-browser-based-mamba-compiler-demo.md
D	.aw/issues/open/epic-authentication-authorization-zero-spec-zero-implem.md
D	.aw/issues/open/epic-background-job-engine-with-status-tracking-retry-c.md
D	.aw/issues/open/epic-module-federation-config-container-manifest-shared.md
D	.aw/issues/open/epic-mui-foundation-migration-replace-tailwind-primitiv.md
D	.aw/issues/open/epic-multi-role-collaboration-foundation.md
D	.aw/issues/open/refactor-audit-cclab-sdd-crate-contents-and-produce-file-le.md
D	.aw/issues/open/refactor-extract-code-intelligence-modules-into-cclab-compa.md
D	.aw/issues/open/refactor-implement-cpython-3-12-reference-counting-in-jit-c.md
D	.aw/issues/open/refactor-remove-conductor-specific-migration-code-from-ccla.md
D	.aw/issues/open/refactor-replace-be-src-agents-with-cclab-agent-agents.md
D	.aw/issues/open/refactor-shrink-cclab-sdd-crate-to-sdd-vocabulary-only.md
D	.aw/issues/open/test-performance-benchmarks-mamba-vs-cpython-3-12.md
M	Cargo.lock
M	Cargo.toml
M	crates/sdd/src/issues/backends/local.rs
M	crates/sdd/src/issues/types.rs
M	projects/agentic-workflow/cli/src/issues.rs
M	projects/agentic-workflow/cli/templates/mainthread/skills/score-monitor-idle/SKILL.md
M	pyproject.toml
```

## Diff Statistics

```
.claude/skills/score-monitor-idle/SKILL.md         | 114 ++---------
 ...ambaconfig-structs-driver-config-rs-vs-confi.md |   2 +-
 ...p-class-to-cclab-api-mamba-for-high-level-se.md |   3 +-
 ...tetime-utilities-parsing-timezone-arithmetic.md |   3 +-
 ...neral-purpose-model-validation-pydantic-like.md |   3 +-
 .../enhancement-add-missing-ml-algorithms.md       |   3 +-
 .../enhancement-add-retry-backoff-utilities.md     |   3 +-
 ...-sqlalchemy-compatible-declarative-orm-layer.md |   3 +-
 ...tatistical-annotations-and-dual-axis-support.md |   3 +-
 ...ent-add-yaml-toml-parsing-and-dotenv-loading.md |   3 +-
 .../enhancement-audio-processing-and-file-i-o.md   |   3 +-
 ...tils-extended-lri-thresholdcounter-cachedpro.md |   3 +-
 ...-runtime-core-shared-runtime-layer-for-cclab.md |   3 +-
 ...ils-orderedmultidict-frozendict-onetoone-sub.md |   3 +-
 ...t-basemodel-field-from-cclab-schema-not-just.md |   3 +-
 ...nt-expose-dl-layers-and-optimizers-to-python.md |   3 +-
 ...ent-expose-existing-rust-ml-models-to-python.md |   3 +-
 ...stem-storage-impl-of-generalized-artifactsto.md |   3 +-
 ...tils-atomic-save-fileperms-mkdir-p-iter-find.md |   3 +-
 ...utils-tokenize-format-str-get-format-args-de.md |   3 +-
 .../closed/enhancement-implement-png-pdf-export.md |   3 +-
 ...authoring-notation-agent-for-rough-idea-well.md |   3 +-
 ...ource-batch-fetch-webhook-receiver-for-issue.md |   3 +-
 ...tils-extended-bucketize-same-split-remap-get.md |   3 +-
 ...ils-jsonliterator-reverse-iter-lines-cclab-s.md |   3 +-
 ...hutils-clamp-bits-bit-string-array-cclab-sci.md |   3 +-
 ...untime-inference-engine-eliminate-ml-depende.md |   3 +-
 ...ils-augpath-shrinkuser-expandpath-cclab-util.md |   3 +-
 ...orm-adapter-format-conversion-comment-create.md |   3 +-
 ...ative-modules-not-loadable-so-build-system-b.md |   3 +-
 ...tils-heappriorityqueue-sortedpriorityqueue-c.md |   3 +-
 ...-silent-importerror-swallowing-in-init-py-fi.md |   3 +-
 ...ls-indexedset-with-ordering-indexing-and-set.md |   3 +-
 ...utils-stats-class-histogram-trimean-kurtosis.md |   3 +-
 ...ls-slugify-case-conversion-pluralize-asciify.md |   3 +-
 ...tils-table-type-with-to-text-to-html-from-di.md |   3 +-
 ...s-tracebackinfo-parsedexception-contextualca.md |   3 +-
 ...ils-extended-daterange-isoparse-parse-timede.md |   3 +-
 ...ng-rust-utility-crate-boltons-inspired-batte.md |   3 +-
 ...ils-make-sentinel-classproperty-get-all-subc.md |   3 +-
 ...-python-stubs-and-expose-missing-chart-types.md |   3 +-
 ...ils-url-class-find-all-links-parse-qsl-quote.md |   3 +-
 ...ent-video-codec-and-container-format-support.md |   3 +-
 ...gn-cclab-python-api-to-ecosystem-conventions.md |   3 +-
 ...ic-align-with-cclab-sdd-as-cloud-sdd-runtime.md |   3 +-
 ...tor-rewires-onto-arsenal-crates-extends-1049.md |   3 +-
 ...reate-projects-score-local-cli-sdd-show-case.md |   4 +-
 ...ose-cclab-sdd-library-multi-kind-generalizat.md |   3 +-
 ...-mamba-concurrency-semantics-no-gil-thread-s.md | 158 +++++++++++----
 ...ain-first-monorepo-layout-libs-apps-platform.md |   3 +-
 ...-notation-for-non-code-artifact-kinds-brd-pr.md |   3 +-
 ...py3-12-conformance-tracking-session-progress.md |  80 ++------
 ...tor-to-thin-shell-extract-all-logic-to-cclab.md |   3 +-
 ...st-change-operations-workflow-monitor-detect.md |   3 +-
 ...dd-pre-change-planning-workflow-goals-issues.md |   3 +-
 ...ng-c-library-ecosystem-compatibility-strateg.md |  92 ++++-----
 ...ng-mamba-language-features-tooling-completen.md | 121 +++++-------
 .../epic-tracking-mamba-package-manager-uv-like.md |  81 +++-----
 ...cking-mamba-py3-12-conformance-test-coverage.md |  63 ++----
 ...king-mamba-stdlib-native-rust-implementation.md |  71 ++-----
 ...facing-documentation-generation-project-type.md |   3 +-
 ...-extract-api-cli-commands-into-cclab-api-cli.md |   3 +-
 ...or-extract-kv-cli-commands-into-cclab-kv-cli.md |   3 +-
 ...or-extract-qc-cli-commands-into-cclab-qc-cli.md |   3 +-
 ...ract-queue-cli-commands-into-cclab-queue-cli.md |   3 +-
 ...ract-razer-cli-commands-into-cclab-razer-cli.md |   3 +-
 ...extract-sdd-cloud-agents-into-separate-crate.md |   3 +-
 ...init-missing-5-skill-templates-handoff-takeo.md |  63 ------
 ...dvanced-filtering-for-issues-priority-labels.md |  25 ---
 ...lobal-toast-notification-system-mui-snackbar.md |  26 ---
 ...teraction-diagrams-for-fe-be-async-flows-sse.md |  48 -----
 ...t-test-cli-command-invoke-playwright-from-pr.md |  39 ----
 ...nt-add-mark-namespace-and-raises-to-cclab-qc.md |  21 --
 ...t-add-native-stdlib-ast-abstract-syntax-tree.md |  43 ----
 ...ent-add-native-stdlib-bdb-debugger-framework.md |  43 ----
 ...tive-stdlib-binascii-binary-ascii-conversion.md |  43 ----
 ...tive-stdlib-builtins-built-in-objects-module.md |  43 ----
 ...tive-stdlib-cmd-line-oriented-command-interp.md |  43 ----
 ...tive-stdlib-codecs-codec-registry-and-base-c.md |  43 ----
 ...tive-stdlib-collections-abc-abstract-base-cl.md |  43 ----
 ...tive-stdlib-colorsys-color-system-conversion.md |  43 ----
 ...tive-stdlib-concurrent-futures-async-executi.md |  43 ----
 ...ative-stdlib-contextvars-context-local-state.md |  43 ----
 ...tive-stdlib-ctypes-foreign-function-interfac.md |  43 ----
 ...tive-stdlib-curses-terminal-handling-for-cha.md |  43 ----
 ...tive-stdlib-dbm-interfaces-to-unix-databases.md |  43 ----
 ...-add-native-stdlib-dis-bytecode-disassembler.md |  43 ----
 ...tive-stdlib-doctest-test-interactive-example.md |  43 ----
 ...d-native-stdlib-email-email-handling-package.md |  43 ----
 ...tive-stdlib-ensurepip-bootstrap-pip-installe.md |  43 ----
 ...tive-stdlib-faulthandler-dump-python-traceba.md |  43 ----
 ...t-add-native-stdlib-fcntl-posix-file-control.md |  43 ----
 ...tive-stdlib-filecmp-file-and-directory-compa.md |  43 ----
 ...tive-stdlib-fileinput-iterate-over-lines-fro.md |  43 ----
 ...tive-stdlib-fnmatch-unix-filename-pattern-ma.md |  43 ----
 ...add-native-stdlib-ftplib-ftp-protocol-client.md |  43 ----
 ...tive-stdlib-future-future-statement-definiti.md |  43 ----
 ...native-stdlib-gc-garbage-collector-interface.md |  43 ----
 ...d-native-stdlib-getopt-c-style-option-parser.md |  43 ----
 ...ative-stdlib-getpass-portable-password-input.md |  43 ----
 ...tive-stdlib-gettext-internationalization-ser.md |  43 ----
 ...d-native-stdlib-graphlib-topological-sorting.md |  43 ----
 ...t-add-native-stdlib-grp-group-database-posix.md |  43 ----
 ...-native-stdlib-imaplib-imap4-protocol-client.md |  43 ----
 ...add-native-stdlib-importlib-import-machinery.md |  43 ----
 ...tive-stdlib-ipaddress-ipv4-ipv6-manipulation.md |  43 ----
 ...tive-stdlib-keyword-test-whether-string-is-a.md |  43 ----
 ...ative-stdlib-linecache-random-access-to-text.md |  43 ----
 ...ative-stdlib-mailbox-manipulate-mailboxes-in.md |  43 ----
 ...tive-stdlib-main-top-level-script-environmen.md |  43 ----
 ...ative-stdlib-mimetypes-map-filenames-to-mime.md |  43 ----
 ...ative-stdlib-mmap-memory-mapped-file-support.md |  43 ----
 ...tive-stdlib-modulefinder-find-modules-used-b.md |  43 ----
 ...tive-stdlib-multiprocessing-process-based-pa.md |  43 ----
 ...dd-native-stdlib-netrc-netrc-file-processing.md |  43 ----
 ...cement-add-native-stdlib-pdb-python-debugger.md |  43 ----
 ...tive-stdlib-pickletools-tools-for-pickle-dev.md |  43 ----
 ...tive-stdlib-pipes-interface-to-shell-pipelin.md |  43 ----
 ...tive-stdlib-pkgutil-package-extension-utilit.md |  43 ----
 ...tive-stdlib-plistlib-apple-plist-file-handli.md |  43 ----
 ...dd-native-stdlib-poplib-pop3-protocol-client.md |  43 ----
 ...t-add-native-stdlib-posix-posix-system-calls.md |  43 ----
 ...tive-stdlib-posixpath-posix-pathname-manipul.md |  43 ----
 ...tive-stdlib-profile-cprofile-deterministic-p.md |  43 ----
 ...dd-native-stdlib-pstats-profiling-statistics.md |  43 ----
 ...tive-stdlib-pty-pseudo-terminal-utilities-po.md |  43 ----
 ...dd-native-stdlib-pwd-password-database-posix.md |  43 ----
 ...tive-stdlib-pyclbr-python-module-browser-sup.md |  43 ----
 ...tive-stdlib-quopri-encode-and-decode-mime-qu.md |  43 ----
 ...ative-stdlib-readline-gnu-readline-interface.md |  43 ----
 ...tive-stdlib-reprlib-alternate-repr-implement.md |  43 ----
 ...-native-stdlib-resource-resource-usage-posix.md |  43 ----
 ...ative-stdlib-rlcompleter-completion-function.md |  43 ----
 ...tive-stdlib-runpy-locating-and-running-pytho.md |  43 ----
 ...ment-add-native-stdlib-sched-event-scheduler.md |  43 ----
 ...-native-stdlib-select-i-o-completion-waiting.md |  43 ----
 ...tive-stdlib-selectors-high-level-i-o-multipl.md |  43 ----
 ...tive-stdlib-shelve-python-object-persistence.md |  43 ----
 ...tive-stdlib-site-site-specific-configuration.md |  43 ----
 ...d-native-stdlib-smtplib-smtp-protocol-client.md |  43 ----
 ...tive-stdlib-socketserver-network-server-fram.md |  43 ----
 ...tive-stdlib-ssl-tls-ssl-wrapper-for-socket-o.md |  43 ----
 ...native-stdlib-stat-interpreting-stat-results.md |  43 ----
 ...tive-stdlib-stringprep-internet-string-prepa.md |  43 ----
 ...tive-stdlib-sysconfig-access-python-s-config.md |  43 ----
 ...ative-stdlib-tabnanny-detection-of-ambiguous.md |  43 ----
 ...ative-stdlib-termios-posix-style-tty-control.md |  43 ----
 ...d-native-stdlib-test-regression-test-support.md |  43 ----
 ...-native-stdlib-timeit-measure-execution-time.md |  43 ----
 ...tive-stdlib-token-constants-for-parse-tree-n.md |  43 ----
 ...tive-stdlib-tokenize-tokenizer-for-python-so.md |  43 ----
 ...dd-native-stdlib-tomllib-toml-parser-pep-680.md |  43 ----
 ...tive-stdlib-tracemalloc-trace-memory-allocat.md |  43 ----
 ...native-stdlib-tty-terminal-control-functions.md |  43 ----
 ...tive-stdlib-types-dynamic-type-creation-util.md |  43 ----
 ...dd-native-stdlib-urllib-url-handling-modules.md |  43 ----
 ...tive-stdlib-venv-virtual-environment-creatio.md |  43 ----
 ...-native-stdlib-wave-read-and-write-wav-files.md |  43 ----
 ...tive-stdlib-webbrowser-convenient-web-browse.md |  43 ----
 ...tive-stdlib-wsgiref-wsgi-utilities-and-refer.md |  43 ----
 ...tive-stdlib-xmlrpc-xml-rpc-client-and-server.md |  43 ----
 ...ative-stdlib-zipapp-manage-executable-python.md |  43 ----
 ...tive-stdlib-zipimport-import-modules-from-zi.md |  43 ----
 ...ative-stdlib-zoneinfo-iana-time-zone-support.md |  43 ----
 ...nt-add-pagination-controls-to-all-list-pages.md |  26 ---
 ...oject-paths-codegen-path-mapping-in-config-t.md |  74 -------
 ...al-time-updates-via-sse-websocket-for-pipeli.md |  22 ---
 ...ace-screenshot-defaults-to-e2e-playwright-co.md |  35 ----
 ...nt-all-support-control-from-x-import-exports.md |  50 -----
 ...ct-kind-tabs-on-projectdetail-issues-changes.md |  55 ------
 ...features-async-for-async-with-async-generato.md |  58 ------
 ...uth-ui-login-page-protected-routes-user-menu.md |  63 ------
 ...orization-middleware-role-based-route-guards.md |  86 --------
 ...-brd-notation-v0-dogfood-score-and-conductor.md |  82 --------
 ...nsion-compatibility-load-cpython-c-extension.md |  58 ------
 ...llection-operations-2-3x-slower-than-cpython.md |  88 ---------
 ...cement-comments-schema-with-author-threading.md |  78 --------
 ...ompile-builtin-compile-source-to-code-object.md |  63 ------
 ...te-missing-state-machine-specs-for-changes-a.md |  25 ---
 ...nductor-be-fe-project-invite-flow-link-based.md |  76 -------
 ...ibution-audit-fields-created-by-triggered-by.md |  61 ------
 ...e-cclab-pipeline-dag-visualization-component.md |  25 ---
 ...-cclab-pipeline-package-dag-visualization-no.md |  66 -------
 .../enhancement-create-cclab-search-component.md   |  22 ---
 ...e-cclab-spec-viewer-package-markdown-mermaid.md |  63 ------
 ...ine-7-artifact-kinds-in-cclab-sdd-vocabulary.md |  45 -----
 ...-workflow-dags-in-yaml-project-spec-gen-chan.md |  54 -----
 ...cate-standalone-artifacts-page-artifacts-are.md |  23 ---
 ...-mockup-notation-external-tool-reference-pro.md |  85 --------
 ...diagnostics-quality-rich-compiler-messages-w.md |  60 ------
 ...ish-codegen-pipeline-from-cclab-agkit-schema.md | 124 ------------
 ...hancement-export-router-class-from-cclab-api.md |  19 --
 ...-local-compute-modules-as-mcp-tools-lint-typ.md |  91 ---------
 ...-mock-backend-to-support-full-user-journey-d.md |  47 -----
 ...t-plan-viewer-dashboard-ui-into-packages-ccl.md |  99 ----------
 ...ensitive-taint-analysis-replace-pattern-matc.md |  61 ------
 ...end-codegen-wireframe-component-design-token.md |  76 -------
 ...de-gen-diff-gen-parse-spec-driven-code-gener.md |  61 ------
 ...lize-cclab-agent-specstore-trait-with-kind-i.md |  83 --------
 ...tor-state-machine-rewrite-5x-slower-than-cpy.md |  87 --------
 ...-greenlet-compatibility-coroutine-based-conc.md |  58 ------
 ...t-grpc-compatibility-grpcio-protobuf-support.md |  57 ------
 ...phase-must-verify-code-against-spec-not-just.md |  47 -----
 ...ent-change-spec-gen-workflow-and-status-stat.md |  23 ---
 ...ent-stage-should-auto-run-lint-type-check-as.md |  61 ------
 ...t-incremental-compilation-and-module-caching.md |  54 -----
 ...server-scoped-toolchain-binding-auto-discove.md |  78 --------
 ...e-editing-add-visual-edit-mode-indicator-and.md |  22 ---
 ...procedural-pdg-impact-slice-taint-must-cross.md |  67 -------
 ...igation-spike-workflow-for-exploratory-debug.md |  88 ---------
 ...detail-wire-start-agent-button-to-pipeline-t.md |  26 ---
 ...ement-language-server-protocol-lsp-for-mamba.md |  50 -----
 ...e-ui-frontend-graph-navigating-artifact-inpu.md |  54 -----
 ...ict-generator-class-runtime-data-structure-o.md |  98 ---------
 ...ment-package-json-exports-subpath-resolution.md |  30 ---
 ...5-type-parameter-syntax-full-generics-suppor.md |  61 ------
 ...nd-state-machine-on-top-of-cclab-queue-workf.md |  71 -------
 ...ifactstore-impl-migrate-issuedb-projectspecd.md |  88 ---------
 ...-prd-notation-v0-dogfood-score-and-conductor.md | 121 ------------
 ...mmatic-dev-server-api-for-test-harness-integ.md |  45 -----
 .../enhancement-project-members-management-page.md |  59 ------
 ...nt-project-settings-add-member-management-ui.md |  23 ---
 ...s-should-guide-agents-to-use-local-compute-t.md | 162 ---------------
 ...-conformance-data-structure-ops-list-dict-se.md | 106 ----------
 ...ement-py3-12-conformance-exception-hierarchy.md | 106 ----------
 ...-conformance-gc-cycle-detection-reference-co.md | 123 ------------
 ...3-12-conformance-generator-iterator-protocol.md | 124 ------------
 ...-conformance-stdlib-api-signatures-return-ty.md | 114 -----------
 .../open/enhancement-real-module-import-system.md  |  69 -------
 .../enhancement-repl-interactive-mamba-shell.md    |  74 -------
 ...ce-self-written-pipeline-executor-with-cclab.md |  56 ------
 ...k-slo-alert-postmortem-notation-compose-prom.md | 125 ------------
 ...ode-alignment-tracking-version-code-drift-de.md |  82 --------
 ...t-test-generation-from-requirementplus-specs.md |  66 -------
 ...3-arg-dynamic-class-creation-type-name-bases.md |  71 -------
 ...d-import-graph-for-ts-rust-parity-with-pytho.md |  55 ------
 ...e-fillback-canonical-structure-agent-friendl.md |  47 -----
 ...enhancement-user-identity-model-session-auth.md |  87 --------
 ...playground-browser-based-mamba-compiler-demo.md |  49 -----
 ...tication-authorization-zero-spec-zero-implem.md |  28 ---
 ...ound-job-engine-with-status-tracking-retry-c.md |  68 -------
 ...-federation-config-container-manifest-shared.md |  49 -----
 ...undation-migration-replace-tailwind-primitiv.md |  33 ----
 .../epic-multi-role-collaboration-foundation.md    |  80 --------
 ...cclab-sdd-crate-contents-and-produce-file-le.md |  53 -----
 ...t-code-intelligence-modules-into-cclab-compa.md | 102 ----------
 ...ent-cpython-3-12-reference-counting-in-jit-c.md | 171 ----------------
 ...-conductor-specific-migration-code-from-ccla.md |  42 ----
 ...eplace-be-src-agents-with-cclab-agent-agents.md |  50 -----
 ...hrink-cclab-sdd-crate-to-sdd-vocabulary-only.md |  46 -----
 ...performance-benchmarks-mamba-vs-cpython-3-12.md | 118 -----------
 Cargo.lock                                         | 154 +++++++--------
 Cargo.toml                                         |   2 +-
 crates/sdd/src/issues/backends/local.rs            |  95 +++++++++
 crates/sdd/src/issues/types.rs                     |  95 +++++++++
 projects/agentic-workflow/cli/src/issues.rs                   | 219 ++++++++++++++-------
 .../mainthread/skills/score-monitor-idle/SKILL.md  | 114 ++---------
 pyproject.toml                                     |   2 +-
 258 files changed, 769 insertions(+), 10832 deletions(-)
```

## Diff

```diff
diff --git a/.claude/skills/score-monitor-idle/SKILL.md b/.claude/skills/score-monitor-idle/SKILL.md
index 5431e0bb..3f04d7b9 100644
--- a/.claude/skills/score-monitor-idle/SKILL.md
+++ b/.claude/skills/score-monitor-idle/SKILL.md
@@ -6,9 +6,7 @@ user-invocable: true
 
 # /aw:monitor-idle
 
-Safety net for the issue CRR (Create → Review → Revise) loop. Runs `aw wi idle` on a cron, reads the recommendations the CLI emits per worktree, and re-dispatches whatever the next step is so a stalled chain self-heals without user intervention.
-
-Background — Claude Code 2.1.110 launches `Agent` dispatches asynchronously and async dispatches do not fire `SubagentStart` / `SubagentStop` hooks reliably (Anthropic open issue #27755). The issue CRR loop is now mainthread-orchestrated; this skill is the recovery path when mainthread itself is no longer in the loop (subagent crashed mid-write, user closed the session before validate ran, etc.).
+Safety net for the issue CRR (Create → Review → Revise) loop. Registers a cron that wakes mainthread to run `aw wi idle`. Mainthread handles the resulting batch envelope per `CLAUDE.md § Score CLI envelope (mainthread protocol)` — this skill owns nothing beyond cron registration.
 
 ## Arguments
 
@@ -23,105 +21,25 @@ Background — Claude Code 2.1.110 launches `Agent` dispatches asynchronously an
 | `interval` (start only) | no | `5m` | `5m`, `10m`, `15m` |
 | `job-id` (stop only) | yes | — | `cron-abc123` |
 
-## Instructions
+## Mode: `start`
 
-### Mode: `start`
+1. **Parse interval** (default `5m`) → cron expression:
+   - `Nm` → `*/N * * * *`
+   - `Nh` → `13 */N * * *` (offset to avoid noon stampede)
+2. **`CronCreate`** with `recurring: true` and prompt:
+   ```
+   Run: aw wi idle
+   ```
+3. **Report** to user: cron job ID, poll interval, 7-day auto-expiry, stop command.
 
-#### 1. Parse interval
+## Mode: `stop`
 
-- Default `5m` if omitted.
-- Convert to cron expression:
-  - `5m` → `*/5 * * * *`
-  - `10m` → `*/10 * * * *`
-  - `15m` → `*/15 * * * *`
-  - `30m` → `*/30 * * * *`
-  - `1h` → `13 * * * *` (offset to avoid noon stampede)
-  - Otherwise parse `Nm` → `*/N * * * *`, `Nh` → `13 */N * * *`.
-
-#### 2. Create the cron
-
-Use `CronCreate` with `recurring: true` and the prompt below verbatim:
-
-```
-You are the issue-idle watchdog. Do the following in order — do NOT skip steps.
-
-## Step 1: Scan for idle worktrees
-
-Run: aw wi idle
-
-The CLI emits a single JSON envelope on stdout, shape:
-
-  {"action":"batch","recommendations":[{...}, {...}]}
-
-Each recommendation has:
-  - slug: <issue slug>
-  - kind: "dispatch" | "invoke" | "skip"
-  - reason: short human-readable explanation
-  - agent: <agent type>            (only when kind = "dispatch")
-  - command: "aw wi ..."    (when kind = "dispatch" or "invoke")
-
-If `recommendations` is empty OR every entry has kind = "skip", report:
-  "No idle worktrees — all CRR chains are healthy or actively running."
-and STOP.
-
-## Step 2: Act on each non-skip recommendation
-
-For each recommendation r in order:
-
-- If r.kind == "skip": ignore.
-- If r.kind == "dispatch":
-    Construct an Agent call with the embedded envelope, e.g.:
-
-      Agent(
-        subagent_type=r.agent,
-        prompt="TASK: Resume issue \"<r.slug>\". Run the brief CLI yourself, do the work, run --apply, and STOP.\n\nENVELOPE: <r as JSON>"
-      )
-
-    Wait for the agent to return. Then immediately run the validate step
-    that mainthread normally runs after the agent exits:
-
-      aw wi validate <r.slug>
-
-    Parse the envelope it prints and follow CLAUDE.md § Score CLI envelope
-    (mainthread protocol) for the next step. Loop until you hit `done` or
-    `error`.
-
-- If r.kind == "invoke":
-    Run r.command directly (no Agent dispatch). Parse the envelope it
-    prints and follow CLAUDE.md § Score CLI envelope (mainthread protocol)
-    for the next step.
-
-## Step 3: Report
-
-After processing every recommendation, print a one-line summary per slug:
-  "<slug>: <kind> — <reason> → <outcome>"
-
-where outcome is the terminal envelope action (done, error, or "still
-running" if you handed off to a subagent that is still executing).
-```
-
-#### 3. Confirm to user
-
-After `CronCreate` returns the job ID, report:
-- Cron job ID (needed for `stop`)
-- Poll interval
-- Reminder: recurring jobs auto-expire after 7 days
-- Reminder: use `/aw:monitor-idle stop <job-id>` (or `CronDelete`) to cancel
-
-### Mode: `stop`
-
-If the first argument is `stop`:
-1. Take the second argument as `<job-id>` (if missing, run `CronList` and ask the user via `AskUserQuestion`).
-2. Call `CronDelete` with that ID.
+1. Take `<job-id>` arg (if missing, `CronList` then `AskUserQuestion`).
+2. `CronDelete` with that ID.
 3. Report deletion.
 
 ## Notes
 
-- This skill mutates nothing on its own — every action is delegated to
-  `score` CLI verbs which carry the source-of-truth (filesystem + git inside
-  the worktree).
-- `aw wi idle` itself is pure read-only — it never commits.
-- If the same recommendation fires repeatedly across multiple ticks (e.g.
-  the same `--apply` keeps failing), the `error` envelope from validate
-  will surface in your summary; investigate the worktree manually rather
-  than letting the cron loop hammer it.
+- The cron prompt is intentionally one line. Mainthread already knows how to handle `batch` / `dispatch` / `invoke` / `skip` envelopes — see `CLAUDE.md`. Duplicating that protocol here would drift.
+- `aw wi idle` is read-only; all commits flow through `aw wi validate` (CRRR sole commit point).
+- If the same dispatch fires every tick, the source-of-truth (worktree) is stuck — investigate manually rather than letting the cron hammer it.
diff --git a/.aw/issues/closed/bug-dual-mambaconfig-structs-driver-config-rs-vs-confi.md b/.aw/issues/closed/bug-dual-mambaconfig-structs-driver-config-rs-vs-confi.md
index 8082ac1a..e392cc0b 100644
--- a/.aw/issues/closed/bug-dual-mambaconfig-structs-driver-config-rs-vs-confi.md
+++ b/.aw/issues/closed/bug-dual-mambaconfig-structs-driver-config-rs-vs-confi.md
@@ -10,7 +10,7 @@ labels:
 - priority:p2
 - crate:mamba
 created_at: 2026-04-03T09:04:17Z
-updated_at: 2026-04-03T09:04:17Z
+updated_at: 2026-04-09T08:43:40Z
 ---
 
 ## Problem
diff --git a/.aw/issues/closed/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md b/.aw/issues/closed/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
index 7018da05..de317012 100644
--- a/.aw/issues/closed/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
+++ b/.aw/issues/closed/enhancement-add-app-class-to-cclab-api-mamba-for-high-level-se.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:mamba
 created_at: 2026-04-03T09:04:27Z
-updated_at: 2026-04-03T09:04:37Z
+updated_at: 2026-04-09T08:43:40Z
 ---
 
-
 ## Problem
 
 `cclab-api-mamba` exposes low-level primitives (`MbRouter`, `MbRequest`, `MbResponse`) but no `App` class. Conductor's `main.py` expects a high-level API:
diff --git a/.aw/issues/closed/enhancement-add-datetime-utilities-parsing-timezone-arithmetic.md b/.aw/issues/closed/enhancement-add-datetime-utilities-parsing-timezone-arithmetic.md
index 4c975b0a..3c2daac7 100644
--- a/.aw/issues/closed/enhancement-add-datetime-utilities-parsing-timezone-arithmetic.md
+++ b/.aw/issues/closed/enhancement-add-datetime-utilities-parsing-timezone-arithmetic.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:util
 created_at: 2026-03-12T07:00:35Z
-updated_at: 2026-03-12T07:00:35Z
+updated_at: 2026-04-09T08:44:40Z
 ---
 
-
 ## Context
 `cclab.util` has `naturaltime` / `naturaldelta` for humanization but lacks datetime parsing, timezone handling, and arithmetic. Developers still need `arrow` or `pendulum`.
 
diff --git a/.aw/issues/closed/enhancement-add-general-purpose-model-validation-pydantic-like.md b/.aw/issues/closed/enhancement-add-general-purpose-model-validation-pydantic-like.md
index 7a636f07..c136d929 100644
--- a/.aw/issues/closed/enhancement-add-general-purpose-model-validation-pydantic-like.md
+++ b/.aw/issues/closed/enhancement-add-general-purpose-model-validation-pydantic-like.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:schema
 created_at: 2026-03-12T07:00:35Z
-updated_at: 2026-03-12T07:00:35Z
+updated_at: 2026-04-09T08:44:41Z
 ---
 
-
 ## Context
 `cclab.schema` only has `BaseSettings` for env config. Developers still need `pip install pydantic` for request/response validation, data modeling, and serialization.
 
diff --git a/.aw/issues/closed/enhancement-add-missing-ml-algorithms.md b/.aw/issues/closed/enhancement-add-missing-ml-algorithms.md
index 6c861d07..7e2ad727 100644
--- a/.aw/issues/closed/enhancement-add-missing-ml-algorithms.md
+++ b/.aw/issues/closed/enhancement-add-missing-ml-algorithms.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:learn
 created_at: 2026-03-12T06:23:12Z
-updated_at: 2026-03-12T06:23:12Z
+updated_at: 2026-04-09T08:44:42Z
 ---
 
-
 ## Context
 Core ML algorithms missing from Rust implementation, compared to scikit-learn.
 
diff --git a/.aw/issues/closed/enhancement-add-retry-backoff-utilities.md b/.aw/issues/closed/enhancement-add-retry-backoff-utilities.md
index 550e4f0a..204a2f36 100644
--- a/.aw/issues/closed/enhancement-add-retry-backoff-utilities.md
+++ b/.aw/issues/closed/enhancement-add-retry-backoff-utilities.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:util
 created_at: 2026-03-12T07:00:35Z
-updated_at: 2026-03-12T07:00:35Z
+updated_at: 2026-04-09T08:44:40Z
 ---
 
-
 ## Context
 API calls, database connections, and external service calls commonly need retry logic. Developers currently `pip install tenacity` or `backoff`.
 
diff --git a/.aw/issues/closed/enhancement-add-sqlalchemy-compatible-declarative-orm-layer.md b/.aw/issues/closed/enhancement-add-sqlalchemy-compatible-declarative-orm-layer.md
index 107cbc32..984de573 100644
--- a/.aw/issues/closed/enhancement-add-sqlalchemy-compatible-declarative-orm-layer.md
+++ b/.aw/issues/closed/enhancement-add-sqlalchemy-compatible-declarative-orm-layer.md
@@ -9,10 +9,9 @@ labels:
 - crate:pg
 - priority:p1
 created_at: 2026-03-20T09:33:16Z
-updated_at: 2026-03-20T09:33:16Z
+updated_at: 2026-04-09T08:44:28Z
 ---
 
-
 **Parent**: #964
 
 Currently cclab.pg has Table + Column + QueryBuilder (low-level). Conductor needs declarative ORM:
diff --git a/.aw/issues/closed/enhancement-add-statistical-annotations-and-dual-axis-support.md b/.aw/issues/closed/enhancement-add-statistical-annotations-and-dual-axis-support.md
index 43e056a6..d604701f 100644
--- a/.aw/issues/closed/enhancement-add-statistical-annotations-and-dual-axis-support.md
+++ b/.aw/issues/closed/enhancement-add-statistical-annotations-and-dual-axis-support.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:plot
 created_at: 2026-03-12T06:22:59Z
-updated_at: 2026-03-12T06:22:59Z
+updated_at: 2026-04-09T08:44:42Z
 ---
 
-
 ## Context
 Missing features compared to matplotlib/plotly for data analysis use cases.
 
diff --git a/.aw/issues/closed/enhancement-add-yaml-toml-parsing-and-dotenv-loading.md b/.aw/issues/closed/enhancement-add-yaml-toml-parsing-and-dotenv-loading.md
index bf3e07b8..201f8cba 100644
--- a/.aw/issues/closed/enhancement-add-yaml-toml-parsing-and-dotenv-loading.md
+++ b/.aw/issues/closed/enhancement-add-yaml-toml-parsing-and-dotenv-loading.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:schema
 created_at: 2026-03-12T07:00:35Z
-updated_at: 2026-03-12T07:00:35Z
+updated_at: 2026-04-09T08:44:41Z
 ---
 
-
 ## Context
 Nearly every Python project needs YAML/TOML config parsing and .env file loading. Currently developers must `pip install pyyaml toml python-dotenv` separately.
 
diff --git a/.aw/issues/closed/enhancement-audio-processing-and-file-i-o.md b/.aw/issues/closed/enhancement-audio-processing-and-file-i-o.md
index 5233ae47..3b53a8fd 100644
--- a/.aw/issues/closed/enhancement-audio-processing-and-file-i-o.md
+++ b/.aw/issues/closed/enhancement-audio-processing-and-file-i-o.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:media
 created_at: 2026-03-12T06:23:33Z
-updated_at: 2026-03-12T06:23:33Z
+updated_at: 2026-04-09T08:44:44Z
 ---
 
-
 ## Context
 Audio support is minimal — only raw PCM extraction and channel deinterleaving. No file I/O or processing.
 
diff --git a/.aw/issues/closed/enhancement-cacheutils-extended-lri-thresholdcounter-cachedpro.md b/.aw/issues/closed/enhancement-cacheutils-extended-lri-thresholdcounter-cachedpro.md
index c1345b3a..cadddcdb 100644
--- a/.aw/issues/closed/enhancement-cacheutils-extended-lri-thresholdcounter-cachedpro.md
+++ b/.aw/issues/closed/enhancement-cacheutils-extended-lri-thresholdcounter-cachedpro.md
@@ -10,10 +10,9 @@ labels:
 - priority:p1
 - crate:util
 created_at: 2026-03-13T11:53:40Z
-updated_at: 2026-03-13T11:58:22Z
+updated_at: 2026-04-09T08:44:35Z
 ---
 
-
 ## Summary
 
 Extend existing `cclab-util/cache.rs` with additional caching primitives.
diff --git a/.aw/issues/closed/enhancement-cclab-runtime-core-shared-runtime-layer-for-cclab.md b/.aw/issues/closed/enhancement-cclab-runtime-core-shared-runtime-layer-for-cclab.md
index 344c21a5..24e81fc2 100644
--- a/.aw/issues/closed/enhancement-cclab-runtime-core-shared-runtime-layer-for-cclab.md
+++ b/.aw/issues/closed/enhancement-cclab-runtime-core-shared-runtime-layer-for-cclab.md
@@ -8,10 +8,9 @@ author: chrischeng-c4
 labels:
 - type:enhancement
 created_at: 2026-03-27T06:38:54Z
-updated_at: 2026-03-27T06:38:54Z
+updated_at: 2026-04-09T08:43:40Z
 ---
 
-
 ## Problem
 
 cclab-api (Quasar, Axum-based HTTP framework) and cclab-queue (Meteor, distributed task queue) are currently independent crates with no integration path. Two key use cases are blocked:
diff --git a/.aw/issues/closed/enhancement-dictutils-orderedmultidict-frozendict-onetoone-sub.md b/.aw/issues/closed/enhancement-dictutils-orderedmultidict-frozendict-onetoone-sub.md
index 4f2d940d..03d562a5 100644
--- a/.aw/issues/closed/enhancement-dictutils-orderedmultidict-frozendict-onetoone-sub.md
+++ b/.aw/issues/closed/enhancement-dictutils-orderedmultidict-frozendict-onetoone-sub.md
@@ -10,10 +10,9 @@ labels:
 - priority:p0
 - crate:collections
 created_at: 2026-03-13T11:53:03Z
-updated_at: 2026-03-13T11:58:56Z
+updated_at: 2026-04-09T08:44:37Z
 ---
 
-
 ## Summary
 
 Implement dictionary utility types missing from Rust/Python stdlib.
diff --git a/.aw/issues/closed/enhancement-export-basemodel-field-from-cclab-schema-not-just.md b/.aw/issues/closed/enhancement-export-basemodel-field-from-cclab-schema-not-just.md
index 4e3b76b4..82330215 100644
--- a/.aw/issues/closed/enhancement-export-basemodel-field-from-cclab-schema-not-just.md
+++ b/.aw/issues/closed/enhancement-export-basemodel-field-from-cclab-schema-not-just.md
@@ -9,10 +9,9 @@ labels:
 - crate:schema
 - priority:p0
 created_at: 2026-03-20T09:33:12Z
-updated_at: 2026-03-20T09:33:12Z
+updated_at: 2026-04-09T08:44:29Z
 ---
 
-
 **Parent**: #964
 
 ## Problem
diff --git a/.aw/issues/closed/enhancement-expose-dl-layers-and-optimizers-to-python.md b/.aw/issues/closed/enhancement-expose-dl-layers-and-optimizers-to-python.md
index ed1543ed..6f2c86ea 100644
--- a/.aw/issues/closed/enhancement-expose-dl-layers-and-optimizers-to-python.md
+++ b/.aw/issues/closed/enhancement-expose-dl-layers-and-optimizers-to-python.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:learn
 created_at: 2026-03-12T06:23:17Z
-updated_at: 2026-03-12T06:23:17Z
+updated_at: 2026-04-09T08:44:41Z
 ---
 
-
 ## Context
 Rust DL core has comprehensive layer/optimizer implementations but Python only exposes the Tensor class.
 
diff --git a/.aw/issues/closed/enhancement-expose-existing-rust-ml-models-to-python.md b/.aw/issues/closed/enhancement-expose-existing-rust-ml-models-to-python.md
index c6228039..5b8885ae 100644
--- a/.aw/issues/closed/enhancement-expose-existing-rust-ml-models-to-python.md
+++ b/.aw/issues/closed/enhancement-expose-existing-rust-ml-models-to-python.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:learn
 created_at: 2026-03-12T06:23:07Z
-updated_at: 2026-03-12T06:23:07Z
+updated_at: 2026-04-09T08:44:42Z
 ---
 
-
 ## Context
 Learn module is at 90%. Rust core has ~25+ classes but Python only exposes ~5 models + metrics.
 
diff --git a/.aw/issues/closed/enhancement-filesystem-storage-impl-of-generalized-artifactsto.md b/.aw/issues/closed/enhancement-filesystem-storage-impl-of-generalized-artifactsto.md
index a88dec19..e1af31fd 100644
--- a/.aw/issues/closed/enhancement-filesystem-storage-impl-of-generalized-artifactsto.md
+++ b/.aw/issues/closed/enhancement-filesystem-storage-impl-of-generalized-artifactsto.md
@@ -10,10 +10,9 @@ labels:
 - priority:p1
 - project:agentic-workflow
 created_at: 2026-04-05T10:33:57Z
-updated_at: 2026-04-05T10:33:57Z
+updated_at: 2026-04-09T08:43:36Z
 ---
 
-
 Part of #1157 — create Score project.
 
 ## Scope
diff --git a/.aw/issues/closed/enhancement-fileutils-atomic-save-fileperms-mkdir-p-iter-find.md b/.aw/issues/closed/enhancement-fileutils-atomic-save-fileperms-mkdir-p-iter-find.md
index dc1b5427..7ccefe96 100644
--- a/.aw/issues/closed/enhancement-fileutils-atomic-save-fileperms-mkdir-p-iter-find.md
+++ b/.aw/issues/closed/enhancement-fileutils-atomic-save-fileperms-mkdir-p-iter-find.md
@@ -10,10 +10,9 @@ labels:
 - priority:p1
 - crate:fs
 created_at: 2026-03-13T11:53:34Z
-updated_at: 2026-03-13T11:59:00Z
+updated_at: 2026-04-09T08:44:35Z
 ---
 
-
 ## Summary
 
 Filesystem utilities for safe file operations.
diff --git a/.aw/issues/closed/enhancement-formatutils-tokenize-format-str-get-format-args-de.md b/.aw/issues/closed/enhancement-formatutils-tokenize-format-str-get-format-args-de.md
index 7947fa72..5cc761de 100644
--- a/.aw/issues/closed/enhancement-formatutils-tokenize-format-str-get-format-args-de.md
+++ b/.aw/issues/closed/enhancement-formatutils-tokenize-format-str-get-format-args-de.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:text
 created_at: 2026-03-13T11:54:39Z
-updated_at: 2026-03-13T11:58:39Z
+updated_at: 2026-04-09T08:44:33Z
 ---
 
-
 ## Summary
 
 Format string introspection utilities.
diff --git a/.aw/issues/closed/enhancement-implement-png-pdf-export.md b/.aw/issues/closed/enhancement-implement-png-pdf-export.md
index f63fc5bb..2924a165 100644
--- a/.aw/issues/closed/enhancement-implement-png-pdf-export.md
+++ b/.aw/issues/closed/enhancement-implement-png-pdf-export.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:plot
 created_at: 2026-03-12T06:22:57Z
-updated_at: 2026-03-12T06:22:57Z
+updated_at: 2026-04-09T08:44:43Z
 ---
 
-
 ## Context
 Plot currently only exports SVG and HTML. PNG and PDF export are stubbed but return errors:
 - _"PNG export requires the 'resvg' feature (not yet available)"_
diff --git a/.aw/issues/closed/enhancement-issue-authoring-notation-agent-for-rough-idea-well.md b/.aw/issues/closed/enhancement-issue-authoring-notation-agent-for-rough-idea-well.md
index 9e1f3b5a..9c0d55ef 100644
--- a/.aw/issues/closed/enhancement-issue-authoring-notation-agent-for-rough-idea-well.md
+++ b/.aw/issues/closed/enhancement-issue-authoring-notation-agent-for-rough-idea-well.md
@@ -11,10 +11,9 @@ labels:
 - project:conductor
 - project:agentic-workflow
 created_at: 2026-04-05T10:36:39Z
-updated_at: 2026-04-05T10:36:39Z
+updated_at: 2026-04-09T08:43:34Z
 ---
 
-
 Part of #1159 — formal notation for non-code artifact kinds.
 
 ## Problem
diff --git a/.aw/issues/closed/enhancement-issuesource-batch-fetch-webhook-receiver-for-issue.md b/.aw/issues/closed/enhancement-issuesource-batch-fetch-webhook-receiver-for-issue.md
index 5f706b4a..2d9dd13e 100644
--- a/.aw/issues/closed/enhancement-issuesource-batch-fetch-webhook-receiver-for-issue.md
+++ b/.aw/issues/closed/enhancement-issuesource-batch-fetch-webhook-receiver-for-issue.md
@@ -10,10 +10,9 @@ labels:
 - crate:agent
 - priority:p0
 created_at: 2026-03-18T02:53:10Z
-updated_at: 2026-03-18T02:53:10Z
+updated_at: 2026-04-09T08:44:32Z
 ---
 
-
 ## Summary
 
 Abstraction for how issues enter the system. Supports both pull (batch fetch) and push (webhook) modes. Agent and Adapter don't care which mode is used.
diff --git a/.aw/issues/closed/enhancement-iterutils-extended-bucketize-same-split-remap-get.md b/.aw/issues/closed/enhancement-iterutils-extended-bucketize-same-split-remap-get.md
index 93b6d157..fc8b2adf 100644
--- a/.aw/issues/closed/enhancement-iterutils-extended-bucketize-same-split-remap-get.md
+++ b/.aw/issues/closed/enhancement-iterutils-extended-bucketize-same-split-remap-get.md
@@ -10,10 +10,9 @@ labels:
 - priority:p0
 - crate:util
 created_at: 2026-03-13T11:53:29Z
-updated_at: 2026-03-13T11:58:19Z
+updated_at: 2026-04-09T08:44:36Z
 ---
 
-
 ## Summary
 
 Extend existing `cclab-util/iter.rs` with additional iteration utilities.
diff --git a/.aw/issues/closed/enhancement-jsonutils-jsonliterator-reverse-iter-lines-cclab-s.md b/.aw/issues/closed/enhancement-jsonutils-jsonliterator-reverse-iter-lines-cclab-s.md
index 6fe2ac14..0cd653fc 100644
--- a/.aw/issues/closed/enhancement-jsonutils-jsonliterator-reverse-iter-lines-cclab-s.md
+++ b/.aw/issues/closed/enhancement-jsonutils-jsonliterator-reverse-iter-lines-cclab-s.md
@@ -10,10 +10,9 @@ labels:
 - crate:schema
 - priority:p2
 created_at: 2026-03-13T11:54:17Z
-updated_at: 2026-03-13T11:58:34Z
+updated_at: 2026-04-09T08:44:34Z
 ---
 
-
 ## Summary
 
 JSON Lines (JSONL) utilities for streaming large datasets.
diff --git a/.aw/issues/closed/enhancement-mathutils-clamp-bits-bit-string-array-cclab-sci.md b/.aw/issues/closed/enhancement-mathutils-clamp-bits-bit-string-array-cclab-sci.md
index 5bd088c1..595d6c74 100644
--- a/.aw/issues/closed/enhancement-mathutils-clamp-bits-bit-string-array-cclab-sci.md
+++ b/.aw/issues/closed/enhancement-mathutils-clamp-bits-bit-string-array-cclab-sci.md
@@ -10,10 +10,9 @@ labels:
 - priority:p1
 - crate:sci
 created_at: 2026-03-13T11:54:00Z
-updated_at: 2026-03-13T11:58:28Z
+updated_at: 2026-04-09T08:44:34Z
 ---
 
-
 ## Summary
 
 Mathematical utility functions.
diff --git a/.aw/issues/closed/enhancement-onnx-runtime-inference-engine-eliminate-ml-depende.md b/.aw/issues/closed/enhancement-onnx-runtime-inference-engine-eliminate-ml-depende.md
index e3a6c73d..43fd4717 100644
--- a/.aw/issues/closed/enhancement-onnx-runtime-inference-engine-eliminate-ml-depende.md
+++ b/.aw/issues/closed/enhancement-onnx-runtime-inference-engine-eliminate-ml-depende.md
@@ -8,10 +8,9 @@ author: chrischeng-c4
 labels:
 - type:enhancement
 created_at: 2026-03-20T10:04:37Z
-updated_at: 2026-03-20T10:04:37Z
+updated_at: 2026-04-09T08:44:20Z
 ---
 
-
 ## Motivation
 
 `projects/das-v2/` currently depends on 7 ML packages that form a tightly coupled inference stack:
diff --git a/.aw/issues/closed/enhancement-pathutils-augpath-shrinkuser-expandpath-cclab-util.md b/.aw/issues/closed/enhancement-pathutils-augpath-shrinkuser-expandpath-cclab-util.md
index e4590146..1967b9ef 100644
--- a/.aw/issues/closed/enhancement-pathutils-augpath-shrinkuser-expandpath-cclab-util.md
+++ b/.aw/issues/closed/enhancement-pathutils-augpath-shrinkuser-expandpath-cclab-util.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:util
 created_at: 2026-03-13T11:54:34Z
-updated_at: 2026-03-13T11:58:38Z
+updated_at: 2026-04-09T08:44:33Z
 ---
 
-
 ## Summary
 
 Path manipulation utilities.
diff --git a/.aw/issues/closed/enhancement-platform-adapter-format-conversion-comment-create.md b/.aw/issues/closed/enhancement-platform-adapter-format-conversion-comment-create.md
index 06136fe5..71458c15 100644
--- a/.aw/issues/closed/enhancement-platform-adapter-format-conversion-comment-create.md
+++ b/.aw/issues/closed/enhancement-platform-adapter-format-conversion-comment-create.md
@@ -10,10 +10,9 @@ labels:
 - crate:agent
 - priority:p0
 created_at: 2026-03-18T02:52:57Z
-updated_at: 2026-03-18T02:52:57Z
+updated_at: 2026-04-09T08:44:32Z
 ---
 
-
 ## Summary
 
 Platform-specific adapter layer that converts between GH/GL/Jira formats and the Restructure Agent's standard I/O. Handles writing back to platforms (comments, create issues, link).
diff --git a/.aw/issues/closed/enhancement-pyo3-native-modules-not-loadable-so-build-system-b.md b/.aw/issues/closed/enhancement-pyo3-native-modules-not-loadable-so-build-system-b.md
index 634c2a5d..e2160853 100644
--- a/.aw/issues/closed/enhancement-pyo3-native-modules-not-loadable-so-build-system-b.md
+++ b/.aw/issues/closed/enhancement-pyo3-native-modules-not-loadable-so-build-system-b.md
@@ -9,10 +9,9 @@ labels:
 - crate:core
 - priority:p0
 created_at: 2026-03-20T09:32:52Z
-updated_at: 2026-03-20T09:32:52Z
+updated_at: 2026-04-09T08:44:30Z
 ---
 
-
 ## Problem
 
 All cclab Python modules fail to load native code. The `try/except ImportError: pass` pattern in `__init__.py` silently swallows errors, making modules appear importable but empty.
diff --git a/.aw/issues/closed/enhancement-queueutils-heappriorityqueue-sortedpriorityqueue-c.md b/.aw/issues/closed/enhancement-queueutils-heappriorityqueue-sortedpriorityqueue-c.md
index 9f29002f..a1ba6e1f 100644
--- a/.aw/issues/closed/enhancement-queueutils-heappriorityqueue-sortedpriorityqueue-c.md
+++ b/.aw/issues/closed/enhancement-queueutils-heappriorityqueue-sortedpriorityqueue-c.md
@@ -10,10 +10,9 @@ labels:
 - crate:queue
 - priority:p1
 created_at: 2026-03-13T11:53:53Z
-updated_at: 2026-03-13T11:58:26Z
+updated_at: 2026-04-09T08:44:35Z
 ---
 
-
 ## Summary
 
 Priority queue implementations with richer API than stdlib `heapq`.
diff --git a/.aw/issues/closed/enhancement-remove-silent-importerror-swallowing-in-init-py-fi.md b/.aw/issues/closed/enhancement-remove-silent-importerror-swallowing-in-init-py-fi.md
index 2c0ebee8..525954dc 100644
--- a/.aw/issues/closed/enhancement-remove-silent-importerror-swallowing-in-init-py-fi.md
+++ b/.aw/issues/closed/enhancement-remove-silent-importerror-swallowing-in-init-py-fi.md
@@ -9,10 +9,9 @@ labels:
 - crate:core
 - priority:p0
 created_at: 2026-03-20T09:32:53Z
-updated_at: 2026-03-20T09:32:53Z
+updated_at: 2026-04-09T08:44:30Z
 ---
 
-
 ## Problem
 
 All `python/cclab/*/__init__.py` files use:
diff --git a/.aw/issues/closed/enhancement-setutils-indexedset-with-ordering-indexing-and-set.md b/.aw/issues/closed/enhancement-setutils-indexedset-with-ordering-indexing-and-set.md
index 1ee87901..8e532991 100644
--- a/.aw/issues/closed/enhancement-setutils-indexedset-with-ordering-indexing-and-set.md
+++ b/.aw/issues/closed/enhancement-setutils-indexedset-with-ordering-indexing-and-set.md
@@ -10,10 +10,9 @@ labels:
 - priority:p0
 - crate:collections
 created_at: 2026-03-13T11:53:09Z
-updated_at: 2026-03-13T11:58:58Z
+updated_at: 2026-04-09T08:44:37Z
 ---
 
-
 ## Summary
 
 Implement `IndexedSet` — an ordered set that supports both set operations and index-based access.
diff --git a/.aw/issues/closed/enhancement-statsutils-stats-class-histogram-trimean-kurtosis.md b/.aw/issues/closed/enhancement-statsutils-stats-class-histogram-trimean-kurtosis.md
index 417f8d73..962085c1 100644
--- a/.aw/issues/closed/enhancement-statsutils-stats-class-histogram-trimean-kurtosis.md
+++ b/.aw/issues/closed/enhancement-statsutils-stats-class-histogram-trimean-kurtosis.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:sci
 created_at: 2026-03-13T11:54:27Z
-updated_at: 2026-03-13T11:58:36Z
+updated_at: 2026-04-09T08:44:33Z
 ---
 
-
 ## Summary
 
 Self-contained statistics calculator (lighter than cclab-sci, no external deps).
diff --git a/.aw/issues/closed/enhancement-strutils-slugify-case-conversion-pluralize-asciify.md b/.aw/issues/closed/enhancement-strutils-slugify-case-conversion-pluralize-asciify.md
index 004924c7..6aab261c 100644
--- a/.aw/issues/closed/enhancement-strutils-slugify-case-conversion-pluralize-asciify.md
+++ b/.aw/issues/closed/enhancement-strutils-slugify-case-conversion-pluralize-asciify.md
@@ -10,10 +10,9 @@ labels:
 - priority:p0
 - crate:text
 created_at: 2026-03-13T11:53:17Z
-updated_at: 2026-03-13T11:58:18Z
+updated_at: 2026-04-09T08:44:36Z
 ---
 
-
 ## Summary
 
 String manipulation utilities commonly needed but missing from stdlib.
diff --git a/.aw/issues/closed/enhancement-tableutils-table-type-with-to-text-to-html-from-di.md b/.aw/issues/closed/enhancement-tableutils-table-type-with-to-text-to-html-from-di.md
index 7fe90403..0c08a8a3 100644
--- a/.aw/issues/closed/enhancement-tableutils-table-type-with-to-text-to-html-from-di.md
+++ b/.aw/issues/closed/enhancement-tableutils-table-type-with-to-text-to-html-from-di.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:text
 created_at: 2026-03-13T11:54:12Z
-updated_at: 2026-03-13T11:58:32Z
+updated_at: 2026-04-09T08:44:34Z
 ---
 
-
 ## Summary
 
 Lightweight 2D table type for formatted output (not a DataFrame — for display/reporting).
diff --git a/.aw/issues/closed/enhancement-tbutils-tracebackinfo-parsedexception-contextualca.md b/.aw/issues/closed/enhancement-tbutils-tracebackinfo-parsedexception-contextualca.md
index 6de2c4a7..f22d041c 100644
--- a/.aw/issues/closed/enhancement-tbutils-tracebackinfo-parsedexception-contextualca.md
+++ b/.aw/issues/closed/enhancement-tbutils-tracebackinfo-parsedexception-contextualca.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:log
 created_at: 2026-03-13T11:54:45Z
-updated_at: 2026-03-13T11:58:55Z
+updated_at: 2026-04-09T08:44:33Z
 ---
 
-
 ## Summary
 
 Rich traceback introspection and manipulation utilities.
diff --git a/.aw/issues/closed/enhancement-timeutils-extended-daterange-isoparse-parse-timede.md b/.aw/issues/closed/enhancement-timeutils-extended-daterange-isoparse-parse-timede.md
index 7a1cc175..a4f4ba32 100644
--- a/.aw/issues/closed/enhancement-timeutils-extended-daterange-isoparse-parse-timede.md
+++ b/.aw/issues/closed/enhancement-timeutils-extended-daterange-isoparse-parse-timede.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - crate:util
 created_at: 2026-03-13T11:54:22Z
-updated_at: 2026-03-13T11:58:35Z
+updated_at: 2026-04-09T08:44:33Z
 ---
 
-
 ## Summary
 
 Extend time/date utilities beyond existing humanize functions.
diff --git a/.aw/issues/closed/enhancement-tracking-rust-utility-crate-boltons-inspired-batte.md b/.aw/issues/closed/enhancement-tracking-rust-utility-crate-boltons-inspired-batte.md
index 06cf750e..abb03650 100644
--- a/.aw/issues/closed/enhancement-tracking-rust-utility-crate-boltons-inspired-batte.md
+++ b/.aw/issues/closed/enhancement-tracking-rust-utility-crate-boltons-inspired-batte.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - type:epic
 created_at: 2026-03-13T11:52:51Z
-updated_at: 2026-03-13T11:58:07Z
+updated_at: 2026-04-09T08:44:37Z
 ---
 
-
 ## Summary
 
 Implement boltons-inspired utilities distributed across existing and new cclab crates. Each module lands in the crate that best matches its domain.
diff --git a/.aw/issues/closed/enhancement-typeutils-make-sentinel-classproperty-get-all-subc.md b/.aw/issues/closed/enhancement-typeutils-make-sentinel-classproperty-get-all-subc.md
index b39eebea..89698ebe 100644
--- a/.aw/issues/closed/enhancement-typeutils-make-sentinel-classproperty-get-all-subc.md
+++ b/.aw/issues/closed/enhancement-typeutils-make-sentinel-classproperty-get-all-subc.md
@@ -10,10 +10,9 @@ labels:
 - crate:core
 - priority:p1
 created_at: 2026-03-13T11:53:48Z
-updated_at: 2026-03-13T11:58:24Z
+updated_at: 2026-04-09T08:44:35Z
 ---
 
-
 ## Summary
 
 Type-level utilities for Python/Rust interop.
diff --git a/.aw/issues/closed/enhancement-update-python-stubs-and-expose-missing-chart-types.md b/.aw/issues/closed/enhancement-update-python-stubs-and-expose-missing-chart-types.md
index fdd6fb1c..ea426be9 100644
--- a/.aw/issues/closed/enhancement-update-python-stubs-and-expose-missing-chart-types.md
+++ b/.aw/issues/closed/enhancement-update-python-stubs-and-expose-missing-chart-types.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:plot
 created_at: 2026-03-12T06:22:56Z
-updated_at: 2026-03-12T06:22:56Z
+updated_at: 2026-04-09T08:44:43Z
 ---
 
-
 ## Context
 Plot module is at 85% completion. The Python stub (`__init__.pyi`) only exposes 7 chart types, but Rust core has 13.
 
diff --git a/.aw/issues/closed/enhancement-urlutils-url-class-find-all-links-parse-qsl-quote.md b/.aw/issues/closed/enhancement-urlutils-url-class-find-all-links-parse-qsl-quote.md
index 98054c3b..24405bb6 100644
--- a/.aw/issues/closed/enhancement-urlutils-url-class-find-all-links-parse-qsl-quote.md
+++ b/.aw/issues/closed/enhancement-urlutils-url-class-find-all-links-parse-qsl-quote.md
@@ -10,10 +10,9 @@ labels:
 - crate:http
 - priority:p1
 created_at: 2026-03-13T11:54:03Z
-updated_at: 2026-03-13T11:58:30Z
+updated_at: 2026-04-09T08:44:34Z
 ---
 
-
 ## Summary
 
 Structured URL manipulation beyond stdlib `urllib.parse`.
diff --git a/.aw/issues/closed/enhancement-video-codec-and-container-format-support.md b/.aw/issues/closed/enhancement-video-codec-and-container-format-support.md
index b8eaa4cd..02b5929e 100644
--- a/.aw/issues/closed/enhancement-video-codec-and-container-format-support.md
+++ b/.aw/issues/closed/enhancement-video-codec-and-container-format-support.md
@@ -9,10 +9,9 @@ labels:
 - type:enhancement
 - crate:media
 created_at: 2026-03-12T06:23:30Z
-updated_at: 2026-03-12T06:23:30Z
+updated_at: 2026-04-09T08:44:41Z
 ---
 
-
 ## Context
 Video module only supports raw (uncompressed) codec. H.264/H.265/VP8/VP9/AV1 enums are defined but not implemented. Cannot read/write .mp4, .mkv, .webm files.
 
diff --git a/.aw/issues/closed/epic-align-cclab-python-api-to-ecosystem-conventions.md b/.aw/issues/closed/epic-align-cclab-python-api-to-ecosystem-conventions.md
index 7e92c5f3..94c03af5 100644
--- a/.aw/issues/closed/epic-align-cclab-python-api-to-ecosystem-conventions.md
+++ b/.aw/issues/closed/epic-align-cclab-python-api-to-ecosystem-conventions.md
@@ -9,10 +9,9 @@ labels:
 - priority:p0
 - type:epic
 created_at: 2026-03-20T08:23:33Z
-updated_at: 2026-03-20T08:23:33Z
+updated_at: 2026-04-09T08:44:30Z
 ---
 
-
 ## Vision
 
 cclab.* claims to replace Python ecosystem (Pydantic, FastAPI, SQLAlchemy, etc.), but the current Python API doesn't match conventions. Fix from Rust level, not aliases.
diff --git a/.aw/issues/closed/epic-align-with-cclab-sdd-as-cloud-sdd-runtime.md b/.aw/issues/closed/epic-align-with-cclab-sdd-as-cloud-sdd-runtime.md
index b6479ec3..2711e4e9 100644
--- a/.aw/issues/closed/epic-align-with-cclab-sdd-as-cloud-sdd-runtime.md
+++ b/.aw/issues/closed/epic-align-with-cclab-sdd-as-cloud-sdd-runtime.md
@@ -11,10 +11,9 @@ labels:
 - project:conductor
 - project:agentic-workflow
 created_at: 2026-03-23T16:30:26Z
-updated_at: 2026-04-05T12:04:05Z
+updated_at: 2026-04-09T08:44:17Z
 ---
 
-
 ## Context
 
 Conductor 目前自己實作了 pipeline executor、agent runtime、spec generation，但這些能力 cclab 生態系已經有了。Conductor 應該是 **cclab-sdd 的雲端 adapter**，不是獨立產品。
diff --git a/.aw/issues/closed/epic-conductor-rewires-onto-arsenal-crates-extends-1049.md b/.aw/issues/closed/epic-conductor-rewires-onto-arsenal-crates-extends-1049.md
index d923a776..2ed0d335 100644
--- a/.aw/issues/closed/epic-conductor-rewires-onto-arsenal-crates-extends-1049.md
+++ b/.aw/issues/closed/epic-conductor-rewires-onto-arsenal-crates-extends-1049.md
@@ -10,10 +10,9 @@ labels:
 - type:epic
 - project:conductor
 created_at: 2026-04-05T10:30:33Z
-updated_at: 2026-04-05T12:45:03Z
+updated_at: 2026-04-09T08:43:38Z
 ---
 
-
 ## Problem
 
 Conductor's Python backend currently reimplements pipeline execution, agent runtime, and spec storage — things that already exist in `cclab-queue`, `cclab-agent`, and the soon-to-be-shrunk `cclab-sdd`. This duplicates work Score will do with the same arsenal.
diff --git a/.aw/issues/closed/epic-create-projects-score-local-cli-sdd-show-case.md b/.aw/issues/closed/epic-create-projects-score-local-cli-sdd-show-case.md
index 2c69fcec..0f08de9b 100644
--- a/.aw/issues/closed/epic-create-projects-score-local-cli-sdd-show-case.md
+++ b/.aw/issues/closed/epic-create-projects-score-local-cli-sdd-show-case.md
@@ -10,11 +10,9 @@ labels:
 - type:epic
 - project:agentic-workflow
 created_at: 2026-04-05T10:30:06Z
-updated_at: 2026-04-05T10:38:01Z
+updated_at: 2026-04-09T08:43:38Z
 ---
 
-
-
 ## Problem
 
 `cclab-sdd` was historically treated as a Rust crate, but conceptually it is **a project** — a local CLI tool that uses Claude Code to implement SDD. It should live under `projects/` as a sibling to `conductor/`, not under `crates/`.
diff --git a/.aw/issues/closed/epic-decompose-cclab-sdd-library-multi-kind-generalizat.md b/.aw/issues/closed/epic-decompose-cclab-sdd-library-multi-kind-generalizat.md
index 99a973a9..bfe32e1f 100644
--- a/.aw/issues/closed/epic-decompose-cclab-sdd-library-multi-kind-generalizat.md
+++ b/.aw/issues/closed/epic-decompose-cclab-sdd-library-multi-kind-generalizat.md
@@ -10,10 +10,9 @@ labels:
 - type:epic
 - crate:sdd
 created_at: 2026-04-05T10:29:46Z
-updated_at: 2026-04-05T12:59:42Z
+updated_at: 2026-04-09T08:43:38Z
 ---
 
-
 ## Problem
 
 `crates/cclab-sdd/` has become a monolith that mixes arsenal primitives (reusable across projects) with show case concerns (local CLI, MCP server, filesystem storage). At the same time, it only supports one artifact kind (`code-change`), but the SDD concept applies to BRD / PRD / TD / Issue / Code / Runbook / Design uniformly.
diff --git a/.aw/issues/closed/epic-design-mamba-concurrency-semantics-no-gil-thread-s.md b/.aw/issues/closed/epic-design-mamba-concurrency-semantics-no-gil-thread-s.md
index 47ff8de7..b493cbf9 100644
--- a/.aw/issues/closed/epic-design-mamba-concurrency-semantics-no-gil-thread-s.md
+++ b/.aw/issues/closed/epic-design-mamba-concurrency-semantics-no-gil-thread-s.md
@@ -10,62 +10,134 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-20T10:22:53Z
-updated_at: 2026-03-25T02:05:27Z
+updated_at: 2026-04-09T08:44:20Z
 ---
 
+## Context
 
-## Problem
+Mamba removes the GIL. This is a fundamental behavioral difference from CPython 3.12. We need to clearly define what guarantees Mamba provides, so users can run existing Python code with predictable results.
 
-Mamba ships without a GIL. That is a correctness story for the runtime, but it is an ambiguity story for users: every piece of existing Python code silently relied on the GIL as an accidental lock, and now we have to decide — publicly and in writing — what Mamba guarantees in its place. Until that contract exists, "does my code still work?" has no answer, and every migration stalls at the first `counter += 1`.
+**Goal: "no-pain migration, same results"** — users should not need to rewrite code, and program output should be identical to CPython.
 
-This epic is the design-and-decide bucket for Mamba's concurrency semantics. It does not implement anything directly — it produces a specification (in `.aw/tech-design/cclab-mamba/logic/concurrency-semantics.md`) that every subsequent concurrency-sensitive issue is measured against.
+## The Problem
 
-## Goals
+CPython's GIL accidentally provides thread-safety guarantees that many Python programs rely on without knowing:
 
-- Define a written concurrency contract: what Mamba guarantees to be atomic, what it does not, and what happens on a data race.
-- Choose a default behavior that keeps "no-pain migration, same results" viable for the common case — shared counters, dict caches, list accumulators — without paying CPython's GIL tax everywhere.
-- Provide an escape hatch (a `--gil`-style mode) for users whose code is not yet audited for racy patterns.
-- Give the compiler and runtime team a clear target so related issues (#1002, #1003, #1190) can assume a stable concurrency model.
+```python
+# Pattern 1: global counter
+counter = 0
+def inc():
+    global counter
+    counter += 1  # CPython: atomic (GIL). Mamba: race condition.
+
+# Pattern 2: shared collection
+results = []
+def worker(x):
+    results.append(x)  # CPython: safe (GIL). Mamba: needs lock on list.
+
+# Pattern 3: dict mutation
+cache = {}
+def update(k, v):
+    cache[k] = v  # CPython: safe (GIL). Mamba: race condition on dict.
+
+# Pattern 4: iteration
+for item in shared_list:  # CPython: raises RuntimeError if mutated. Mamba: ???
+    process(item)
+```
+
+## Design Options
+
+### Option A: Fine-grained locks on builtins (CPython 3.13 approach)
+
+Every builtin operation (`list.append`, `dict.__setitem__`, `+=`) acquires a per-object lock internally.
+
+| Pros | Cons |
+|------|------|
+| Closest to CPython behavior | Performance overhead on every operation |
+| Existing code just works | Complex implementation |
+| Same approach as CPython 3.13 free-threaded | Lock contention under high concurrency |
+
+### Option B: Atomic builtins, explicit locking for compounds
+
+Builtin single operations are atomic (list.append, dict[k]=v), but compound operations (read-modify-write like `+=`) are NOT.
+
+```python
+# Safe (single atomic operation)
+results.append(x)       # Mamba: internally locked
+cache[k] = v            # Mamba: internally locked
+
+# NOT safe (compound operation)
+counter += 1             # LOAD + ADD + STORE — not atomic
+cache[k] = cache[k] + 1  # READ + MODIFY + WRITE — not atomic
+# User must use threading.Lock for these
+```
+
+| Pros | Cons |
+|------|------|
+| Lower overhead (no lock on every bytecode) | `counter += 1` breaks without Lock |
+| Clear mental model | Differs from CPython |
+| Matches Java/Go/Rust behavior | |
+
+### Option C: Automatic lock coarsening by compiler
+
+Mamba compiler detects shared mutable state and automatically inserts locks.
+
+```python
+# User writes:
+counter += 1
+
+# Mamba compiles to (conceptually):
+with __auto_lock(counter):
+    counter += 1
+```
+
+| Pros | Cons |
+|------|------|
+| Zero user effort | Hard to get right (what is "shared"?) |
+| Truly "no-pain migration" | May over-lock (performance) or under-lock (bugs) |
+| Unique compiler advantage | Complex static analysis |
+
+### Option D: GIL-compatible mode (opt-in)
+
+Provide a `--gil` flag that enables a global lock, making Mamba behave exactly like CPython.
+
+```bash
+mamba run script.py          # No GIL (fast, but need correct code)
+mamba run --gil script.py    # With GIL (slow, but CPython-compatible)
+```
+
+| Pros | Cons |
+|------|------|
+| 100% compatible when needed | Defeats the purpose of removing GIL |
+| Easy escape hatch | |
+| Good for migration/debugging | |
 
 ## Current State
 
-The runtime already takes per-object `RwLock`s on the core mutable builtins (`List`, `Dict`, `Set` in `ObjData`). That means single operations like `list.append()` and `dict[k] = v` are already atomic. The open design question is about **compound** operations: `counter += 1`, `cache[k] = cache[k] + 1`, `results[-1].field += x` — all of which CPython makes look atomic by holding the GIL across bytecodes.
+Mamba runtime already has `RwLock` on:
+- `ObjData::List(RwLock<Vec<MbValue>>)` 
+- `ObjData::Dict(RwLock<HashMap<String, MbValue>>)`
+- `ObjData::Set(RwLock<Vec<MbValue>>)`
 
-## Design Options Under Consideration
+This means single operations like `list.append()`, `dict[k] = v` are already thread-safe. The gap is compound operations (`+=`, `read-then-write`).
 
-- **A — Fine-grained locks on all builtins** (CPython 3.13 free-threaded approach). Maximum compatibility, highest overhead.
-- **B — Atomic single ops, explicit locking for compounds**. Industry-standard model (Java, Go, Rust). Breaks `counter += 1` without a `Lock`.
-- **C — Automatic lock coarsening by the compiler**. Uniquely Mamba; hardest to get right.
-- **D — `--gil` opt-in mode**. Migration escape hatch; not a default.
+## Questions to Decide
 
-Detailed trade-off analysis lives in the working design doc, not in this tracker.
+1. **Which option (A/B/C/D) or combination?**
+2. **Should `counter += 1` be atomic by default?** (CPython: yes via GIL. Java/Go/Rust: no.)
+3. **What happens on data race?** (Defined behavior with stale value? Exception? Undefined?)
+4. **Should single-threaded programs pay any locking cost?**
+5. **Do we need a migration tool that detects GIL-dependent patterns?**
 
-## Sub-issues
+## Recommendation
 
-No sub-issues exist yet — this epic is the placeholder for the design decision itself. Once the contract is agreed, it will spawn implementation issues for:
+Combination of **B + D**:
+- Default: atomic builtins, explicit locking for compound ops (matches industry standard)
+- `--gil` flag for 100% CPython compatibility during migration
+- Document clearly: "Mamba is thread-safe for single operations. Use `threading.Lock` for read-modify-write patterns."
+- Future: static analysis warning for detected race conditions
 
-- Compile-time race detection pass (if option C survives).
-- `--gil` runtime mode wiring (if option D survives).
-- Documentation updates ("Migrating from CPython: threading model").
-- Test matrix covering the decided atomicity contract.
-
-## Milestones
-
-1. **Decision recorded** — written spec in `.aw/tech-design/cclab-mamba/logic/concurrency-semantics.md` with a chosen option (or combination) and a rationale.
-2. **User-facing doc** — "Concurrency in Mamba" page describing atomic ops, compound ops, and recommended locking patterns.
-3. **Compliance tests** — integration tests that assert the decided contract (what is atomic, what is not, what happens on a race).
-4. **Migration guide** — examples of GIL-dependent patterns and their Mamba-safe rewrites.
-
-## Out of Scope
-
-- Implementing `threading.Lock`, `threading.RLock`, `queue.Queue` — those are stdlib issues, not concurrency-semantics issues.
-- The tokio runtime design for async I/O — separate track.
-- Garbage collection under no-GIL — separate concern (#757).
-- Whether to support free-threaded CPython-compiled extensions — that is #843's problem.
-
-## Related
-
-- #1002 — gevent/greenlet compatibility (depends on the decision here)
-- #1003 — gRPC compatibility (multi-threaded server semantics)
-- #1004 — C library ecosystem tracking
-- #1190 — Import system (module-level state is shared state)
+## Related Issues
+- #1002 gevent/greenlet compatibility
+- #1003 gRPC compatibility
+- #1004 C library ecosystem tracking
diff --git a/.aw/issues/closed/epic-domain-first-monorepo-layout-libs-apps-platform.md b/.aw/issues/closed/epic-domain-first-monorepo-layout-libs-apps-platform.md
index 08b691ba..90dc8ebb 100644
--- a/.aw/issues/closed/epic-domain-first-monorepo-layout-libs-apps-platform.md
+++ b/.aw/issues/closed/epic-domain-first-monorepo-layout-libs-apps-platform.md
@@ -10,10 +10,9 @@ labels:
 - type:epic
 - type:refactor
 created_at: 2026-04-07T07:08:45Z
-updated_at: 2026-04-07T07:08:45Z
+updated_at: 2026-04-09T08:43:32Z
 ---
 
-
 ## Problem
 
 The repository organizes code by **language runtime** (`crates/` for Rust, `packages/` for TypeScript/npm), forcing developers to scatter a single domain across multiple unrelated directories.
diff --git a/.aw/issues/closed/epic-formal-notation-for-non-code-artifact-kinds-brd-pr.md b/.aw/issues/closed/epic-formal-notation-for-non-code-artifact-kinds-brd-pr.md
index 770919aa..76d164ba 100644
--- a/.aw/issues/closed/epic-formal-notation-for-non-code-artifact-kinds-brd-pr.md
+++ b/.aw/issues/closed/epic-formal-notation-for-non-code-artifact-kinds-brd-pr.md
@@ -11,10 +11,9 @@ labels:
 - project:conductor
 - project:agentic-workflow
 created_at: 2026-04-05T10:31:02Z
-updated_at: 2026-04-05T10:38:04Z
+updated_at: 2026-04-09T08:43:38Z
 ---
 
-
 ## Problem
 
 Current Conductor/cclab-sdd specs are all **tech-design-level**: OpenAPI, ER diagrams, state machines, layout DSL. These notations are mature because the industry has 30 years of conventions.
diff --git a/.aw/issues/closed/epic-py3-12-conformance-tracking-session-progress.md b/.aw/issues/closed/epic-py3-12-conformance-tracking-session-progress.md
index a28adf31..8575514c 100644
--- a/.aw/issues/closed/epic-py3-12-conformance-tracking-session-progress.md
+++ b/.aw/issues/closed/epic-py3-12-conformance-tracking-session-progress.md
@@ -10,75 +10,23 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-04-07T01:51:08Z
-updated_at: 2026-04-07T01:51:08Z
+updated_at: 2026-04-09T08:43:31Z
 ---
 
+## Session Summary (2026-04-05 ~ 2026-04-07)
 
-## Problem
+### Test Status
+- **1000+ integration tests passing**, 0 ignored, 0 xfail
+- behavioral_lang: 52/52, behavioral_stdlib: 41/41, conformance: 136/136, fixture: 284/284
 
-Py3.12 conformance work happens in bursts — a few days of concentrated attention produces dozens of commits across parser, runtime, and stdlib. Without a rolling summary issue, the progress becomes invisible the moment the session ends: neither reviewers nor future-us can answer "what moved last week?" without grep-spelunking git log. This epic is that rolling summary — the narrative ledger that complements the atomic per-feature issues under #750.
+### Features Implemented (18 features, 35+ commits)
+yield-from throw/close, lambda defaults, decorated func ABI, callable-sentinel iter, functools.partial, functools.wraps+__name__, re.Match, datetime/timedelta, Counter, defaultdict, deque, namedtuple, json indent, class decorators, chained comparisons, walrus operator, generic kwargs+defaults, list/tuple repeat
 
-Unlike #750 (which is a static matrix of required behaviors), this epic is time-bound and session-scoped. Each session appends a new summary block; resolved features are referenced, not re-described.
+### Performance
+int_sum 2.2x faster, fib 2.0x faster, generator 10x improved (52ms→5ms), average 1.0x CPython
 
-## Goals
-
-- Keep a running ledger of Py3.12 conformance progress visible at a glance.
-- Surface regressions quickly — if test counts drop or performance tanks between sessions, it shows up here first.
-- Provide reviewers with a "what's new since last week" entry point that does not require reading every individual feature issue.
-- Act as the handover doc between work sessions, so the next person (or the same person, two weeks later) can resume without a full context-rebuild.
-
-## Current Session (2026-04-05 ~ 2026-04-07)
-
-**Test status**
-
-- 1000+ integration tests passing, 0 ignored, 0 xfail.
-- behavioral_lang: 52/52, behavioral_stdlib: 41/41, conformance: 136/136, fixture: 284/284.
-
-**Features landed (18 features, 35+ commits)**
-
-`yield from` throw/close, lambda defaults, decorated function ABI, callable-sentinel `iter`, `functools.partial`, `functools.wraps` + `__name__`, `re.Match`, `datetime` / `timedelta`, `Counter`, `defaultdict`, `deque`, `namedtuple`, `json` indent, class decorators, chained comparisons, walrus operator, generic `kwargs` + defaults, list/tuple repeat.
-
-**Performance**
-
-- `int_sum` 2.2x faster than previous session.
-- `fib` 2.0x faster.
-- Generator throughput 10x improved (52 ms → 5 ms on the benchmark case).
-- Average vs CPython: 1.0x (parity).
-
-**Known gaps carried into next session**
-
-- #1187 — Generator coroutine (remaining 5x performance gap)
-- #1189 — `*args` / `**kwargs` parameter handling
-- #1190 — Import system
-- #1191 — Collection performance
-
-## Sub-issues
-
-This epic tracks sessions, not individual features. Feature-level tracking lives under the master conformance epic #750. Active open child issues referenced in the current session:
-
-- #1187 — Generator coroutine performance
-- #1189 — `*args` / `**kwargs` params
-- #1190 — Import system
-- #1191 — Collection perf
-
-## Milestones
-
-Sessions are not scheduled — they happen when concentrated conformance work happens. Milestone moments worth recording here:
-
-- **Test count crosses 1500** (milestone for "core runtime done")
-- **Average perf vs CPython crosses 1.5x** (milestone for "measurably faster")
-- **Zero `xfail` for more than 10 consecutive sessions** (milestone for stability)
-- **First self-hosted benchmark suite** (runs Mamba benchmarks on Mamba itself)
-
-## Out of Scope
-
-- Individual feature specs — those belong to their own issues.
-- Stdlib native rewrites — tracked in #749.
-- Package manager work — tracked in #751.
-- Long-term design (concurrency, imports) — tracked in #1009 and #1190 respectively.
-
-## Related
-
-- #750 — Py3.12 conformance & test coverage (the master matrix)
-- #749 — Native stdlib rewrite
-- #851 — Language features & tooling completeness
+### Remaining
+- #1187 Generator coroutine (5x gap)
+- #1189 *args/**kwargs params
+- #1190 Import system
+- #1191 Collection perf
diff --git a/.aw/issues/closed/epic-refactor-to-thin-shell-extract-all-logic-to-cclab.md b/.aw/issues/closed/epic-refactor-to-thin-shell-extract-all-logic-to-cclab.md
index c098ee8f..0a92d1e4 100644
--- a/.aw/issues/closed/epic-refactor-to-thin-shell-extract-all-logic-to-cclab.md
+++ b/.aw/issues/closed/epic-refactor-to-thin-shell-extract-all-logic-to-cclab.md
@@ -10,10 +10,9 @@ labels:
 - type:epic
 - project:conductor
 created_at: 2026-03-25T04:26:19Z
-updated_at: 2026-04-05T10:38:30Z
+updated_at: 2026-04-09T08:43:43Z
 ---
 
-
 ## Vision
 
 Conductor should have **zero custom runtime logic**. Every capability comes from a cclab crate or package. Conductor is a composition layer: routes + schemas + config + prompts.
diff --git a/.aw/issues/closed/epic-sdd-post-change-operations-workflow-monitor-detect.md b/.aw/issues/closed/epic-sdd-post-change-operations-workflow-monitor-detect.md
index 35f6ed32..c955f8e8 100644
--- a/.aw/issues/closed/epic-sdd-post-change-operations-workflow-monitor-detect.md
+++ b/.aw/issues/closed/epic-sdd-post-change-operations-workflow-monitor-detect.md
@@ -9,10 +9,9 @@ labels:
 - type:epic
 - priority:p3
 created_at: 2026-03-24T09:15:23Z
-updated_at: 2026-03-24T09:15:23Z
+updated_at: 2026-04-09T08:43:44Z
 ---
 
-
 Split from #1058. Operations is a feedback loop, fundamentally different from the linear planning workflow.
 
 ## Goal
diff --git a/.aw/issues/closed/epic-sdd-pre-change-planning-workflow-goals-issues.md b/.aw/issues/closed/epic-sdd-pre-change-planning-workflow-goals-issues.md
index 80f83fc4..29ac00ce 100644
--- a/.aw/issues/closed/epic-sdd-pre-change-planning-workflow-goals-issues.md
+++ b/.aw/issues/closed/epic-sdd-pre-change-planning-workflow-goals-issues.md
@@ -9,10 +9,9 @@ labels:
 - type:epic
 - priority:p3
 created_at: 2026-03-24T09:15:11Z
-updated_at: 2026-03-24T09:15:11Z
+updated_at: 2026-04-09T08:43:45Z
 ---
 
-
 Split from #1058. Pre-change planning is a separate workflow from operations.
 
 ## Goal
diff --git a/.aw/issues/closed/epic-tracking-c-library-ecosystem-compatibility-strateg.md b/.aw/issues/closed/epic-tracking-c-library-ecosystem-compatibility-strateg.md
index 44771465..3bea634d 100644
--- a/.aw/issues/closed/epic-tracking-c-library-ecosystem-compatibility-strateg.md
+++ b/.aw/issues/closed/epic-tracking-c-library-ecosystem-compatibility-strateg.md
@@ -10,72 +10,54 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-20T10:03:29Z
-updated_at: 2026-03-25T02:05:44Z
+updated_at: 2026-04-09T08:44:20Z
 ---
 
+## Context
 
-## Problem
+Many critical Python libraries depend on C extensions. This tracking issue organizes the strategy for C library compatibility.
 
-The Python ecosystem's most important libraries are not Python — they are thin Python wrappers around C extensions. numpy, grpcio, cryptography, Pillow, psycopg, pandas: all of these fail immediately on a naive Mamba runtime because Mamba cannot just `dlopen` a CPython ABI `.so`. This is not a bug to be fixed; it is a strategic choice to be managed. Every C dependency we fail to handle is a production workload we cannot accept.
+## Tier 1 — Must Have (production blockers)
 
-This epic is the prioritization and strategy tracker — not an implementation issue. It answers "which C-dependent libraries matter most, and what is our approach for each of them?" and routes the work to native-Rust reimplementation, FFI bridging, or explicit out-of-scope decisions.
-
-## Goals
-
-- Maintain an up-to-date prioritization of C-dependent Python libraries by business impact.
-- For every Tier 1 library, record a decided strategy (native Rust reimplementation, FFI bridge, or wait-and-see) and link to the implementing issue.
-- Keep the philosophy consistent: default to native Rust implementations exposing the same Python API, fall back to FFI only when a Rust rewrite is impractical.
-- Track which `cclab.*` crates already provide drop-in replacements for major C-backed libraries.
-
-## Tier 1 — Production Blockers
-
-| Library | C dependency | Strategy | Tracking |
-|---|---|---|---|
-| numpy | BLAS/LAPACK + custom C | Native Rust via `cclab-array` | existing work |
-| grpcio | libgrpc C core | Native Rust via tonic/prost | #1003 |
-| gevent/greenlet | Stack asm, C extension | Native coroutine impl | #1002 |
-| protobuf | C extension for speed | Native Rust via prost | #1003 |
-| cryptography | OpenSSL / libffi | Native Rust via ring/rustls | — |
-| psycopg2/asyncpg | libpq | Native Rust via tokio-postgres / `cclab-pg` | existing work |
-| pymongo | C extension (optional) | Pure-Python fallback usable today | — |
+| Library | C Dependency | Approach |
+|---------|-------------|----------|
+| **numpy** | BLAS/LAPACK, custom C | Native Rust (cclab-array) |
+| **grpcio** | libgrpc C core | Native Rust (tonic) |
+| **gevent/greenlet** | Stack manipulation asm | Native coroutine impl |
+| **protobuf** | C extension for speed | Native Rust (prost) |
+| **cryptography** | OpenSSL/libffi | Native Rust (ring/rustls) |
+| **psycopg2/asyncpg** | libpq | Native Rust (tokio-postgres) |
+| **pymongo** | C extension optional | Pure Python fallback exists |
 
 ## Tier 2 — High Value
 
-| Library | C dependency | Strategy |
-|---|---|---|
-| Pillow/PIL | libjpeg, libpng, zlib | Native Rust via `image` crate |
-| lxml | libxml2, libxslt | Native Rust via `quick-xml` |
-| pandas | numpy + Cython | Native Rust via `cclab-frame` |
-| pyyaml | libyaml (optional) | Pure-Python fallback usable today |
-| uvloop | libuv | Already have tokio runtime |
-| orjson/ujson | Custom C | Native Rust via `serde_json` |
+| Library | C Dependency | Approach |
+|---------|-------------|----------|
+| **Pillow/PIL** | libjpeg, libpng, zlib | Native Rust (image crate) |
+| **lxml** | libxml2, libxslt | Native Rust (quick-xml) |
+| **pandas** | numpy + Cython | Native Rust (cclab-frame) |
+| **pyyaml** | libyaml optional | Pure Python fallback exists |
+| **uvloop** | libuv | Already have tokio runtime |
+| **orjson/ujson** | Custom C | Native Rust (serde_json) |
 
 ## Tier 3 — Nice to Have
 
-| Library | C dependency | Strategy |
-|---|---|---|
-| scipy | BLAS/LAPACK + Fortran | Native Rust via `cclab-sci` (partial) |
-| torch/tensorflow | CUDA + C++ | Out of scope |
-| opencv-python | OpenCV C++ | Native Rust via `cclab-media` (partial) |
+| Library | C Dependency | Approach |
+|---------|-------------|----------|
+| **scipy** | BLAS/LAPACK + Fortran | Native Rust (cclab-sci) |
+| **torch/tensorflow** | CUDA, custom C++ | Out of scope |
+| **opencv-python** | OpenCV C++ | Native Rust (cclab-media) |
 
-## Strategy Principles
+## Strategy
 
-- **Don't load .so files when a Rust rewrite is feasible.** Native implementations are faster, safer, and free us from the CPython ABI.
-- **Preserve the Python API shape.** Users import `numpy as np`; they should not care that `np` is backed by `cclab-array`.
-- **FFI is the escape hatch, not the default.** If a library has no Rust equivalent and cannot be reasonably rewritten, then `ctypes` / `cffi` loading (#981) is the path.
-- **The `cclab.*` crate family is the quiet foundation.** Most of Tier 1 and Tier 2 already has a native Rust counterpart somewhere in the monorepo.
+Mamba's approach differs from CPython compatibility:
+- **Don't load .so files** — instead, provide native Rust implementations with the same Python API
+- **cclab.* modules** already cover many of these (array, frame, pg, http, crypto, etc.)
+- **Bridge layer**: for libraries without native equivalents, consider #843 CPython ABI shim
 
-## Sub-issues
-
-- #843 — C extension compatibility (the general .so loading problem, the escape hatch for everything not listed above)
-- #981 — ctypes FFI
-- #1002 — gevent/greenlet
-- #1003 — gRPC
-- #751 — Package manager (determines how Mamba consumes these libs)
-
-## Out of Scope
-
-- Implementing any specific native replacement — those belong to their own crate-scoped issues.
-- GPU-accelerated ML frameworks (torch, tensorflow, jax) — not reasonable to reimplement.
-- Windows-specific native extensions — Mamba is Linux/macOS first.
-- CPython ABI implementation details — that is #843's problem.
+## Related Issues
+- #843 C extension compatibility (general .so loading)
+- #1002 gevent/greenlet
+- #1003 gRPC
+- #981 ctypes FFI
+- #751 Package manager
diff --git a/.aw/issues/closed/epic-tracking-mamba-language-features-tooling-completen.md b/.aw/issues/closed/epic-tracking-mamba-language-features-tooling-completen.md
index 2b33a1a5..6e7e2343 100644
--- a/.aw/issues/closed/epic-tracking-mamba-language-features-tooling-completen.md
+++ b/.aw/issues/closed/epic-tracking-mamba-language-features-tooling-completen.md
@@ -9,87 +9,60 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-13T11:41:04Z
-updated_at: 2026-03-13T11:41:04Z
+updated_at: 2026-04-09T08:44:37Z
 ---
 
-
-## Problem
-
-Mamba's scope sprawls naturally: parser gaps, runtime correctness bugs, performance pushes, and developer tooling all compete for the same attention. Without a master tracker, work happens in whichever corner is loudest that week, and holistic progress becomes invisible. This epic is the roadmap for everything that is not stdlib (#749) and not Py3.12 conformance tests (#750) — essentially "the compiler, the runtime, and the tools developers touch."
-
-## Goals
-
-- One place to see every open language-feature gap, runtime correctness item, performance target, and tooling deliverable.
-- Group work by category (parser, runtime, perf/tools, ecosystem) so owners can pick a lane without scanning the whole issue list.
-- Keep P0/P1 items visibly prioritized — these are the gating items for "Mamba can run my code."
-- Provide the cross-link hub between #749, #750, #751 (the three sibling trackers) and this one.
-
-## Sub-issues
-
-### Language features — parser / compiler
-
-| # | Feature | Priority |
-|---|---|---|
-| #827 | Match/case (PEP 634) | P0 |
-| #828 | Import aliases (`import X as Y`) | P0 |
-| #829 | Relative imports (`from . import`) | P0 |
-| #841 | Multi-file compilation & module graph | P0 |
-| #830 | PEP 695 full type parameter syntax | P1 |
-| #831 | Dict literal unpacking (`{**d}`) | P1 |
-| #832 | Parenthesized `with` statements (PEP 617) | P1 |
-| #845 | Star expressions / extended unpacking | P1 |
-| #846 | `global` and `nonlocal` statements | P1 |
-| #847 | Decorator arguments and chaining | P1 |
-| #848 | String escapes — Unicode, raw, bytes | P1 |
-
-### Runtime correctness
-
-| # | Feature | Priority |
-|---|---|---|
-| #833 | BigInt fallback (48-bit overflow) | P0 |
-| #834 | Exception chaining (`__cause__`, `__context__`) | P1 |
-| #835 | List/tuple slicing with step | P1 |
-| #849 | Class features — slots, properties, classmethods | P1 |
-| #850 | `async for`, `async with`, async generators | P1 |
-| #844 | Tuple comparison operators | P2 |
-
-### Performance & tooling
-
-| # | Feature | Priority |
-|---|---|---|
-| #836 | Benchmark suite (vs CPython/PyPy) | P0 |
-| #837 | Incremental compilation / caching | P1 |
-| #838 | REPL — interactive shell | P1 |
-| #840 | Error diagnostics quality | P1 |
-| #839 | Language Server Protocol (LSP) | P2 |
+## Tracking: Mamba Language Features & Tooling
+
+Master tracking issue for language feature completeness, runtime correctness, and developer tooling.
+
+### Language Features — Parser/Compiler
+
+| # | Feature | Priority | Status |
+|---|---------|----------|--------|
+| #827 | Match/case (PEP 634) | P0 | 🔲 |
+| #828 | Import aliases (`import X as Y`) | P0 | 🔲 |
+| #829 | Relative imports (`from . import`) | P0 | 🔲 |
+| #841 | Multi-file compilation & module graph | P0 | 🔲 |
+| #830 | PEP 695 full type parameter syntax | P1 | 🔲 |
+| #831 | Dict literal unpacking (`{**d}`) | P1 | 🔲 |
+| #832 | Parenthesized with statements (PEP 617) | P1 | 🔲 |
+| #845 | Star expressions / extended unpacking | P1 | 🔲 |
+| #846 | Global and nonlocal statements | P1 | 🔲 |
+| #847 | Decorator arguments and chaining | P1 | 🔲 |
+| #848 | String escapes — Unicode, raw, bytes | P1 | 🔲 |
+
+### Runtime Correctness
+
+| # | Feature | Priority | Status |
+|---|---------|----------|--------|
+| #833 | BigInt fallback (48-bit overflow) | P0 | 🔲 |
+| #834 | Exception chaining (__cause__, __context__) | P1 | 🔲 |
+| #835 | List/tuple slicing with step | P1 | 🔲 |
+| #844 | Tuple comparison operators | P2 | 🔲 |
+| #849 | Class features — slots, properties, classmethods | P1 | 🔲 |
+| #850 | Async for, async with, async generators | P1 | 🔲 |
+
+### Performance & Tooling
+
+| # | Feature | Priority | Status |
+|---|---------|----------|--------|
+| #836 | Benchmark suite (vs CPython/PyPy) | P0 | 🔲 |
+| #837 | Incremental compilation / caching | P1 | 🔲 |
+| #838 | REPL — interactive shell | P1 | 🔲 |
+| #840 | Error diagnostics quality | P1 | 🔲 |
+| #839 | Language Server Protocol (LSP) | P2 | 🔲 |
 
 ### Ecosystem
 
-| # | Feature | Priority |
-|---|---|---|
-| #842 | WASM playground | P2 |
-| #843 | C extension compatibility | P2 |
-| #751 | Package manager (uv-like) | tracked in own epic |
+| # | Feature | Priority | Status |
+|---|---------|----------|--------|
+| #842 | WASM playground | P2 | 🔲 |
+| #843 | C extension compatibility | P2 | 🔲 |
+| #751 | Package manager (uv-like) | — | 🔲 |
 
-## Milestones
-
-1. **Parser completeness** — all P0 parser items closed. Mamba accepts any well-formed Py3.12 source.
-2. **Runtime parity** — all P0/P1 runtime items closed. Test corpus from #750 passes without xfail.
-3. **Tooling usable** — benchmark, REPL, diagnostics, and cache all shipping. "A developer can use Mamba as their daily driver" becomes true.
-4. **Ecosystem bridges** — WASM playground public, C extension compatibility at least Tier 1 coverage per #1004.
-
-## Out of Scope
-
-- Stdlib module implementations (that is #749).
-- Py3.12 conformance test harness (that is #750).
-- Package manager design and implementation (that is #751).
-- C library ecosystem strategy (that is #1004).
-- Concurrency semantics design (that is #1009).
-
-## Related Trackers
+### Related Tracking Issues
 
 - #750 — Py3.12 conformance & test coverage
 - #749 — Stdlib native implementation
 - #751 — Package manager
-- #1004 — C library ecosystem compatibility
-- #1009 — Concurrency semantics
diff --git a/.aw/issues/closed/epic-tracking-mamba-package-manager-uv-like.md b/.aw/issues/closed/epic-tracking-mamba-package-manager-uv-like.md
index 678fe846..4bc1ca2b 100644
--- a/.aw/issues/closed/epic-tracking-mamba-package-manager-uv-like.md
+++ b/.aw/issues/closed/epic-tracking-mamba-package-manager-uv-like.md
@@ -10,77 +10,42 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-08T16:43:54Z
-updated_at: 2026-03-25T02:12:11Z
+updated_at: 2026-04-09T08:44:45Z
 ---
 
+## Goal
 
-## Problem
+Build a fast, native package manager for mamba, inspired by uv. Handles dependency resolution, installation, and virtual environments.
 
-A language without a package manager is an interesting toy, not a production tool. uv demonstrated that the existing Python packaging ecosystem (PyPI, wheels, pyproject.toml, lock files) can be made an order of magnitude faster simply by rewriting the client in Rust with serious concern for parallelism and caching. Mamba should inherit that lesson: keep the ecosystem compatibility (PyPI, wheels, pyproject.toml) and rewrite the client with Mamba's own native stack.
+## Key Features
 
-This epic tracks the design and delivery of that package manager — what we're calling (working title) `mamba` the CLI, which happens to also be the runtime. One tool, not two.
-
-## Goals
-
-- A native-Rust package manager that reads `pyproject.toml` and produces deterministic, reproducible environments from PyPI.
-- Startup and resolution speed comparable to uv — no "coffee break" installs.
-- First-class interop with the Mamba runtime: installing a package should make it immediately runnable by `mamba run`, with no separate activation dance.
-- Support both pure-Python packages (easy) and packages with C extensions (hard; bounded by #1004).
-- Usable as both a standalone CLI and a library that other tools (Conductor, SDD) can embed.
-
-## Sub-issues
-
-Phase 1 — Core resolver & installer
-
-- [ ] Package index client (PyPI-compatible)
-- [ ] Dependency resolver (PubGrub or equivalent)
-- [ ] Wheel installer
-- [ ] Lock file format (deterministic, human-diffable)
+### Phase 1 — Core
+- [ ] Package index client (PyPI-compatible registry)
+- [ ] Dependency resolver (PubGrub or similar algorithm)
+- [ ] Package installation (wheel format support)
+- [ ] Lock file format (deterministic builds)
 - [ ] Virtual environment creation and management
 
-Phase 2 — Developer experience
-
-- [ ] `mamba init` — project scaffolding
-- [ ] `mamba add <pkg>` / `mamba remove <pkg>`
+### Phase 2 — Developer Experience
+- [ ] `mamba init` — project scaffolding with pyproject.toml
+- [ ] `mamba add <pkg>` / `mamba remove <pkg>` — dependency management
 - [ ] `mamba install` — install from lock file
-- [ ] `mamba run <script>` — run with resolved environment
-- [ ] `mamba sync` — reconcile env with lock file
+- [ ] `mamba run <script>` — run with correct environment
+- [ ] `mamba sync` — sync environment to lock file
 
-Phase 3 — Advanced
-
-- [ ] Global package cache
+### Phase 3 — Advanced
+- [ ] Caching layer (global package cache)
 - [ ] Parallel downloads and installs
 - [ ] Cross-platform wheel selection
 - [ ] Source distribution builds
-- [ ] Workspace / monorepo support
+- [ ] Workspace support (monorepo)
 
-Specific child issues will be spun out as each phase begins. This epic is the umbrella until they exist.
-
-## Milestones
-
-1. **Resolver works** — given a `pyproject.toml`, produces a valid lock file for a non-trivial dependency graph (e.g. `requests`, `pydantic`, `rich`).
-2. **Install works** — lock file → populated virtualenv. Pure-Python packages only.
-3. **Runtime integration** — `mamba run` automatically uses the resolved environment, no `activate` script.
-4. **C extension story** — wheels with C extensions work for the Tier 1 set from #1004 (at minimum the ones with native-Rust replacements).
-5. **Speed target hit** — resolution and install time within 1.5x of uv on the same input.
-
-## Design Notes
-
-- **Lock file format**: start by studying uv's and Cargo's `Cargo.lock`. Diff-friendly and reproducible are non-negotiable.
-- **Virtualenv layout**: reuse PEP 405 layout for compatibility with tools that inspect `sys.prefix`.
-- **Resolver**: PubGrub is the obvious choice — it is what uv uses and its error messages are dramatically better than pip's legacy resolver.
-- **Config format**: `pyproject.toml` with Mamba-specific extensions under `[tool.mamba]`. Do not invent a new config file.
-
-## Out of Scope
-
-- Publishing packages to PyPI — consumption first, publishing later.
-- Replacing PyPI as an index — we consume it, we do not compete with it.
-- Pre-built binary distribution of Mamba itself — handled by the release pipeline, not the package manager.
-- C extension loading mechanics — that is #843 / #1004.
-- Windows support — Linux/macOS first.
+## Design Considerations
+- Written in Rust for speed (like uv)
+- Compatible with PyPI ecosystem
+- pyproject.toml as config format
+- Should work with both mamba and CPython packages
 
 ## References
-
-- [uv](https://github.com/astral-sh/uv) — the reference point
+- [uv](https://github.com/astral-sh/uv) — fast Python package manager in Rust
 - [PubGrub](https://github.com/dart-lang/pub/blob/master/doc/solver.md) — version solving algorithm
-- PEP 405, PEP 517, PEP 518, PEP 621 — packaging standards Mamba must respect
diff --git a/.aw/issues/closed/epic-tracking-mamba-py3-12-conformance-test-coverage.md b/.aw/issues/closed/epic-tracking-mamba-py3-12-conformance-test-coverage.md
index 7f542b94..037604f2 100644
--- a/.aw/issues/closed/epic-tracking-mamba-py3-12-conformance-test-coverage.md
+++ b/.aw/issues/closed/epic-tracking-mamba-py3-12-conformance-test-coverage.md
@@ -9,72 +9,33 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-08T16:43:51Z
-updated_at: 2026-03-10T06:35:03Z
+updated_at: 2026-04-09T08:44:46Z
 ---
 
+## Goal
 
-## Problem
-
-"Runs Python code" is a meaningless claim without a conformance definition. Mamba needs a reference — a specific Python version whose behavior we promise to match — and a systematic way to prove we match it. Py3.12 is that reference: modern enough to matter, stable enough to pin, narrow enough to be achievable. This epic tracks the work of building and maintaining the conformance test harness and pushing coverage upward toward "behaviorally indistinguishable from CPython 3.12 on the common path."
-
-This is separate from language-feature completeness (#851) and stdlib implementation (#749) in one important way: those two ask "does the feature exist?"; this one asks "does it behave identically to CPython?" A feature can be implemented and still fail conformance if its edge cases diverge.
-
-## Goals
-
-- A test harness that can be pointed at arbitrary Py3.12 behaviors and assert Mamba matches them.
-- Full coverage of the core runtime: arithmetic, comparisons, truthiness, object model, exceptions, generators, GC.
-- Full coverage of builtins and core data structures: `list`, `dict`, `set`, `tuple`, `str`, `bytes`, all builtin functions.
-- Stdlib API-level conformance: signatures, return types, error messages match CPython for the modules Mamba implements natively.
-- Zero silent divergence — every known deviation is either fixed or explicitly documented.
-
-## Current State
-
-- 1742 lib tests passing, 0 flaky.
-- Test isolation fixed (commit 21866c7).
-- No systematic Py3.12 conformance testing yet — tests today assert Mamba's behavior against itself, not against CPython.
+Ensure mamba core logic, builtins, and stdlib behave identically to CPython 3.12.
 
 ## Sub-issues
 
 ### Infrastructure
-
 - [ ] #752 — Py3.12 conformance test harness (P0)
 
-### Core runtime
-
-- [ ] #753 — `MbValue` arithmetic, comparison & truthiness (P0)
+### Core Runtime
+- [ ] #753 — MbValue arithmetic, comparison & truthiness (P0)
 - [ ] #754 — Object model: class, MRO, descriptors, metaclass (P0)
 - [ ] #755 — Exception hierarchy (P1)
 - [ ] #756 — Generator & iterator protocol (P1)
-- [ ] #757 — GC cycle detection & reference counting (P2) — test isolation fixed
-
-### Builtins & data structures
+- [ ] #757 — GC cycle detection & reference counting (P2) — test isolation fixed ✅
 
+### Builtins & Data Structures
 - [ ] #758 — Builtins: 108 tests → full Py3.12 verification (P0)
-- [ ] #759 — Data structure ops: `list`, `dict`, `set`, `tuple`, `str`, `bytes` (P1)
+- [ ] #759 — Data structure ops: list, dict, set, tuple, str, bytes (P1)
 
 ### Stdlib
-
 - [ ] #760 — Stdlib API signatures & return types (P2)
 
-## Milestones
-
-1. **Harness exists** (#752) — can run a Py3.12 test suite against Mamba and produce pass/fail diff.
-2. **Core runtime green** — all P0 runtime sub-issues closed.
-3. **Builtins green** — #758 closed, no xfail on builtin behavior.
-4. **Data-structure green** — #759 closed, all container ops match CPython.
-5. **Stdlib API green** — #760 closed for every natively implemented stdlib module from #749.
-6. **Continuous coverage** — new features ship with conformance tests by default; no more "test against ourselves."
-
-## Out of Scope
-
-- Individual language features (that is #851 and its children).
-- Stdlib module implementations themselves (that is #749).
-- Performance parity with CPython (tracked via the benchmark suite in #836).
-- Python 3.13 / 3.14 features — pin to 3.12 until 3.12 is green.
-- Error-message verbatim match — "equivalent meaning" is enough.
-
-## Related
-
-- #749 — Stdlib native implementation
-- #851 — Language features & tooling completeness
-- #1192 — Py3.12 conformance session progress (rolling ledger of sessions)
+## Current State
+- 1742 lib tests passing, 0 flaky
+- Test isolation fixed (commit 21866c7)
+- No systematic Py3.12 conformance testing yet
diff --git a/.aw/issues/closed/epic-tracking-mamba-stdlib-native-rust-implementation.md b/.aw/issues/closed/epic-tracking-mamba-stdlib-native-rust-implementation.md
index d1abd624..6d2421ce 100644
--- a/.aw/issues/closed/epic-tracking-mamba-stdlib-native-rust-implementation.md
+++ b/.aw/issues/closed/epic-tracking-mamba-stdlib-native-rust-implementation.md
@@ -9,73 +9,36 @@ labels:
 - type:epic
 - crate:mamba
 created_at: 2026-03-08T16:43:30Z
-updated_at: 2026-03-08T16:43:30Z
+updated_at: 2026-04-09T08:44:46Z
 ---
 
+## Goal
 
-## Problem
-
-Python's standard library is not optional — half of the "Python" experience is its stdlib, and no user thinks of `json`, `re`, `datetime`, `collections` as separate products. Mamba today has 72 stub modules: the API shape exists, but the implementations return `MbValue::none()` or fixed placeholder values. Any real program touching them breaks the moment it depends on actual behavior. The other 72 modules don't even have stubs.
-
-Rewriting 144 stdlib modules in native Rust is slow work, but it has a clear structure and an obvious unit of progress: one module at a time, each either fully implemented or explicitly deferred. This epic is the master tracker for that work. It is a scope container, not a design doc.
-
-## Goals
-
-- Every stdlib module Mamba ships is either (a) fully implemented in native Rust with Py3.12-compatible behavior, or (b) explicitly out of scope and documented as such.
-- "Fully implemented" means: real logic (not stubs), tests covering core API surface, behavioral parity with CPython verified by conformance tests (#750).
-- Modules are prioritized by usage impact — the "P1" set below is what actual code in the wild imports most often.
-- The work is parallelizable: each module is an independent sub-issue with no cross-module dependencies beyond Mamba's own type system.
+Rewrite all stdlib modules in native Rust with real logic (not stubs). Target: Py3.12-compatible behavior.
 
 ## Current State
 
-- **72 stub modules** exist under `crates/cclab-mamba/src/runtime/stdlib/` — API surface only, no real logic.
-- **72 missing modules** tracked as open issues (#652 – #738).
-- Total target: **144 modules**.
+- **72 stub modules** exist in `crates/cclab-mamba/src/runtime/stdlib/` — have API surface but return `MbValue::none()` or fixed values
+- **72 open issues** for modules not yet created (#652–#738)
+- Total: **144 modules** to fully implement
 
 ## Existing Stub Modules (72)
 
 abc, argparse, array, asyncio, base64, bisect, bz2, calendar, cmath, collections, configparser, contextlib, copy, csv, dataclasses, datetime, decimal, difflib, enum, fractions, functools, glob, gzip, hashlib, heapq, hmac, html_parser, http, inspect, io, itertools, json, locale, logging, lzma, math, numbers, operator, os, pathlib, pickle, platform, pprint, queue, random, re, secrets, shlex, shutil, signal, socket, sqlite3, statistics, string_constants, struct, subprocess, sys, tarfile, tempfile, textwrap, threading, time, traceback, typing, unicodedata, unittest, uuid, warnings, weakref, xml, zipfile, zlib
 
-These need their stubs replaced with real implementations. Existing stub does not count as "done."
+## New Modules Needed (72 open issues)
 
-## Sub-issues — Missing Modules (72 open issues)
+### P1 (16)
+#652 atexit, #653 gc, #654 types, #655 importlib, #656 codecs, #657 errno, #658 selectors, #661 ssl, #662 urllib, #663 email, #664 multiprocessing, #665 concurrent.futures, #666 tracemalloc, #667 dis, #668 ast, #669 tokenize
 
-### Priority 1 (16)
+### P2 (29)
+#670–#698 (fnmatch, fileinput, linecache, shelve, dbm, mimetypes, ipaddress, ftplib, smtplib, poplib, getopt, getpass, gettext, pdb, profile, pstats, timeit, doctest, sched, stat, keyword, site, sysconfig, pkgutil, runpy, reprlib, mmap, select, token)
 
-#652 `atexit`, #653 `gc`, #654 `types`, #655 `importlib`, #656 `codecs`, #657 `errno`, #658 `selectors`, #661 `ssl`, #662 `urllib`, #663 `email`, #664 `multiprocessing`, #665 `concurrent.futures`, #666 `tracemalloc`, #667 `dis`, #668 `ast`, #669 `tokenize`
+### P3 (27)
+#699–#738 remaining (socketserver, xmlrpc, wsgiref, webbrowser, venv, zipapp, zipimport, wave, curses, readline, pwd, grp, fcntl, pty, termios, tty, pipes, posixpath, pyclbr, tabnanny, stringprep, rlcompleter, plistlib, quopri, netrc, mailbox, imaplib)
 
-### Priority 2 (29)
+## Definition of Done
 
-#670 – #698: `fnmatch`, `fileinput`, `linecache`, `shelve`, `dbm`, `mimetypes`, `ipaddress`, `ftplib`, `smtplib`, `poplib`, `getopt`, `getpass`, `gettext`, `pdb`, `profile`, `pstats`, `timeit`, `doctest`, `sched`, `stat`, `keyword`, `site`, `sysconfig`, `pkgutil`, `runpy`, `reprlib`, `mmap`, `select`, `token`
-
-### Priority 3 (27)
-
-#699 – #738: `socketserver`, `xmlrpc`, `wsgiref`, `webbrowser`, `venv`, `zipapp`, `zipimport`, `wave`, `curses`, `readline`, `pwd`, `grp`, `fcntl`, `pty`, `termios`, `tty`, `pipes`, `posixpath`, `pyclbr`, `tabnanny`, `stringprep`, `rlcompleter`, `plistlib`, `quopri`, `netrc`, `mailbox`, `imaplib`
-
-## Milestones
-
-1. **Stub replacement, P0 core** — `sys`, `os`, `io`, `json`, `re`, `collections`, `functools`, `itertools`, `datetime`, `typing` fully implemented. These are the "can't run any program without them" modules.
-2. **P1 missing modules complete** — all 16 P1 new modules landed, most notably `ssl`, `urllib`, `email`, `ast`, `concurrent.futures`.
-3. **Stubs eliminated** — all 72 existing stubs have real implementations. Zero placeholders remaining.
-4. **P2 modules complete** — 29 P2 modules shipped.
-5. **Full parity** — every module with a reasonable implementation target is done. Remaining exceptions are explicitly documented.
-
-## Definition of Done (per module)
-
-- [ ] Real Rust implementation, not a stub.
-- [ ] Unit tests cover the core API surface.
-- [ ] Behavior matches Python 3.12 where that is reasonably achievable.
-- [ ] Conformance tests under #750 for the module pass.
-
-## Out of Scope
-
-- Modules whose entire purpose is CPython-specific internals (`_thread` locking primitives beyond the Mamba runtime, `_imp` import internals). Surface only what Python code actually imports.
-- Windows-only modules (`winreg`, `winsound`, `msvcrt`) — Linux/macOS first.
-- Modules whose upstream is deprecated in 3.12 (`distutils`, etc).
-- GPU / specialized hardware modules.
-
-## Related
-
-- #750 — Py3.12 conformance & test coverage (verifies behavior)
-- #851 — Language features & tooling (parser/runtime gaps that block stdlib work)
-- #1004 — C library ecosystem (some stdlib modules overlap with the C-extension story)
+- [ ] Module has real Rust implementation (not stubs)
+- [ ] Tests cover core API surface
+- [ ] Behavior matches Python 3.12
diff --git a/.aw/issues/closed/epic-user-facing-documentation-generation-project-type.md b/.aw/issues/closed/epic-user-facing-documentation-generation-project-type.md
index de12db7a..78149fea 100644
--- a/.aw/issues/closed/epic-user-facing-documentation-generation-project-type.md
+++ b/.aw/issues/closed/epic-user-facing-documentation-generation-project-type.md
@@ -9,10 +9,9 @@ labels:
 - type:epic
 - priority:p3
 created_at: 2026-03-24T04:26:26Z
-updated_at: 2026-03-24T04:26:26Z
+updated_at: 2026-04-09T08:44:18Z
 ---
 
-
 ## Problem
 
 Specs are machine contracts, not user-facing docs. `docs ≠ specs`. User-facing documentation needs:
diff --git a/.aw/issues/closed/refactor-extract-api-cli-commands-into-cclab-api-cli.md b/.aw/issues/closed/refactor-extract-api-cli-commands-into-cclab-api-cli.md
index 51941061..6107add0 100644
--- a/.aw/issues/closed/refactor-extract-api-cli-commands-into-cclab-api-cli.md
+++ b/.aw/issues/closed/refactor-extract-api-cli-commands-into-cclab-api-cli.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - type:refactor
 created_at: 2026-03-25T04:29:55Z
-updated_at: 2026-03-25T04:29:55Z
+updated_at: 2026-04-09T08:43:43Z
 ---
 
-
 ## Problem
 
 `api` CLI subcommands (serve, generate, codegen, init, etc.) are defined inline in `cclab-cli/src/api/` instead of following the `cclab-{name}-cli` auto-registration pattern.
diff --git a/.aw/issues/closed/refactor-extract-kv-cli-commands-into-cclab-kv-cli.md b/.aw/issues/closed/refactor-extract-kv-cli-commands-into-cclab-kv-cli.md
index e5004bd8..0d6a9f39 100644
--- a/.aw/issues/closed/refactor-extract-kv-cli-commands-into-cclab-kv-cli.md
+++ b/.aw/issues/closed/refactor-extract-kv-cli-commands-into-cclab-kv-cli.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - type:refactor
 created_at: 2026-03-25T04:29:58Z
-updated_at: 2026-03-25T04:29:58Z
+updated_at: 2026-04-09T08:43:43Z
 ---
 
-
 ## Problem
 
 `kv` CLI subcommands are defined inline in `cclab-cli/src/kv.rs` instead of following the `cclab-{name}-cli` auto-registration pattern.
diff --git a/.aw/issues/closed/refactor-extract-qc-cli-commands-into-cclab-qc-cli.md b/.aw/issues/closed/refactor-extract-qc-cli-commands-into-cclab-qc-cli.md
index 301f0e54..bd2a3837 100644
--- a/.aw/issues/closed/refactor-extract-qc-cli-commands-into-cclab-qc-cli.md
+++ b/.aw/issues/closed/refactor-extract-qc-cli-commands-into-cclab-qc-cli.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - type:refactor
 created_at: 2026-03-25T04:30:02Z
-updated_at: 2026-03-25T04:30:02Z
+updated_at: 2026-04-09T08:43:42Z
 ---
 
-
 ## Problem
 
 `qc` CLI subcommands are defined inline in `cclab-cli/src/qc/` instead of following the `cclab-{name}-cli` auto-registration pattern.
diff --git a/.aw/issues/closed/refactor-extract-queue-cli-commands-into-cclab-queue-cli.md b/.aw/issues/closed/refactor-extract-queue-cli-commands-into-cclab-queue-cli.md
index 207843c3..9df9386b 100644
--- a/.aw/issues/closed/refactor-extract-queue-cli-commands-into-cclab-queue-cli.md
+++ b/.aw/issues/closed/refactor-extract-queue-cli-commands-into-cclab-queue-cli.md
@@ -10,10 +10,9 @@ labels:
 - priority:p2
 - type:refactor
 created_at: 2026-03-25T04:30:00Z
-updated_at: 2026-03-25T04:30:00Z
+updated_at: 2026-04-09T08:43:42Z
 ---
 
-
 ## Problem
 
 `queue` CLI subcommands are defined inline in `cclab-cli/src/queue.rs` instead of following the `cclab-{name}-cli` auto-registration pattern.
diff --git a/.aw/issues/closed/refactor-extract-razer-cli-commands-into-cclab-razer-cli.md b/.aw/issues/closed/refactor-extract-razer-cli-commands-into-cclab-razer-cli.md
index 74633b52..fa577dab 100644
--- a/.aw/issues/closed/refactor-extract-razer-cli-commands-into-cclab-razer-cli.md
+++ b/.aw/issues/closed/refactor-extract-razer-cli-commands-into-cclab-razer-cli.md
@@ -10,10 +10,9 @@ labels:
 - type:refactor
 - crate:razer
 created_at: 2026-03-25T04:30:31Z
-updated_at: 2026-03-25T04:30:31Z
+updated_at: 2026-04-09T08:43:42Z
 ---
 
-
 ## Problem
 
 `razer` CLI subcommands are defined inline in `cclab-cli/src/razer.rs` instead of following the `cclab-{name}-cli` auto-registration pattern.
diff --git a/.aw/issues/closed/refactor-extract-sdd-cloud-agents-into-separate-crate.md b/.aw/issues/closed/refactor-extract-sdd-cloud-agents-into-separate-crate.md
index 5e586c1c..fb24c590 100644
--- a/.aw/issues/closed/refactor-extract-sdd-cloud-agents-into-separate-crate.md
+++ b/.aw/issues/closed/refactor-extract-sdd-cloud-agents-into-separate-crate.md
@@ -10,10 +10,9 @@ labels:
 - type:refactor
 - crate:cclab-agent
 created_at: 2026-03-23T06:39:12Z
-updated_at: 2026-03-23T07:35:06Z
+updated_at: 2026-04-09T08:44:19Z
 ---
 
-
 ## Problem
 
 `cclab-agent` currently mixes two concerns:
diff --git a/.aw/issues/open/bug-score-init-missing-5-skill-templates-handoff-takeo.md b/.aw/issues/open/bug-score-init-missing-5-skill-templates-handoff-takeo.md
deleted file mode 100644
index ae97c3b1..00000000
--- a/.aw/issues/open/bug-score-init-missing-5-skill-templates-handoff-takeo.md
+++ /dev/null
@@ -1,63 +0,0 @@
----
-type: bug
-title: 'fix: aw init missing 5 skill templates (handoff, takeoff, build-debug, release-patch, mamba-test-coverage)'
-state: draft
-id: 470a08ce-7e36-48ab-9be7-a580b2487292
-labels:
-- crate:score
-- type:bug
-phase: change_implementation_created
-branch: cclab/bug-score-init-missing-5-skill-templates-handoff-takeo
-git_workflow: worktree
----
-
-
-
-
-## Problem
-
-`aw init` installs skills via `install_skills()` in `projects/agentic-workflow/cli/src/init.rs`. Currently only 9 skills are installed: score-run-change, score-fillback-main-specs, score-codex-review, score-gemini-explore-specs, score-gemini-explore-codebase, score-merge, score-revise-artifact, score-issue, score-issue-patrol. Five skills that exist on disk in `.claude/skills/` are NOT installed by `aw init`:
-1. `score-handoff` — handoff session continuity
-2. `score-takeoff` — resume from handoff
-3. `score-build-debug` — build debug + install (has scripts/build.sh)
-4. `score-release-patch` — bump patch version + build (has scripts/release.sh)
-5. `score-mamba-test-coverage` — test coverage analysis (has scripts/coverage.sh)
-
-Template SKILL.md files already exist in `projects/agentic-workflow/cli/templates/mainthread/skills/` for all 5 missing skills, but they lack `scripts/` subdirectories (needed for build-debug, release-patch, mamba-test-coverage). The current `install_claude_skills()` only writes `SKILL.md`, not scripts. Root cause: these skills were added manually after the init system was built and never wired into `init.rs`.
-
-## Requirements
-
-- R1: Add `include_str!` constants (`SKILL_HANDOFF`, `SKILL_TAKEOFF`, `SKILL_BUILD_DEBUG`, `SKILL_RELEASE_PATCH`, `SKILL_MAMBA_TEST_COVERAGE`) referencing the 5 template SKILL.md files
-- R2: Add the 5 missing skills to the `skills` vec in `install_claude_skills()` so they are installed by `aw init`
-- R3: Add `scripts/build.sh`, `scripts/release.sh`, and `scripts/coverage.sh` template files under their respective skill directories in `projects/agentic-workflow/cli/templates/mainthread/skills/`
-- R4: Extend `install_claude_skills()` to check for and install a `scripts/` subdirectory alongside `SKILL.md` for each skill that has one (copy scripts with executable permissions, `chmod +x`)
-- R5: Running `aw init` on an already-initialized project updates all 14 skills (9 existing + 5 new) without data loss
-
-## Scope
-
-### In-scope
-- `projects/agentic-workflow/cli/src/init.rs` — add constants + extend install logic for SKILL.md and scripts/
-- `projects/agentic-workflow/cli/templates/mainthread/skills/score-handoff/SKILL.md` — already exists, wire in
-- `projects/agentic-workflow/cli/templates/mainthread/skills/score-takeoff/SKILL.md` — already exists, wire in
-- `projects/agentic-workflow/cli/templates/mainthread/skills/score-build-debug/SKILL.md` — already exists; add `scripts/build.sh`
-- `projects/agentic-workflow/cli/templates/mainthread/skills/score-release-patch/SKILL.md` — already exists; add `scripts/release.sh`
-- `projects/agentic-workflow/cli/templates/mainthread/skills/score-mamba-test-coverage/SKILL.md` — already exists; add `scripts/coverage.sh`
-
-### Out-of-scope
-- Modifying any existing installed skill content
-- Changes to agent definitions, hooks, or settings.json
-- Adding new skills beyond the 5 listed
-
-## Reference Context
-
-### Related Specs
-| Spec | Relevance | Key Requirements |
-|------|-----------|------------------|
-| projects/agentic-workflow/specs/init-command.md | high | Defines install_claude_skills() contract, R12/R13 added score-issue and score-issue-patrol; same pattern applies for the 5 missing skills; Changes section documents the include_str! + skills vec pattern |
-| .aw/tech-design/crates/cclab-cli/claude/skills.md | medium | Defines SKILL.md structure including optional scripts/ subdirectory with execute permissions; documents multi-file skill layout |
-| .aw/changes/score-handoff-takeoff/groups/default/specs/score-handoff-takeoff-spec.md | high | Specifies score-handoff and score-takeoff CLI commands and their skill wrappers (R6: install SKILL.md templates); confirms these skills were implemented but not wired into init |
-
-### Spec Plan
-| Spec ID | Action | Main Spec Ref | Sections |
-|---------|--------|---------------|----------|
-| score-init-command | modify | projects/agentic-workflow/specs/init-command.md | requirements, changes |
diff --git a/.aw/issues/open/enhancement-add-advanced-filtering-for-issues-priority-labels.md b/.aw/issues/open/enhancement-add-advanced-filtering-for-issues-priority-labels.md
deleted file mode 100644
index 1e5d7594..00000000
--- a/.aw/issues/open/enhancement-add-advanced-filtering-for-issues-priority-labels.md
+++ /dev/null
@@ -1,25 +0,0 @@
----
-type: enhancement
-title: Add advanced filtering for issues (priority, labels, author)
-state: open
-github_id: 1073
-url: https://github.com/chrischeng-c4/cclab/issues/1073
-author: chrischeng-c4
-labels:
-- type:enhancement
-- priority:p3
-- project:conductor
-created_at: 2026-03-24T07:32:57Z
-updated_at: 2026-03-24T07:32:57Z
----
-
-## Problem
-Issue list only supports status + project filter. No filtering by priority, labels, author, or date range.
-
-## Solution
-- MUI Chip-based filter bar
-- Priority filter (dropdown)
-- Label filter (multi-select)
-- Author filter
-- Date range (created_at)
-- API: extend list endpoints with filter query params
diff --git a/.aw/issues/open/enhancement-add-global-toast-notification-system-mui-snackbar.md b/.aw/issues/open/enhancement-add-global-toast-notification-system-mui-snackbar.md
deleted file mode 100644
index 52b927a5..00000000
--- a/.aw/issues/open/enhancement-add-global-toast-notification-system-mui-snackbar.md

... truncated (12336 more lines)
```

## Review: enhancement-resolver-conditional-exports-import-require-browse-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: enhancement-resolver-conditional-exports-import-require-browse

**Summary**: Implementation matches all spec requirements (R1-R5). Hard checklist passes: code matches spec, 16 resolver tests present (including T1-T10 from test plan), all 742 cclab-jet tests pass with 0 failures. Key changes: (1) ResolveOptions.conditions field with dev-mode defaults, (2) resolve_export_value iterates object keys in JSON insertion order using conditions as membership filter (Node.js PACKAGE_EXPORTS_RESOLVE compliant), (3) recursive descent into nested condition objects, (4) ResolveConfig in config.rs for build-mode overrides. Workspace Cargo.toml updated with serde_json preserve_order feature to guarantee insertion-order iteration.

