Fix bundler for Nx monorepos: 1) resolver walks up to root node_modules (#962), 2) circular deps use runtime wrappers instead of bail (#962). Test on tech-platform.

## Revision (epoch: 1773986578)

Fix 'Missing local name in export specifier' error when building shared-ui-form-inputs. The bundler's import/export parser (imports.rs) fails on certain export syntax patterns. Also need to handle circular dependencies gracefully — keep cyclic modules in runtime wrappers instead of bailing.


## Revision (epoch: 1773987675)

Add externals support for lib builds: when building a lib project (not an app), default to externalizing all node_modules dependencies. This prevents libs from bundling React, antd, lodash etc. into their output (kidzania-lib was 6.6MB). Also add --externals CLI flag for manual control.


## Revision (epoch: 1773993208)

Fix remaining build failures: 1) Use project.json projectType for lib/app detection instead of path heuristic only. 2) Add build timeout (30s per project) to prevent hanging on huge bundles. 3) Parallel project builds within each topological level.
