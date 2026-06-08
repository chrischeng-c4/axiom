# Cue Generated App Repository Template

This directory is the fixture template for hidden generated-app repositories.
Business users do not see this repository shape; Cue writes and reads it through
Registry, Admin, and release evidence.

Required files:

- `app-spec.json`
- `policy.json`
- `permissions.json`
- `connectors.json`
- `.gitlab-ci.yml`
- `tests/permission-tests.json`
- `tests/workflow-tests.json`
- `tests/policy-tests.json`
- `generated/runtime-config.json`
- `generated/ui-manifest.json`
- `releases/`
