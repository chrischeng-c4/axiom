# Task: Code Review (Iteration {{iteration}})

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Instructions

1. **Get requirements**:
   - Read all specs and tasks

2. **Get implementation summary**:
   - Review changed files against requirements

3. **Review focus**:
   - Test results (are all tests passing?)
   - Security (any vulnerabilities?)
   - Best practices (performance, error handling)
   - Requirement compliance (does code match specs?)

4. **Submit review**:
   - Use `score artifact review-implementation` with findings

## Severity Guidelines
- **HIGH**: Failing tests, security issues, missing features
- **MEDIUM**: Style issues, missing tests, minor improvements
- **LOW**: Suggestions, nice-to-haves

## Verdict Guidelines
- **APPROVED**: All tests pass, no HIGH issues
- **REVIEWED**: Some issues exist (fixable)
- **REJECTED**: Critical problems

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"requirements"}'
score workflow list-changed-files {{change_id}}
```

### Generate Artifact
```
score artifact review-implementation {{change_id}} <payload_path>
```
