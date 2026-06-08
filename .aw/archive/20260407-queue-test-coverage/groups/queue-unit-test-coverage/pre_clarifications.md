---
change: queue-test-coverage
group: queue-unit-test-coverage
date: 2026-04-04
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Should we also add integration tests requiring running infrastructure (NATS/Redis), or focus purely on unit tests?
- **Answer**: Both. Add unit tests AND integration tests. Integration tests that need NATS/Redis should use real services.

### Q2: General
- **Question**: For cloud_scheduler_backend and external API modules, how should we handle tests?
- **Answer**: For Google Cloud: use homebrew-installable emulators (e.g. google-cloud-sdk emulators). For services that can be started locally via homebrew, use those real local services instead of mocking.

### Q3: General
- **Question**: Should metrics tests require the 'metrics' feature flag?
- **Answer**: Test both paths: test metric recording under #[cfg(feature="metrics")] AND test the no-op path without the feature flag.

