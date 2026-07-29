# UAF detector process service topology

Issue: #3005
Parent inventory: #2968
Source revision: `5b9b8c922f`

This Stage 1 slice classifies the debug-only UAF detector's two mutable
process statics. The detector is a process diagnostic service, not interpreter
execution state. This design gives that service one explicit aggregate without
changing `src/**`.

## Bounded context

```text
Process
└── RuntimeDiagnostics
    └── UafDetectorProcessService
        ├── armed: AtomicBool
        └── env_checked: Once

ExecutionContext / OS worker / MbObject
└── consumes the process detector policy
```

`RuntimeDiagnostics` owns the detector for the process lifetime. Execution
contexts, OS workers, tests, and objects may invoke it, but none owns or resets
its state.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `UafDetectorProcessService` | process service | one debug-build detector |
| `DetectorArmed` | monotonic atomic value | `false`, then optionally `true` |
| `EnvironmentSample` | once-only initialization event | first read of `MAMBA_UAF_DETECTOR` |
| `PointerValidation` | diagnostic operation | alignment and object-kind checks |

The exact target destination is:

```rust
#[cfg(debug_assertions)]
struct UafDetectorProcessService {
    armed: std::sync::atomic::AtomicBool,
    env_checked: std::sync::Once,
}

#[cfg(debug_assertions)]
static UAF_DETECTOR: UafDetectorProcessService = UafDetectorProcessService {
    armed: std::sync::atomic::AtomicBool::new(false),
    env_checked: std::sync::Once::new(),
};
```

The aggregate consolidates the current two statics; it does not add a new
semantic owner or another layer of mutable process state.

## Frozen inventory

The two production diagnostic identities have sorted newline-terminated
SHA-256
`fe68f1c6ee5af296eb491f5ae9017daf32ff3fcee1680916ee1c6bbe3f7f7176`.
There are zero test-only identities within this admitted slice.

| Current symbol | Storage | Role | Target disposition |
|---|---|---|---|
| `UAF_DETECTOR_ARMED` | debug-only process `AtomicBool` | monotonic detector policy | `UAF_DETECTOR.armed` |
| `UAF_DETECTOR_ENV_CHECKED` | debug-only process `Once` | once-only environment sample | `UAF_DETECTOR.env_checked` |

The accepted selector contains 40 physical rows:

| Family | Occurrences |
|---|---:|
| `UAF_DETECTOR_ARMED` | 4 |
| `UAF_DETECTOR_ENV_CHECKED` | 2 |
| `MAMBA_UAF_DETECTOR` | 4 |
| `parse_uaf_detector_env` | 8 |
| `uaf_detector_armed` | 5 |
| `force_arm_uaf_detector` | 7 |
| `debug_validate_obj` | 10 |
| **Total** | **40** |

The rows comprise comments, two state declarations, helper declarations and
calls, two production validation calls, and unit-test calls. Comments and
function definitions are selector evidence, not additional state identities.

## Current behavior

`mb_retain` and `mb_release` call `debug_validate_obj` in debug builds. The
first call enters `Once::call_once`, samples `MAMBA_UAF_DETECTOR`, and sets the
atomic flag when the value is nonempty and not `"0"`. Every successful call
then reads the flag.

`force_arm_uaf_detector` can store `true` before or after the environment
sample. The environment-disabled branch never writes `false`, so neither
ordering can disarm an already armed service. Multiple `true` stores are
allowed; the observable value can transition from `false` to `true` only once.

When disarmed, pointer validation is a semantic no-op after the policy check.
It is not zero-overhead in a debug build: the first call performs once-only
initialization and environment access, and later calls still reach the helper
and atomic load. Release builds compile out the detector calls and state.

The public `Once::call_once` contract supplies once-only successful
initialization and makes concurrent callers wait for an in-progress
initializer. This design makes no claim about an internal mutex, lock freedom,
wait freedom, fairness, or exact cost.

## Lifecycle matrix

| Boundary | Result |
|---|---|
| debug process start | `armed=false`; environment not sampled |
| first validation | sample environment once; return current flag |
| environment disabled | remain false; later reads still pay debug check |
| environment enabled | store true; validation remains armed |
| force before sample | store true; later disabled sample cannot disarm |
| force after sample | store true after the initial false/true result |
| concurrent sample | one successful initializer; other callers wait |
| concurrent force/read | atomic flag has no data race; false-to-true is monotonic |
| runtime cleanup | no reset or disarm; state is unchanged |
| execution-context retirement | unchanged; context is not the owner |
| OS-worker exit | unchanged; state is not TLS |
| same-process test continuation | an earlier force-arm remains visible |
| release build | service and validation calls are absent |
| process exit | operating system retires the static process state |

Tests that need both initially disarmed and initially environment-armed states
must use separate processes. Adding a reset only for test convenience would
weaken the production invariant and is forbidden.

## Diagnostic-attribution boundary

`CURRENT_TEST_NAME` is not a test-only identity. It is separate debug-only
process diagnostic attribution state written by
`tests/cpython_ported/harness.rs` so a worker-thread panic can name the outer
integration test.

It remains out of #3005 because attribution has a mutable string, lock/failure
policy, overwrite lifecycle, and cleanup questions distinct from detector
arming. It requires its own owner slice under #2968.

## Source implementation slice

Exact primary changed path:

- `projects/mamba/src/runtime/rc.rs`

The future implementation may:

1. add `UafDetectorProcessService` and its exact initialized process static;
2. move the environment-sampling and force-arm behavior behind aggregate
   methods;
3. route `uaf_detector_armed` and `force_arm_uaf_detector` through the
   aggregate without changing their externally observed behavior.

Required invariants:

1. the initial flag is false;
2. the environment is sampled at most once per process;
3. any force-arm or enabled sample makes the observable flag permanently true;
4. force-before-sample and force-after-sample converge on true;
5. all workers and execution contexts observe the one process policy;
6. runtime cleanup and context retirement do not reset it;
7. debug pointer checks retain their current alignment/kind behavior;
8. release builds contain neither state nor validation calls;
9. no new allocation or blocking operation is added to the steady-state check.

Forbidden changes:

- a reset or disarm method;
- ownership by `ExecutionContext`, TLS, an object, or a test;
- separate detector instances per worker/context;
- repeated environment sampling;
- changing the accepted environment-value parser;
- weakening the positive pointer-validation controls;
- moving diagnostic code into release builds;
- claiming undocumented synchronization or performance properties.

## Verification gates

- Exact-set gate: both identities and all 40 selector rows reconcile until the
  consolidation lands.
- Release-absence gate: release artifacts contain no detector state or calls.
- Environment-disabled gate: a clean child process samples unset, empty, and
  `"0"` as false.
- Environment-enabled gate: clean child processes sample representative
  nonempty/non-`"0"` values as true.
- Force-before-sample gate: a clean child forces first, then samples a disabled
  environment, and remains true.
- Force-after-sample gate: a clean child samples false, forces, and then reads
  true.
- Concurrent gate: barrier-controlled readers, one initializer, and force
  calls preserve data-race freedom and monotonic observed state.
- Worker/context visibility gate: all workers and contexts consume the same
  process policy.
- Cleanup gate: runtime cleanup and context retirement leave the policy
  unchanged.
- Existing unit tests named in the frozen rows remain regression seams; AGY's
  measure-only run did not execute them.

## Dependency and dispatcher result

- #3005 is a Stage 1 classification slice under #2968.
- #2968 must close before Stage 2 #2839 can be dispatched.
- AGY's first report reconciled the exact set but conflated semantic no-op with
  zero overhead, misclassified `CURRENT_TEST_NAME`, asserted an undocumented
  `Once` implementation, and omitted the implementation/test slice.
- The resumed report corrected those claims and passed independent snapshot
  verification. This is accepted after one revision and is not a one-pass ramp
  sample.
