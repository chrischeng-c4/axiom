// HANDWRITE-BEGIN gap="vat-headless-docker-command-shim" tracker="#1685" reason="Apple Container exposes a public CLI rather than a Docker Engine socket. This small multicall shim must validate Docker-shaped argv before invoking that CLI so unsupported Engine semantics never silently widen into a misleading best effort."
//! Opt-in, fail-closed `docker` command compatibility over Apple Container.
//!
//! `vat docker install-shim` creates a `docker -> vat` symlink.  When the
//! executable is invoked through that name, [`run_from_env`] receives raw
//! Docker-shaped argv before Clap parses VAT's normal CLI and translates only
//! the explicitly supported subset to Apple's public `container` CLI.
//!
//! This is deliberately *not* a Docker Engine implementation: it exposes no
//! socket/API and refuses Engine-oriented commands (`info`, `version`,
//! `context`, SDKs, Testcontainers, and unknown flags) before any runtime
//! process starts. Its three Compose profiles are intentionally much narrower
//! than general Docker Compose and are captured before VAT starts a runtime.

use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitCode, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

/// Short, stable command corpus surfaced by `docker --help` when the shim is
/// installed.  Keep this list narrow: it is a promise that each verb is
/// translated intentionally, rather than an implication of Docker parity.
pub const SUPPORTED_COMMANDS: &str = "build (including strict bounded VAT JSON receipts), pull (including strict bounded VAT JSON receipts), push, run (including strict bounded ephemeral VAT JSON one-shots), create, ps (including strict Apple-native JSON inventory), images (including strict Apple-native JSON inventory), logs (including strict bounded VAT JSON snapshots), exec, inspect (including strict Apple-native container JSON), start, stop, kill, rm, cp, login, logout, image (including strict Apple-native image inspect JSON), container, network, volume, stats (non-streaming Apple-native JSON only), system (strict Apple-native JSON disk report only), compose (strict single-service and host-facing-independent-v1 profiles)";

/// The narrow Compose forms advertised by `docker --help`. Keep JSON modes
/// explicit here so an agent does not discover the parser's stricter exec
/// contract only after attempting a raw-output invocation.
const STRICT_COMPOSE_HELP: &str = "The supported Compose profiles are only: one literal-image service; one literal-short-build service with up -d --build; or 2-4 literal-image services with x-vat-compose-profile: host-facing-independent-v1, each independently published to a unique loopback host port. Service-name DNS, dependencies, networks, and volumes are unavailable. Use docker compose --dry-run -f FILE -p PROJECT up -d [--build] for a file/profile-only preflight: it does not import, build, or start anything, and the returned launch argv revalidates the file. Then use docker compose -p PROJECT ps [--format json]|logs [--format json [--tail LINES]] SERVICE|exec -T SERVICE -- COMMAND|exec -T --format json SERVICE -- COMMAND (or --format=json)|down. Exact JSON ps, logs, and exec forms use VAT's schemas, not Docker Compose's output schemas.";

/// The intentionally exact one-shot stats form advertised by `docker --help`.
/// Familiar flags select a constrained Apple-native observation; they never
/// imply Docker Engine stats schema or a health/liveness claim.
const STRICT_STATS_HELP: &str = "Stats accepts only `docker stats --no-stream --format json CONTAINER [CONTAINER...]` (or --format=json). It returns one validated Apple Container JSON document unchanged under a bounded observation deadline; it is read-only and is not Docker Engine stats schema, ownership, health, or liveness proof.";

/// The strict global disk-report form. It is intentionally success-only: a
/// child nonzero status suppresses all stdout so stale/partial native output
/// cannot be mistaken for one current global evidence document.
const STRICT_SYSTEM_DF_HELP: &str = "Global disk reporting accepts only `docker system df --format json` (or --format=json). VAT invokes only `container system df --format json`, returns one validated opaque Apple Container JSON document unchanged only after a successful bounded observation, and suppresses stdout on child failure. The report is shared Apple Container disk evidence only: it is not Docker Engine schema, ownership, per-image attribution, reclaimability/action, cleanup/prune, health/readiness, or a secret-redaction guarantee.";

/// The narrow direct-container inventory form advertised by `docker --help`.
/// This deliberately retains Apple Container's opaque JSON instead of
/// pretending to provide Docker Engine's list schema or object semantics.
const STRICT_PS_HELP: &str = "Agent inventory accepts only `docker ps --format json` (or --format=json), optionally with exactly one `--all` or `-a`; `docker container ls` and `docker container list` accept the same form. It returns one validated Apple Container JSON document unchanged under a bounded observation deadline; it is not Docker Engine schema, ownership, health, readiness, or liveness proof.";

/// The strict image-list inventory form advertised by `docker --help`. It has
/// no selector/filter surface, so agents cannot mistake an Apple-native list
/// for Docker Engine image matching or registry/build proof.
const STRICT_IMAGES_HELP: &str = "Image inventory accepts only `docker images --format json` (or --format=json); `docker image ls` and `docker image list` accept the same form. It returns one validated Apple Container JSON document unchanged under a bounded observation deadline; it is not Docker Engine image schema, ownership, provenance, security, executability, registry, build-readiness, health, readiness, or liveness proof.";

/// The one strict direct-container inspect form advertised by `docker --help`.
/// It deliberately excludes Docker's polymorphic inspect behavior and never
/// rewrites or redacts Apple's native object document.
const STRICT_INSPECT_HELP: &str = "Container inspect accepts only `docker inspect --format json CONTAINER` (or --format=json); `docker container inspect` accepts the same form. It returns one validated Apple Container JSON document unchanged under a bounded observation deadline; it is not Docker Engine inspect schema; ownership, provenance, security, image identity, registry, or build-status proof; health/readiness/liveness/port-reachability proof; or a secret-redaction guarantee.";

/// The one strict direct-container log snapshot form advertised by `docker
/// --help`. Apple Container exposes only textual stdio output here, so this is
/// deliberately a bounded VAT document rather than a Docker/Apple log schema.
const STRICT_LOGS_HELP: &str = "Agent log snapshots accept only `docker logs --format json --tail LINES CONTAINER` (or --format=json/--tail=LINES); `docker container logs` accepts the same form. LINES must be 1..=1000. VAT invokes only `container logs -n LINES CONTAINER` and returns one bounded vat.docker.logs.v1 document; its apple_container_stdio is untrusted content, not Docker Engine logs schema or multiplex/demux, and is not ownership/provenance/security/image/registry/build-status, health/readiness/liveness/port-reachability, or secret-redaction proof. --follow, --boot, timestamps, and all other log selectors are unsupported in JSON mode.";

/// The direct exec JSON form is intentionally foreground-only and bounded.
/// It does not turn the shim into an interactive process supervisor or imply
/// Docker Engine stream, TTY, ownership, readiness, or redaction semantics.
const STRICT_EXEC_HELP: &str = "Agent exec snapshots accept only `docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...]` (or --format=json/--timeout=SECONDS); `docker container exec` accepts the same form. SECONDS must be 1..=1200. The Docker-facing literal `--` is required and stripped after validation; VAT invokes only `container exec CONTAINER COMMAND [ARG...]`, bounds the host Apple Container client observation, and returns one vat.docker.exec.v1 document; it does not claim to terminate a guest process. TTY, interactive, detach, env, user, workdir, and all other exec flags are unsupported in JSON mode; stdout/stderr are untrusted command output, not Docker Engine stream semantics, ownership/readiness/health proof, or a secret-redaction guarantee.";

/// The direct run JSON form is intentionally an ephemeral, foreground-only
/// one-shot. VAT owns the generated name and label so every exit path can
/// clean up precisely without accepting caller-provided lifecycle selectors.
const STRICT_RUN_HELP: &str = "Agent one-shot runs accept only `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` (or --format=json/--timeout=SECONDS). SECONDS must be 1..=1200; the selectors may be reordered before IMAGE. VAT rejects detach, TTY, interactive, caller names/labels, ports, networks, mounts, env, and every other run option before starting Apple Container; while VAT remains running it generates an owner-labeled foreground container, bounds the host Apple Container client observation, confirms exact-container cleanup, and returns one vat.docker.run.v1 document. This is not Docker Engine parity and a client timeout does not claim guest-wide process termination or crash-recovery cleanup.";

/// The direct build receipt is intentionally bounded and non-owning. It
/// retains the caller-selected image and never turns a host-client deadline
/// into a builder cancellation or rollback promise.
const STRICT_BUILD_HELP: &str = "Build receipts accept only `docker build --format json --timeout SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform PLATFORM] [--label K=V ...] CONTEXT` (or documented equals forms). SECONDS must be 1..=1200 and CONTEXT must be one existing local directory. VAT invokes only public `container build` with its JSON/deadline selectors stripped, then returns one bounded vat.docker.build.v1 receipt after the Apple client exits; images are retained with no auto-cleanup. Build args, labels, and output are opaque/untrusted; this is not Docker Engine/API, provenance, ownership, readiness, security, secret-redaction, cancellation, or rollback proof.";

/// The direct pull receipt is deliberately non-owning and does not turn a
/// host-client deadline or a child exit into a registry/download/image-state
/// assertion. It captures arbitrary backend output only inside VAT's wrapper.
const STRICT_PULL_HELP: &str = "Pull receipts accept only `docker pull --format json --timeout SECONDS IMAGE` (or documented equals forms). SECONDS must be 1..=1200; the selectors may be reordered before one safe opaque IMAGE reference. VAT invokes only public `container image pull IMAGE` with its JSON/deadline selectors stripped, then returns one bounded vat.docker.pull.v1 receipt after the Apple client exits; images are never owned or cleaned up by VAT. Pull output is opaque/untrusted; this is not Docker Engine/API, registry management, provenance, digest/platform/freshness, image-state, ownership, security, secret-redaction, cancellation, download-completion, or rollback proof.";

/// The direct image-inspect JSON form remains an opaque Apple-native document.
/// It deliberately does not promise Docker inspect schema or image trust.
const STRICT_IMAGE_INSPECT_HELP: &str = "Image metadata inspection accepts only `docker image inspect --format json IMAGE` (or --format=json). VAT invokes only `container image inspect IMAGE`, returns one validated opaque Apple Container JSON document under a bounded observation deadline, and does not implement Docker's polymorphic inspect/template behavior. The result is not Docker Engine schema; ownership, provenance, security, registry, pull/build completion, executability, health/readiness/liveness proof; or a secret-redaction guarantee.";

/// Whether this process was invoked through the opt-in `docker` symlink.
pub fn invoked_as_docker() -> bool {
    env::args_os()
        .next()
        .and_then(|argv0| Path::new(&argv0).file_name().map(OsStr::to_owned))
        .as_deref()
        == Some(OsStr::new("docker"))
}

/// Run the compatibility shim with the raw argv that followed the `docker`
/// executable name. Most mapped commands keep inherited child stdio and their
/// numeric exit code; the strict bounded native-JSON observations capture and
/// validate Apple Container output before replaying it.
pub fn run_from_env() -> Result<ExitCode> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if let Some(code) = shim_meta_command(&args)? {
        return Ok(code);
    }

    if args.first().and_then(|arg| arg.to_str()) == Some("compose") {
        return run_compose(&args[1..]);
    }

    // `stats` is intentionally not a normal translated command. Even with
    // `--no-stream`, the Apple Container CLI can hang; route the one supported
    // JSON observation through a deadline plus captured/validated stdout.
    if args.first().and_then(|arg| arg.to_str()) == Some("stats") {
        return run_stats(&args[1..]);
    }

    // `system df` has no inherited compatibility translation. Its one JSON
    // selector form is read-only, bounded, and success-only; raw `docker
    // system df` remains unsupported by the ordinary fail-closed dispatcher.
    if let Some(request) = docker_system_df_json_request_from_argv(&args)? {
        return run_system_df_json(&request);
    }

    // Direct build has one explicit bounded receipt form. Existing raw build
    // translations remain below; the strict parser owns an argv only after a
    // JSON/deadline selector appears so it can fail before any builder starts.
    if let Some(request) = docker_build_json_request_from_argv(&args)? {
        return run_build_json(&request);
    }

    // Direct pull has one explicit bounded receipt form. Existing raw pull
    // translations remain below; the strict parser owns an argv after either
    // selector appears so a late/malformed selector cannot reach Apple.
    if let Some(request) = docker_pull_json_request_from_argv(&args)? {
        return run_pull_json(&request);
    }

    // The JSON form of `ps` is likewise intentionally not a normal
    // translated command. Its Apple-native output must be bounded and fully
    // validated before replay, while the pre-existing text/quiet `ps` forms
    // keep their inherited child-stdio behavior below.
    if let Some(request) = docker_ps_json_request_from_argv(&args)? {
        return run_ps_json(&request);
    }

    // Image inventory follows the same bounded native-JSON rule as the
    // direct container inventory above, but only for the three documented
    // image-list aliases. Existing text and quiet image-list paths continue
    // through the ordinary translator below.
    if let Some(request) = docker_images_json_request_from_argv(&args)? {
        return run_images_json(&request);
    }

    // Direct container inspect accepts a Docker-shaped JSON selector only as
    // an explicit opt-in to Apple's bare native JSON document. The existing
    // unformatted inspect translations remain on their inherited stdio path.
    if let Some(request) = docker_inspect_json_request_from_argv(&args)? {
        return run_inspect_json(&request);
    }

    // Image metadata has a separate strict route: Docker's direct `inspect`
    // remains container-only here, while `docker image inspect` can preserve
    // Apple's one native image document without claiming Docker polymorphism.
    if let Some(request) = docker_image_inspect_json_request_from_argv(&args)? {
        return run_image_inspect_json(&request);
    }

    // Apple Container logs are text, not a native JSON protocol. Its one
    // agent-shaped JSON form therefore has its own bounded wrapper and must
    // intercept the selector before generic text translation can spawn.
    if let Some(request) = docker_logs_json_request_from_argv(&args)? {
        return run_logs_json(&request);
    }

    // Direct exec is arbitrary foreground command output rather than native
    // Apple JSON. Its explicit agent selector must never fall through to the
    // raw inherited-stdio path after partial validation.
    if let Some(request) = docker_exec_json_request_from_argv(&args)? {
        return run_exec_json(&request);
    }

    // Direct run is a foreground, VAT-owned one-shot rather than a generic
    // Docker lifecycle translation. Its JSON selector must be intercepted
    // before a caller-supplied name, port, or detach option can reach Apple.
    if let Some(request) = docker_run_json_request_from_argv(&args)? {
        return run_run_json(&request);
    }

    let translated = translate(&args)?;
    let program = translated
        .first()
        .context("Docker shim generated an empty Apple Container command")?;
    let status = Command::new(program)
        .args(&translated[1..])
        .status()
        .with_context(|| format!("run Apple Container command `{}`", translated.join(" ")))?;
    Ok(exit_code(status))
}

fn shim_meta_command(args: &[OsString]) -> Result<Option<ExitCode>> {
    let args = utf8_args(args)?;
    if args.is_empty() || (args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h" | "help"))
    {
        println!(
            "vat Docker-command shim (Apple Container backend; not a Docker Engine)\n\
Supported commands: {SUPPORTED_COMMANDS}\n\
Install with: vat docker install-shim --dir <directory-on-PATH>\n\
Unsupported Docker flags and Engine/API commands fail before a runtime process starts.\n\
No Docker Engine socket/API, general Compose, SDK, Testcontainers, or devcontainer compatibility is provided.\n\
{STRICT_STATS_HELP}\n\
{STRICT_SYSTEM_DF_HELP}\n\
{STRICT_PS_HELP}\n\
{STRICT_IMAGES_HELP}\n\
{STRICT_INSPECT_HELP}\n\
{STRICT_LOGS_HELP}\n\
{STRICT_EXEC_HELP}\n\
{STRICT_RUN_HELP}\n\
{STRICT_BUILD_HELP}\n\
{STRICT_PULL_HELP}\n\
{STRICT_IMAGE_INSPECT_HELP}\n\
{STRICT_COMPOSE_HELP}"
        );
        return Ok(Some(ExitCode::SUCCESS));
    }
    if args.len() == 1 && matches!(args[0].as_str(), "--version" | "-v") {
        println!(
            "vat Docker-command shim {} (Apple Container backend; not a Docker Engine)",
            crate::VERSION
        );
        return Ok(Some(ExitCode::SUCCESS));
    }
    Ok(None)
}

/// Every bounded Apple Container observation must have a wall-clock boundary:
/// the CLI can leave a read-only child or its pipe-owning descendants running
/// after the direct root exits. Keep this private and fixed so the opt-in shim
/// never becomes a process supervisor or a general streaming API.
const DOCKER_BOUNDED_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(5);
const DOCKER_BOUNDED_OBSERVATION_POLL_INTERVAL: Duration = Duration::from_millis(10);
const DOCKER_BOUNDED_OBSERVATION_STOP_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_DOCKER_NATIVE_JSON_CAPTURE_BYTES: usize = 256 * 1024;
const MAX_DOCKER_LOGS_JSON_TAIL_LINES: usize = 1000;
const MAX_DOCKER_BOUNDED_TEXT_STREAM_CAPTURE_BYTES: usize = 64 * 1024;
const MAX_DOCKER_BOUNDED_TEXT_JSON_STRING_BYTES: usize = 64 * 1024;
const MAX_DOCKER_EXEC_JSON_TIMEOUT_SECONDS: u64 = 1200;
const MAX_DOCKER_RUN_JSON_TIMEOUT_SECONDS: u64 = 1200;
const MAX_DOCKER_BUILD_JSON_TIMEOUT_SECONDS: u64 = 1200;
const MAX_DOCKER_PULL_JSON_TIMEOUT_SECONDS: u64 = 1200;
const DOCKER_RUN_OWNER_LABEL: &str = "io.cclab.vat.docker-run-owner";
const DOCKER_RUN_OWNER_TOKEN_BYTES: usize = 16;

/// The only supported Docker-shaped stats surface. It is a one-shot,
/// read-only Apple Container observation, not Docker Engine stats schema
/// compatibility or a liveness/ownership proof.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerStatsRequest {
    container_ids: Vec<String>,
}

/// The strict global disk report has no filters, positional selectors, or
/// action flags. Its one Docker-shaped JSON selector is validation-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DockerSystemDfJsonRequest;

/// The strict `ps --format json` request has no object filters or
/// positional arguments. `all` is deliberately the sole inventory modifier
/// that maps to a documented Apple Container list flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DockerPsJsonRequest {
    all: bool,
}

/// The strict image inventory intentionally has no modifiers, selectors, or
/// Docker filter semantics. Its only accepted Docker-shaped flag is one exact
/// JSON selector, normalized to Apple's public image-list command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DockerImagesJsonRequest;

/// A strict direct-container inspect request. Docker's generic inspect
/// polymorphism is intentionally not supported here: this one identifier is
/// always sent to Apple's public `container inspect` command.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerInspectJsonRequest {
    container_id: String,
}

/// A strict direct-image metadata request. Docker's polymorphic `inspect`
/// behavior remains out of scope: this one reference is always sent to
/// Apple's public `container image inspect` command.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerImageInspectJsonRequest {
    image_reference: String,
}

/// A direct Apple Container log observation is explicitly a finite suffix
/// snapshot. The Docker-shaped JSON selector never reaches the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerLogsJsonRequest {
    container_id: String,
    tail_lines: usize,
}

/// One non-interactive direct-container command snapshot. The command is kept
/// as argv rather than shell text. The Docker-facing literal separator is a
/// validated input boundary and is deliberately removed before Apple spawning.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerExecJsonRequest {
    container_id: String,
    timeout_seconds: u64,
    command: Vec<String>,
}

/// One explicit foreground image invocation. Caller-controlled lifecycle
/// options are intentionally absent: the JSON path owns the generated name
/// and ownership label so cleanup cannot target a caller-selected container.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerRunJsonRequest {
    image: String,
    timeout_seconds: u64,
    command: Vec<String>,
}

/// One explicit, finite host-client build observation. The context has been
/// canonicalized from one existing local directory before Apple Container is
/// started; build arguments and labels remain opaque backend argv values.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerBuildJsonRequest {
    tag: String,
    context: String,
    dockerfile: Option<String>,
    build_args: Vec<String>,
    target: Option<String>,
    platform: Option<String>,
    labels: Vec<String>,
    timeout_seconds: u64,
}

/// One explicit, finite host-client image-pull observation. The reference is
/// deliberately opaque after a narrow argv-boundary validation; VAT neither
/// owns the result nor manages registry/image lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerPullJsonRequest {
    image_reference: String,
    timeout_seconds: u64,
}

/// High-entropy identity used only for one strict Docker-shaped run. The
/// container name and label token are independently random so an inspect-time
/// label check cannot be reconstructed from the generated name alone.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerRunJsonOwnership {
    name: String,
    token: String,
}

#[derive(Clone, Copy)]
struct DockerBoundedObservationSurface {
    docker_command: &'static str,
    apple_operation: &'static str,
}

const DOCKER_STATS_NATIVE_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker stats",
    apple_operation: "stats",
};

const DOCKER_SYSTEM_DF_NATIVE_JSON: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker system df",
        apple_operation: "system df",
    };

const DOCKER_PS_NATIVE_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker ps",
    apple_operation: "list",
};

const DOCKER_IMAGES_NATIVE_JSON: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker images",
        apple_operation: "image list",
    };

const DOCKER_INSPECT_NATIVE_JSON: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker inspect",
        apple_operation: "inspect",
    };

const DOCKER_IMAGE_INSPECT_NATIVE_JSON: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker image inspect",
        apple_operation: "image inspect",
    };

const DOCKER_LOGS_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker logs",
    apple_operation: "logs",
};

const DOCKER_EXEC_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker exec",
    apple_operation: "exec client",
};

const DOCKER_RUN_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker run",
    apple_operation: "run client",
};

const DOCKER_BUILD_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker build",
    apple_operation: "build client",
};

const DOCKER_PULL_JSON: DockerBoundedObservationSurface = DockerBoundedObservationSurface {
    docker_command: "docker pull",
    apple_operation: "pull client",
};

const DOCKER_RUN_CLEANUP_INSPECT: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker run",
        apple_operation: "run cleanup inspect",
    };

const DOCKER_RUN_CLEANUP_DELETE: DockerBoundedObservationSurface =
    DockerBoundedObservationSurface {
        docker_command: "docker run",
        apple_operation: "run cleanup delete",
    };

#[derive(Debug)]
struct DockerNativeJsonCapturedStream {
    bytes: Vec<u8>,
    capped: bool,
}

#[derive(Debug)]
struct DockerNativeJsonObservation {
    status: ExitStatus,
    stdout: DockerNativeJsonCapturedStream,
    stderr: DockerNativeJsonCapturedStream,
}

/// Internal common result for bounded Apple Container captures. Native JSON
/// and textual log snapshots deliberately use different stream types and
/// post-capture policies even though they share the process-lifecycle guard.
#[derive(Debug)]
struct DockerBoundedObservation<Stream> {
    status: ExitStatus,
    stdout: Stream,
    stderr: Stream,
}

/// One bounded textual stream from an Apple Container command. This is
/// deliberately separate from native JSON capture because VAT owns the JSON
/// wrapper and must retain arbitrary textual command output as a safe suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerBoundedTextCapturedStream {
    text: String,
    truncated: bool,
    utf8_lossy: bool,
}

/// Run the narrow stats observation with captured stdout. The successful
/// public stdout stays exactly Apple Container's JSON document; VAT validates
/// it before replaying it rather than wrapping it in a misleading Docker or
/// VAT stats schema.
fn run_stats(args: &[OsString]) -> Result<ExitCode> {
    let request = parse_docker_stats_args(&utf8_args(args)?)?;
    let observation = observe_docker_stats(&request)?;
    replay_docker_native_json(DOCKER_STATS_NATIVE_JSON, &observation)
}

/// Run the strict global Apple Container disk observation. Unlike the older
/// stats surface, a child nonzero status is not evidence and therefore cannot
/// replay even a native-looking stdout document.
fn run_system_df_json(request: &DockerSystemDfJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_system_df_json(request)?;
    replay_docker_native_json_success_only(DOCKER_SYSTEM_DF_NATIVE_JSON, &observation)
}

/// Run the strict one-shot `ps --format json` inventory. Its output is
/// intentionally Apple's opaque native JSON rather than a VAT record or
/// Docker Engine list schema.
fn run_ps_json(request: &DockerPsJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_ps_json(request)?;
    replay_docker_native_json(DOCKER_PS_NATIVE_JSON, &observation)
}

/// Run the strict one-shot image inventory. As with `docker ps --format json`,
/// the public result is Apple's opaque native JSON rather than a Docker Engine
/// image list or a VAT wrapper.
fn run_images_json(request: &DockerImagesJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_images_json(request)?;
    replay_docker_native_json(DOCKER_IMAGES_NATIVE_JSON, &observation)
}

/// Run the strict one-shot direct-container inspect. The backend's bare
/// inspect output is retained as opaque Apple-native JSON; VAT does not
/// synthesize Docker's polymorphic inspect schema or redact native fields.
fn run_inspect_json(request: &DockerInspectJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_inspect_json(request)?;
    replay_docker_native_json(DOCKER_INSPECT_NATIVE_JSON, &observation)
}

/// Run the strict direct-image inspect. The backend's bare inspect output is
/// retained as opaque Apple-native JSON; VAT neither synthesizes Docker's
/// polymorphic inspect schema nor interprets image metadata.
fn run_image_inspect_json(request: &DockerImageInspectJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_image_inspect_json(request)?;
    replay_docker_native_json(DOCKER_IMAGE_INSPECT_NATIVE_JSON, &observation)
}

/// Emit one VAT-owned receipt after a bounded Apple Container build client
/// exits. A host timeout or capture/setup failure returns an error before any
/// receipt can be written; a normal child nonzero exit retains its receipt.
fn run_build_json(request: &DockerBuildJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_build_json(request)?;
    println!("{}", docker_build_json_result(request, &observation));
    Ok(exit_code(observation.status))
}

fn docker_build_json_result(
    request: &DockerBuildJsonRequest,
    observation: &DockerBoundedObservation<DockerBoundedTextCapturedStream>,
) -> serde_json::Value {
    let mut result = serde_json::json!({
        "schema": "vat.docker.build.v1",
        "format": "vat_json",
        "type": "vat_docker_build",
        "command": "build",
        "backend": "apple-container",
        "tag": request.tag,
        "context": request.context,
        "context_kind": "existing_local_directory",
        "dockerfile": request.dockerfile,
        "requested_timeout_seconds": request.timeout_seconds,
        "timeout_scope": "host-container-client-observation",
        "source": "apple-container-build",
        "runtime_invoked": true,
        "outcome": if observation.status.success() { "completed" } else { "failed" },
        "child_exit_code": observation.status.code(),
        "stdout": observation.stdout.text,
        "stdout_truncated": observation.stdout.truncated,
        "stdout_utf8_lossy": observation.stdout.utf8_lossy,
        "stderr": observation.stderr.text,
        "stderr_truncated": observation.stderr.truncated,
        "stderr_utf8_lossy": observation.stderr.utf8_lossy,
        "image_lifecycle": "retained_no_auto_cleanup",
        "partial_or_replaced_image_cleanup_attempted": false,
        "docker_engine_api_implemented": false,
        "provenance_verified": false,
        "ownership_verified": false,
        "readiness_verified": false,
        "security_verified": false,
        "secret_redaction_guaranteed": false,
        "cancellation_guaranteed": false,
        "rollback_guaranteed": false,
        "untrusted_build_arguments": true,
        "untrusted_build_labels": true,
        "untrusted_build_output": true,
    });
    if observation.status.success() {
        result["next"] = serde_json::Value::String(format!(
            "docker image inspect --format json {}",
            shell_quote_command_argument(&request.tag)
        ));
    } else {
        // A failed build may have retained a partial or replaced image under
        // TAG, so never point the agent at image inspect as though it proved
        // this build's output. The generic shim help is runnable and contains
        // no caller-provided build argv or potentially secret-bearing args.
        result["terminal"] = serde_json::Value::String("build_failed".to_string());
        result["next"] = serde_json::Value::String("docker --help".to_string());
    }
    result
}

/// Emit one VAT-owned receipt after a bounded Apple Container image-pull
/// client exits. Setup, capture, pipe, and deadline errors return before any
/// receipt can be written; a normal child nonzero exit remains observable.
fn run_pull_json(request: &DockerPullJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_pull_json(request)?;
    println!("{}", docker_pull_json_result(request, &observation));
    Ok(exit_code(observation.status))
}

fn docker_pull_json_result(
    request: &DockerPullJsonRequest,
    observation: &DockerBoundedObservation<DockerBoundedTextCapturedStream>,
) -> serde_json::Value {
    let mut result = serde_json::json!({
        "schema": "vat.docker.pull.v1",
        "format": "vat_json",
        "type": "vat_docker_pull",
        "command": "pull",
        "backend": "apple-container",
        "image": request.image_reference,
        "requested_timeout_seconds": request.timeout_seconds,
        "timeout_scope": "host-container-client-observation",
        "source": "apple-container-image-pull",
        "runtime_invoked": true,
        "outcome": if observation.status.success() { "completed" } else { "failed" },
        "child_exit_code": observation.status.code(),
        "stdout": observation.stdout.text,
        "stdout_truncated": observation.stdout.truncated,
        "stdout_utf8_lossy": observation.stdout.utf8_lossy,
        "stderr": observation.stderr.text,
        "stderr_truncated": observation.stderr.truncated,
        "stderr_utf8_lossy": observation.stderr.utf8_lossy,
        "image_lifecycle": "not_owned_no_auto_cleanup",
        "cleanup_attempted": false,
        "registry_management_implemented": false,
        "provenance_verified": false,
        "digest_verified": false,
        "platform_verified": false,
        "freshness_verified": false,
        "image_state_verified": false,
        "ownership_verified": false,
        "security_verified": false,
        "secret_redaction_guaranteed": false,
        "cancellation_guaranteed": false,
        "download_completion_guaranteed": false,
        "rollback_guaranteed": false,
        "untrusted_pull_output": true,
    });
    if observation.status.success() {
        result["next"] = serde_json::Value::String(format!(
            "docker image inspect --format json {}",
            shell_quote_command_argument(&request.image_reference)
        ));
    } else {
        // A nonzero client may still have changed a local image or remote
        // transfer state. Keep the runnable handoff fixed and argument-free
        // rather than implying that inspecting the caller ref proves anything.
        result["terminal"] = serde_json::Value::String("pull_failed".to_string());
        result["next"] = serde_json::Value::String("docker --help".to_string());
    }
    result
}

/// Emit one VAT-owned bounded document because Apple Container logs are
/// arbitrary textual stdio rather than a native JSON response. Both captured
/// child streams remain inside this document so agents never receive a raw
/// mixed stream that could be mistaken for Docker log multiplexing.
fn run_logs_json(request: &DockerLogsJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_logs_json(request)?;
    println!("{}", docker_logs_json_result(request, &observation));
    Ok(exit_code(observation.status))
}

fn docker_logs_json_result(
    request: &DockerLogsJsonRequest,
    observation: &DockerBoundedObservation<DockerBoundedTextCapturedStream>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "vat.docker.logs.v1",
        "format": "vat_json",
        "type": "vat_docker_logs",
        "command": "logs",
        "backend": "apple-container",
        "container": request.container_id,
        "requested_tail_lines": request.tail_lines,
        "source": "apple-container-stdio",
        "runtime_invoked": true,
        "outcome": if observation.status.success() { "observed" } else { "failed" },
        "child_exit_code": observation.status.code(),
        "apple_container_stdio": observation.stdout.text,
        "apple_container_stdio_truncated": observation.stdout.truncated,
        "apple_container_stdio_utf8_lossy": observation.stdout.utf8_lossy,
        "diagnostic_stderr": observation.stderr.text,
        "diagnostic_stderr_truncated": observation.stderr.truncated,
        "diagnostic_stderr_utf8_lossy": observation.stderr.utf8_lossy,
        "secret_redaction_guaranteed": false,
        "untrusted_log_content": true,
        "next": format!("docker inspect --format json {}", request.container_id),
    })
}

/// Emit one VAT-owned bounded document for an explicit foreground exec
/// snapshot. This intentionally captures both child pipes rather than
/// replaying arbitrary command output around the one agent-facing document.
fn run_exec_json(request: &DockerExecJsonRequest) -> Result<ExitCode> {
    let observation = observe_docker_exec_json(request)?;
    println!("{}", docker_exec_json_result(request, &observation));
    Ok(exit_code(observation.status))
}

fn docker_exec_json_result(
    request: &DockerExecJsonRequest,
    observation: &DockerBoundedObservation<DockerBoundedTextCapturedStream>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "vat.docker.exec.v1",
        "format": "vat_json",
        "type": "vat_docker_exec",
        "command": "exec",
        "backend": "apple-container",
        "container": request.container_id,
        "requested_timeout_seconds": request.timeout_seconds,
        "timeout_scope": "host-container-client-observation",
        "source": "apple-container-exec",
        "runtime_invoked": true,
        "outcome": if observation.status.success() { "completed" } else { "failed" },
        "child_exit_code": observation.status.code(),
        "stdout": observation.stdout.text,
        "stdout_truncated": observation.stdout.truncated,
        "stdout_utf8_lossy": observation.stdout.utf8_lossy,
        "stderr": observation.stderr.text,
        "stderr_truncated": observation.stderr.truncated,
        "stderr_utf8_lossy": observation.stderr.utf8_lossy,
        "secret_redaction_guaranteed": false,
        "untrusted_command_output": true,
        "next": format!("docker inspect --format json {}", request.container_id),
    })
}

/// Run one generated, owner-labeled foreground container. Normal, nonzero,
/// and observed-timeout paths attempt cleanup before exposing a result;
/// process crash/recovery is deliberately outside this one-shot contract.
fn run_run_json(request: &DockerRunJsonRequest) -> Result<ExitCode> {
    let ownership = fresh_docker_run_json_ownership()?;
    let observation = observe_docker_run_json(request, &ownership);
    let cleanup = cleanup_docker_run_json_ownership(&ownership);

    match (observation, cleanup) {
        (Ok(observation), Ok(())) => {
            println!(
                "{}",
                docker_run_json_result(request, &ownership, &observation)
            );
            Ok(exit_code(observation.status))
        }
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error).context(
            "VAT's docker run JSON one-shot did not emit a result because exact owner-checked cleanup was not confirmed",
        ),
        (Err(run_error), Ok(())) => Err(run_error),
        (Err(run_error), Err(cleanup_error)) => Err(cleanup_error).context(format!(
            "VAT's docker run JSON one-shot failed and exact owner-checked cleanup was not confirmed; original run failure: {run_error}"
        )),
    }
}

fn docker_run_json_result(
    request: &DockerRunJsonRequest,
    ownership: &DockerRunJsonOwnership,
    observation: &DockerBoundedObservation<DockerBoundedTextCapturedStream>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "vat.docker.run.v1",
        "format": "vat_json",
        "type": "vat_docker_run",
        "command": "run",
        "backend": "apple-container",
        "image": request.image,
        "generated_container_name": ownership.name,
        "requested_timeout_seconds": request.timeout_seconds,
        "timeout_scope": "host-container-client-observation",
        "source": "apple-container-run",
        "runtime_invoked": true,
        "outcome": if observation.status.success() { "completed" } else { "failed" },
        "child_exit_code": observation.status.code(),
        "stdout": observation.stdout.text,
        "stdout_truncated": observation.stdout.truncated,
        "stdout_utf8_lossy": observation.stdout.utf8_lossy,
        "stderr": observation.stderr.text,
        "stderr_truncated": observation.stderr.truncated,
        "stderr_utf8_lossy": observation.stderr.utf8_lossy,
        "cleanup": "confirmed_absent",
        "secret_redaction_guaranteed": false,
        "untrusted_command_output": true,
        "terminal": "cleaned_up",
    })
}

fn fresh_docker_run_json_ownership() -> Result<DockerRunJsonOwnership> {
    let mut bytes = [0_u8; DOCKER_RUN_OWNER_TOKEN_BYTES * 2];
    getrandom::fill(&mut bytes).map_err(|error| {
        anyhow::anyhow!("read OS CSPRNG for docker run JSON owner identity: {error}")
    })?;
    Ok(DockerRunJsonOwnership {
        name: format!(
            "vat-docker-run-{}",
            hex_encode(&bytes[..DOCKER_RUN_OWNER_TOKEN_BYTES])
        ),
        token: format!(
            "vat-run-{}",
            hex_encode(&bytes[DOCKER_RUN_OWNER_TOKEN_BYTES..])
        ),
    })
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

enum DockerRunCleanupInspection {
    Owned,
    ConfirmedAbsent,
}

/// Inspect the exact generated name. A successful inspect must prove the
/// label first; a failed inspect is accepted only for Apple's explicit
/// not-found diagnostic. Any other ambiguity deliberately leaves the
/// container alone and fails the agent-facing operation closed.
fn inspect_docker_run_json_ownership(
    ownership: &DockerRunJsonOwnership,
) -> Result<DockerRunCleanupInspection> {
    let mut command = Command::new("container");
    command.args(["inspect", &ownership.name]);
    let observation = capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_RUN_CLEANUP_INSPECT,
    )?;
    if !observation.status.success() {
        if docker_run_inspect_reports_not_found(&observation, &ownership.name) {
            return Ok(DockerRunCleanupInspection::ConfirmedAbsent);
        }
        bail!(
            "VAT's docker run JSON cleanup could not confirm that generated container `{}` is absent: Apple Container inspect did not return its explicit container-not-found diagnostic",
            ownership.name
        );
    }
    if observation.stdout.capped {
        bail!(
            "VAT's docker run JSON cleanup could not validate generated container `{}` ownership because Apple Container inspect output exceeded VAT's bounded capture limit",
            ownership.name
        );
    }
    let document = serde_json::from_slice::<serde_json::Value>(&observation.stdout.bytes)
        .with_context(|| {
            format!(
                "VAT's docker run JSON cleanup could not validate generated container `{}` ownership because Apple Container inspect did not return JSON",
                ownership.name
            )
        })?;
    if !docker_run_inspect_has_owner_label(&document, ownership) {
        bail!(
            "VAT's docker run JSON cleanup refused to delete generated container `{}` because its exact owner label did not verify",
            ownership.name
        );
    }
    Ok(DockerRunCleanupInspection::Owned)
}

fn docker_run_inspect_has_owner_label(
    document: &serde_json::Value,
    ownership: &DockerRunJsonOwnership,
) -> bool {
    let container = document
        .as_array()
        .and_then(|containers| containers.first())
        .unwrap_or(document);
    container
        .get("configuration")
        .and_then(|configuration| configuration.get("labels"))
        .and_then(|labels| labels.get(DOCKER_RUN_OWNER_LABEL))
        .and_then(serde_json::Value::as_str)
        == Some(ownership.token.as_str())
}

fn docker_run_inspect_reports_not_found(
    observation: &DockerNativeJsonObservation,
    name: &str,
) -> bool {
    if observation.stderr.capped {
        return false;
    }
    let diagnostic = String::from_utf8_lossy(&observation.stderr.bytes);
    let normalized = diagnostic.to_ascii_lowercase();
    normalized.contains(&format!(
        "error: container not found: {}",
        name.to_ascii_lowercase()
    ))
}

fn delete_docker_run_json_owned_container(ownership: &DockerRunJsonOwnership) -> Result<()> {
    let mut command = Command::new("container");
    command.args(["delete", "--force", &ownership.name]);
    let observation = capture_docker_bounded_text_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_RUN_CLEANUP_DELETE,
    )?;
    if !observation.status.success() {
        bail!(
            "VAT's docker run JSON cleanup could not delete exact owner-verified container `{}`",
            ownership.name
        );
    }
    Ok(())
}

fn cleanup_docker_run_json_ownership(ownership: &DockerRunJsonOwnership) -> Result<()> {
    match inspect_docker_run_json_ownership(ownership)? {
        DockerRunCleanupInspection::ConfirmedAbsent => Ok(()),
        DockerRunCleanupInspection::Owned => {
            // Apple Container has no atomic conditional delete. The immediate
            // inspect-time label check is the narrowest available authority;
            // any ambiguity leaks rather than authorizing a name-only delete.
            delete_docker_run_json_owned_container(ownership)?;
            match inspect_docker_run_json_ownership(ownership)? {
                DockerRunCleanupInspection::ConfirmedAbsent => Ok(()),
                DockerRunCleanupInspection::Owned => bail!(
                    "VAT's docker run JSON cleanup could not confirm absence after deleting exact owner-verified container `{}`",
                    ownership.name
                ),
            }
        }
    }
}

/// Preserve a complete Apple Container JSON document byte-for-byte only after
/// bounded capture and whole-document validation. A backend nonzero status is
/// retained when its native stdout was valid; malformed or capped stdout is
/// fail-closed and never reaches the agent-facing stream.
fn replay_docker_native_json(
    surface: DockerBoundedObservationSurface,
    observation: &DockerNativeJsonObservation,
) -> Result<ExitCode> {
    if observation.stdout.capped {
        return fail_docker_native_json_output(
            surface,
            observation,
            &format!(
                "Apple Container {} output exceeded VAT's bounded capture limit",
                surface.apple_operation
            ),
        );
    }
    if serde_json::from_slice::<serde_json::Value>(&observation.stdout.bytes).is_err() {
        return fail_docker_native_json_output(
            surface,
            observation,
            &format!(
                "Apple Container {} output was not one valid JSON document",
                surface.apple_operation
            ),
        );
    }

    // Preserve the native JSON bytes (including Apple Container's whitespace)
    // only after full validation, so a malformed child stream cannot be
    // mistaken for the strict agent surface.
    let mut stdout = std::io::stdout();
    stdout
        .write_all(&observation.stdout.bytes)
        .with_context(|| {
            format!(
                "write validated Apple Container {} JSON",
                surface.apple_operation
            )
        })?;
    stdout.flush().with_context(|| {
        format!(
            "flush validated Apple Container {} JSON",
            surface.apple_operation
        )
    })?;
    write_docker_native_json_stderr(surface, &observation.stderr)?;
    Ok(exit_code(observation.status))
}

/// A global system report is evidence only after the Apple client succeeds.
/// Unlike inventory/status surfaces that preserve a valid native document on
/// child failure, this stricter policy suppresses stdout before parsing or
/// replaying it so a stale/partial report cannot be misread as current disk
/// state. Bounded stderr and the child exit remain available for remediation.
fn replay_docker_native_json_success_only(
    surface: DockerBoundedObservationSurface,
    observation: &DockerNativeJsonObservation,
) -> Result<ExitCode> {
    if !observation.status.success() {
        write_docker_native_json_stderr(surface, &observation.stderr)?;
        eprintln!(
            "{}: Apple Container {} exited nonzero; raw stdout was suppressed",
            surface.docker_command, surface.apple_operation
        );
        return Ok(exit_code(observation.status));
    }
    replay_docker_native_json(surface, observation)
}

fn fail_docker_native_json_output(
    surface: DockerBoundedObservationSurface,
    observation: &DockerNativeJsonObservation,
    message: &str,
) -> Result<ExitCode> {
    write_docker_native_json_stderr(surface, &observation.stderr)?;
    eprintln!(
        "{}: {message}; raw stdout was suppressed",
        surface.docker_command
    );
    Ok(if observation.status.success() {
        ExitCode::FAILURE
    } else {
        exit_code(observation.status)
    })
}

fn write_docker_native_json_stderr(
    surface: DockerBoundedObservationSurface,
    stderr_capture: &DockerNativeJsonCapturedStream,
) -> Result<()> {
    let mut stderr = std::io::stderr();
    stderr
        .write_all(&stderr_capture.bytes)
        .with_context(|| format!("write Apple Container {} stderr", surface.apple_operation))?;
    if stderr_capture.capped {
        stderr
            .write_all(
                format!(
                    "\nvat {}: Apple Container {} stderr exceeded VAT's bounded capture limit; remaining stderr was suppressed\n",
                    surface.docker_command, surface.apple_operation
                )
                .as_bytes(),
            )
            .with_context(|| {
                format!(
                    "write Apple Container {} stderr truncation marker",
                    surface.apple_operation
                )
            })?;
    }
    stderr
        .flush()
        .with_context(|| format!("flush Apple Container {} stderr", surface.apple_operation))
}

fn parse_docker_stats_args(args: &[String]) -> Result<DockerStatsRequest> {
    let mut no_stream = false;
    let mut json_format = false;
    let mut container_ids = Vec::new();
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if !container_ids.is_empty() {
            if is_option(argument) {
                bail!(
                    "VAT's docker stats accepts --no-stream and --format json only before explicit container ids"
                );
            }
            validate_docker_stats_container_id(argument)?;
            container_ids.push(argument.clone());
            index += 1;
            continue;
        }

        match argument.as_str() {
            "--no-stream" => {
                if no_stream {
                    bail!("VAT's docker stats accepts --no-stream exactly once");
                }
                no_stream = true;
                index += 1;
            }
            "--format" => {
                if json_format {
                    bail!("VAT's docker stats accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker stats requires `--format json` before explicit container ids",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker stats accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker stats accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker stats format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker stats accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker stats option `{argument}`; VAT accepts only exact --no-stream and --format json before explicit container ids"
                );
            }
            _ => {
                validate_docker_stats_container_id(argument)?;
                container_ids.push(argument.clone());
                index += 1;
            }
        }
    }

    if !no_stream || !json_format || container_ids.is_empty() {
        bail!(
            "VAT's docker stats accepts only `docker stats --no-stream --format json CONTAINER [CONTAINER...]` (or --format=json)"
        );
    }
    Ok(DockerStatsRequest { container_ids })
}

fn validate_docker_stats_container_id(container_id: &str) -> Result<()> {
    if container_id.is_empty()
        || container_id.starts_with('-')
        || container_id
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!(
            "VAT's docker stats requires explicit non-empty container ids without whitespace or a leading '-'"
        )
    }
    Ok(())
}

/// Identify only top-level `docker system df` once a format selector appears.
/// The raw command has no compatibility mapping; a selector-bearing malformed
/// form belongs to this parser so it fails before any Apple process starts.
fn docker_system_df_json_request_from_argv(
    args: &[OsString],
) -> Result<Option<DockerSystemDfJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    if verb != "system" {
        return Ok(None);
    }
    let Some((subcommand, system_args)) = rest.split_first() else {
        return Ok(None);
    };
    if subcommand != "df" {
        return Ok(None);
    }
    if !system_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_system_df_json_args(system_args).map(Some)
}

/// Parse the one selector-only global disk grammar. No filters, verbosity,
/// object selectors, or action flags can reach the public Apple report.
fn parse_docker_system_df_json_args(args: &[String]) -> Result<DockerSystemDfJsonRequest> {
    let mut json_format = false;
    let mut index = 0;
    while let Some(argument) = args.get(index) {
        match argument.as_str() {
            "--" => bail!("VAT's docker system df JSON form does not accept `--`"),
            "--format" => {
                if json_format {
                    bail!("VAT's docker system df JSON form accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker system df JSON form requires `--format json` and no other arguments",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker system df JSON form accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker system df JSON form accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker system df format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker system df JSON form accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            _ if is_option(argument) => bail!(
                "unsupported docker system df JSON option `{argument}`; VAT accepts only exact --format json"
            ),
            _ => bail!(
                "VAT's docker system df JSON form accepts no positional arguments; use only `docker system df --format json`"
            ),
        }
    }
    if !json_format {
        bail!(
            "VAT's docker system df JSON form accepts only `docker system df --format json` (or --format=json)"
        );
    }
    Ok(DockerSystemDfJsonRequest)
}

/// Identify the direct and documented Docker-shaped aliases that need native
/// JSON capture. Forms without a format selector deliberately fall through to
/// the older text `translate_ps` path, preserving its existing human/quiet
/// semantics. Once a format option appears, this strict parser owns the full
/// request so templates and options-after-positionals fail before spawn.
fn docker_ps_json_request_from_argv(args: &[OsString]) -> Result<Option<DockerPsJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    let ps_args = if verb == "ps" {
        rest
    } else if verb == "container" {
        let Some((subcommand, rest)) = rest.split_first() else {
            return Ok(None);
        };
        match subcommand.as_str() {
            // Keep the JSON aliases deliberately smaller than the preexisting
            // text group: the public inventory aliases are `ls` and `list`.
            "ls" | "list" => rest,
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    };

    if !ps_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_ps_json_args(ps_args).map(Some)
}

/// Parse only the documented agent inventory spellings. There are no accepted
/// positional arguments, so any non-option is rejected rather than being
/// silently forwarded as a Docker filter or selector.
fn parse_docker_ps_json_args(args: &[String]) -> Result<DockerPsJsonRequest> {
    let mut all = false;
    let mut json_format = false;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        match argument.as_str() {
            "--all" | "-a" => {
                if all {
                    bail!("VAT's docker ps JSON inventory accepts --all or -a exactly once");
                }
                all = true;
                index += 1;
            }
            "--format" => {
                if json_format {
                    bail!("VAT's docker ps JSON inventory accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker ps JSON inventory requires `--format json` before any positional arguments",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker ps JSON inventory accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker ps JSON inventory accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker ps JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker ps JSON inventory accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--quiet" | "-q" => {
                bail!(
                    "VAT's docker ps JSON inventory does not combine --quiet with Apple-native JSON"
                );
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker ps JSON inventory option `{argument}`; VAT accepts only exact --format json and optional --all/-a before no positional arguments"
                );
            }
            _ => {
                bail!(
                    "VAT's docker ps JSON inventory does not accept positional arguments; options must precede no positional arguments"
                );
            }
        }
    }

    if !json_format {
        bail!(
            "VAT's docker ps JSON inventory accepts only `docker ps --format json` (or --format=json), optionally with --all or -a"
        );
    }
    Ok(DockerPsJsonRequest { all })
}

/// Identify only the direct `images` spelling and Docker's two image-group
/// aliases. A format selector deliberately transfers ownership to the strict
/// parser below, so templates, positionals, and unsupported image flags fail
/// before Apple Container is spawned. Without a format selector, retain the
/// pre-existing text/quiet translation behavior.
fn docker_images_json_request_from_argv(
    args: &[OsString],
) -> Result<Option<DockerImagesJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    let image_args = if verb == "images" {
        rest
    } else if verb == "image" {
        let Some((subcommand, rest)) = rest.split_first() else {
            return Ok(None);
        };
        match subcommand.as_str() {
            "ls" | "list" => rest,
            _ => return Ok(None),
        }
    } else {
        return Ok(None);
    };

    if !image_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_images_json_args(image_args).map(Some)
}

/// Parse the intentionally exact agent image inventory form. Docker image
/// filters/templates have no public Apple Container equivalence that VAT can
/// safely claim, so any flag other than its single JSON selector is rejected.
fn parse_docker_images_json_args(args: &[String]) -> Result<DockerImagesJsonRequest> {
    let mut json_format = false;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker images JSON inventory accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker images JSON inventory requires `--format json` with no image selectors",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker images JSON inventory accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker images JSON inventory accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker images format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker images JSON inventory accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker images JSON inventory option `{argument}`; VAT accepts only exact --format json with no image selectors"
                );
            }
            _ => {
                bail!(
                    "VAT's docker images JSON inventory does not accept positional image or repository selectors"
                );
            }
        }
    }

    if !json_format {
        bail!(
            "VAT's docker images JSON inventory accepts only `docker images --format json` (or --format=json)"
        );
    }
    Ok(DockerImagesJsonRequest)
}

/// Identify only direct Docker inspect and its explicit `docker container`
/// alias. Once a format selector appears, this strict parser owns the whole
/// argv so Docker's object-type selector, templates, and multiple-object
/// behavior cannot silently reach Apple's container-only inspect command.
fn docker_inspect_json_request_from_argv(
    args: &[OsString],
) -> Result<Option<DockerInspectJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    let inspect_args = if verb == "inspect" {
        rest
    } else if verb == "container" {
        let Some((subcommand, rest)) = rest.split_first() else {
            return Ok(None);
        };
        if subcommand == "inspect" {
            rest
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    if !inspect_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_inspect_json_args(inspect_args).map(Some)
}

/// Parse exactly one explicit container id/name after one JSON selector.
/// Apple's inspect command emits native JSON without an output-format flag;
/// the Docker-shaped selector is therefore validation-only and never passed
/// to the backend.
fn parse_docker_inspect_json_args(args: &[String]) -> Result<DockerInspectJsonRequest> {
    let mut json_format = false;
    let mut container_id = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if container_id.is_some() {
            if is_option(argument) {
                bail!(
                    "VAT's docker inspect JSON form accepts its one --format json selector before exactly one container id"
                );
            }
            bail!("VAT's docker inspect JSON form accepts exactly one explicit container id");
        }

        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker inspect JSON form accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker inspect JSON form requires `--format json` before one container id",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker inspect JSON form accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker inspect JSON form accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker inspect format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker inspect JSON form accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker inspect JSON option `{argument}`; VAT accepts only exact --format json before one container id"
                );
            }
            _ => {
                if !json_format {
                    bail!(
                        "VAT's docker inspect JSON form requires its --format json selector before one container id"
                    );
                }
                validate_docker_inspect_container_id(argument)?;
                container_id = Some(argument.clone());
                index += 1;
            }
        }
    }

    if !json_format || container_id.is_none() {
        bail!(
            "VAT's docker inspect JSON form accepts only `docker inspect --format json CONTAINER` (or --format=json)"
        );
    }
    Ok(DockerInspectJsonRequest {
        container_id: container_id.expect("checked above"),
    })
}

fn validate_docker_inspect_container_id(container_id: &str) -> Result<()> {
    let mut characters = container_id.bytes();
    let Some(first) = characters.next() else {
        bail!("VAT's docker inspect JSON form requires one non-empty container id");
    };
    if !first.is_ascii_alphanumeric()
        || !characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, b'.' | b'_' | b'-')
        })
    {
        bail!(
            "VAT's docker inspect JSON form requires one safe container id/name matching [A-Za-z0-9][A-Za-z0-9_.-]*"
        );
    }
    Ok(())
}

/// Identify only direct `docker image inspect`. Once a format selector
/// appears, this strict parser owns the whole argv so templates, multiple
/// references, and Docker image-inspect option behavior cannot reach Apple's
/// one-reference native inspect command.
fn docker_image_inspect_json_request_from_argv(
    args: &[OsString],
) -> Result<Option<DockerImageInspectJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    if verb != "image" {
        return Ok(None);
    }
    let Some((subcommand, inspect_args)) = rest.split_first() else {
        return Ok(None);
    };
    if subcommand != "inspect" {
        return Ok(None);
    }

    if !inspect_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_image_inspect_json_args(inspect_args).map(Some)
}

/// Parse exactly one explicit image reference after one JSON selector.
/// Apple's image inspect command emits native JSON without an output-format
/// flag; the Docker-shaped selector is validation-only and never reaches the
/// backend.
fn parse_docker_image_inspect_json_args(args: &[String]) -> Result<DockerImageInspectJsonRequest> {
    let mut json_format = false;
    let mut image_reference = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if image_reference.is_some() {
            if argument == "--" {
                bail!("VAT's docker image inspect JSON form does not accept `--`");
            }
            if is_option(argument) {
                bail!(
                    "VAT's docker image inspect JSON form accepts its one --format json selector before exactly one image reference"
                );
            }
            bail!(
                "VAT's docker image inspect JSON form accepts exactly one explicit image reference"
            );
        }

        match argument.as_str() {
            "--" => bail!("VAT's docker image inspect JSON form does not accept `--`"),
            "--format" => {
                if json_format {
                    bail!(
                        "VAT's docker image inspect JSON form accepts --format json exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker image inspect JSON form requires `--format json` before one image reference",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker image inspect JSON form accepts only `--format json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!(
                        "VAT's docker image inspect JSON form accepts --format=json exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker image inspect format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker image inspect JSON form accepts only `--format=json`; it preserves Apple Container JSON and does not implement Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker image inspect JSON option `{argument}`; VAT accepts only exact --format json before one image reference"
                );
            }
            _ => {
                if !json_format {
                    bail!(
                        "VAT's docker image inspect JSON form requires its --format json selector before one image reference"
                    );
                }
                validate_docker_image_inspect_reference(argument)?;
                image_reference = Some(argument.clone());
                index += 1;
            }
        }
    }

    if !json_format || image_reference.is_none() {
        bail!(
            "VAT's docker image inspect JSON form accepts only `docker image inspect --format json IMAGE` (or --format=json)"
        );
    }
    Ok(DockerImageInspectJsonRequest {
        image_reference: image_reference.expect("checked above"),
    })
}

/// Image names are kept opaque rather than reimplementing Docker's full image
/// grammar. The narrow parser still rejects a malformed argv boundary before
/// Apple Container starts: a reference may not be empty, option-shaped, or
/// contain whitespace/control characters.
fn validate_docker_image_inspect_reference(image_reference: &str) -> Result<()> {
    if image_reference.is_empty()
        || image_reference.starts_with('-')
        || image_reference
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        bail!(
            "VAT's docker image inspect JSON form requires one non-empty image reference without a leading `-`, whitespace, or control characters"
        );
    }
    Ok(())
}

/// Identify only direct `logs` and its documented `docker container logs`
/// alias. A format selector transfers the entire argv to the strict snapshot
/// parser so it can never fall through to a streaming or unconstrained text
/// invocation after partial validation.
fn docker_logs_json_request_from_argv(args: &[OsString]) -> Result<Option<DockerLogsJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    let logs_args = if verb == "logs" {
        rest
    } else if verb == "container" {
        let Some((subcommand, rest)) = rest.split_first() else {
            return Ok(None);
        };
        if subcommand == "logs" {
            rest
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    if !logs_args
        .iter()
        .any(|argument| argument == "--format" || argument.starts_with("--format="))
    {
        return Ok(None);
    }
    parse_docker_logs_json_args(logs_args).map(Some)
}

/// Parse one finite agent snapshot and reject every Docker log selector whose
/// behavior Apple Container does not document as a bounded single-container
/// observation. The container id is final so an option can never become a
/// backend argument after observation has started.
fn parse_docker_logs_json_args(args: &[String]) -> Result<DockerLogsJsonRequest> {
    let mut json_format = false;
    let mut tail_lines = None;
    let mut container_id = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if container_id.is_some() {
            if is_option(argument) {
                bail!(
                    "VAT's docker logs JSON snapshot accepts --format json and --tail LINES only before one final container id"
                );
            }
            bail!("VAT's docker logs JSON snapshot accepts exactly one final container id");
        }

        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker logs JSON snapshot accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker logs JSON snapshot requires `--format json` before --tail LINES and one container id",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker logs JSON snapshot accepts only `--format json`; it emits VAT's vat.docker.logs.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker logs JSON snapshot accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker logs JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker logs JSON snapshot accepts only `--format=json`; it emits VAT's vat.docker.logs.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--tail" => {
                if tail_lines.is_some() {
                    bail!("VAT's docker logs JSON snapshot accepts --tail exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker logs JSON snapshot requires --tail LINES before one container id",
                )?;
                tail_lines = Some(parse_docker_logs_json_tail(value)?);
                index += 2;
            }
            _ if argument.starts_with("--tail=") => {
                if tail_lines.is_some() {
                    bail!("VAT's docker logs JSON snapshot accepts --tail exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker logs JSON tail option")?;
                tail_lines = Some(parse_docker_logs_json_tail(value)?);
                index += 1;
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker logs JSON snapshot option `{argument}`; VAT accepts only one --format json and one bounded --tail LINES before one container id"
                );
            }
            _ => {
                if !json_format || tail_lines.is_none() {
                    bail!(
                        "VAT's docker logs JSON snapshot requires `docker logs --format json --tail LINES CONTAINER`"
                    );
                }
                validate_docker_logs_json_container_id(argument)?;
                container_id = Some(argument.clone());
                index += 1;
            }
        }
    }

    let Some(container_id) = container_id else {
        bail!(
            "VAT's docker logs JSON snapshot accepts only `docker logs --format json --tail LINES CONTAINER` (or --format=json/--tail=LINES)"
        );
    };
    let Some(tail_lines) = tail_lines else {
        bail!(
            "VAT's docker logs JSON snapshot accepts only `docker logs --format json --tail LINES CONTAINER` (or --format=json/--tail=LINES)"
        );
    };
    if !json_format {
        bail!(
            "VAT's docker logs JSON snapshot accepts only `docker logs --format json --tail LINES CONTAINER` (or --format=json/--tail=LINES)"
        );
    }
    Ok(DockerLogsJsonRequest {
        container_id,
        tail_lines,
    })
}

fn parse_docker_logs_json_tail(value: &str) -> Result<usize> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!(
            "VAT's docker logs JSON snapshot --tail requires a positive whole decimal line count"
        );
    }
    let tail_lines = value.parse::<usize>().with_context(|| {
        "VAT's docker logs JSON snapshot --tail requires a positive whole line count"
    })?;
    if !(1..=MAX_DOCKER_LOGS_JSON_TAIL_LINES).contains(&tail_lines) {
        bail!(
            "VAT's docker logs JSON snapshot --tail must be between 1 and {MAX_DOCKER_LOGS_JSON_TAIL_LINES} lines"
        );
    }
    Ok(tail_lines)
}

fn validate_docker_logs_json_container_id(container_id: &str) -> Result<()> {
    let mut characters = container_id.bytes();
    let Some(first) = characters.next() else {
        bail!("VAT's docker logs JSON snapshot requires one non-empty container id");
    };
    if !first.is_ascii_alphanumeric()
        || !characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, b'.' | b'_' | b'-')
        })
    {
        bail!(
            "VAT's docker logs JSON snapshot requires one safe container id/name matching [A-Za-z0-9][A-Za-z0-9_.-]*"
        );
    }
    Ok(())
}

/// Identify only direct exec and its documented docker container exec alias.
/// The selector scan intentionally stops at the first positional argument (the
/// raw exec container) or literal separator, so a raw command such as
/// docker exec CONTAINER -- command --format=json is never intercepted.
fn docker_exec_json_request_from_argv(args: &[OsString]) -> Result<Option<DockerExecJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, rest)) = args.split_first() else {
        return Ok(None);
    };
    let exec_args = if verb == "exec" {
        rest
    } else if verb == "container" {
        let Some((subcommand, rest)) = rest.split_first() else {
            return Ok(None);
        };
        if subcommand == "exec" {
            rest
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    if !docker_exec_json_selector_before_container(exec_args) {
        return Ok(None);
    }
    parse_docker_exec_json_args(exec_args).map(Some)
}

fn docker_exec_json_selector_before_container(args: &[String]) -> bool {
    let mut index = 0;
    while let Some(argument) = args.get(index) {
        if argument == "--" || !is_option(argument) {
            break;
        }
        if argument == "--format" || argument.starts_with("--format=") {
            return true;
        }
        // Value-taking raw exec options are not the container. Skip their
        // separate values so a later format selector before the real
        // container remains strict and cannot silently use raw execution.
        if docker_exec_option_takes_value(argument) {
            if args
                .get(index + 1)
                .is_some_and(|value| value == "--format" || value.starts_with("--format="))
            {
                return true;
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    false
}

fn docker_exec_option_takes_value(argument: &str) -> bool {
    matches!(
        argument,
        "--timeout"
            | "-e"
            | "--env"
            | "--env-file"
            | "-u"
            | "--user"
            | "-w"
            | "--workdir"
            | "--ulimit"
    )
}

/// Parse exactly one foreground host-client observation. Both selectors must
/// precede one safe container id, which in turn must be followed by the
/// literal separator and at least one command argv element.
fn parse_docker_exec_json_args(args: &[String]) -> Result<DockerExecJsonRequest> {
    let mut json_format = false;
    let mut timeout_seconds = None;
    let mut container_id = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if container_id.is_some() {
            if argument != "--" {
                bail!(
                    "VAT's docker exec JSON snapshot requires one literal -- separator immediately after its container id"
                );
            }
            let command = args[index + 1..].to_vec();
            if command.is_empty() {
                bail!(
                    "VAT's docker exec JSON snapshot requires at least one command argument after the literal -- separator"
                );
            }
            return Ok(DockerExecJsonRequest {
                container_id: container_id.expect("checked above"),
                timeout_seconds: timeout_seconds.expect("container follows timeout"),
                command,
            });
        }

        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker exec JSON snapshot accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker exec JSON snapshot requires --format json before --timeout SECONDS and one container id",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker exec JSON snapshot accepts only --format json; it emits VAT's vat.docker.exec.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker exec JSON snapshot accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker exec JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker exec JSON snapshot accepts only --format=json; it emits VAT's vat.docker.exec.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--timeout" => {
                if timeout_seconds.is_some() {
                    bail!("VAT's docker exec JSON snapshot accepts --timeout SECONDS exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker exec JSON snapshot requires --timeout SECONDS before one container id",
                )?;
                timeout_seconds = Some(parse_docker_exec_json_timeout(value)?);
                index += 2;
            }
            _ if argument.starts_with("--timeout=") => {
                if timeout_seconds.is_some() {
                    bail!("VAT's docker exec JSON snapshot accepts --timeout=SECONDS exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker exec JSON timeout option")?;
                timeout_seconds = Some(parse_docker_exec_json_timeout(value)?);
                index += 1;
            }
            "--" => {
                bail!(
                    "VAT's docker exec JSON snapshot requires --format json, --timeout SECONDS, and one container id before the literal -- separator"
                );
            }
            _ if is_option(argument) => {
                bail!(
                    "unsupported docker exec JSON snapshot option {argument:?}; VAT accepts only one --format json and one --timeout SECONDS before one container id"
                );
            }
            _ => {
                if !json_format || timeout_seconds.is_none() {
                    bail!(
                        "VAT's docker exec JSON snapshot requires --format json and --timeout SECONDS before one container id"
                    );
                }
                validate_docker_exec_json_container_id(argument)?;
                container_id = Some(argument.clone());
                index += 1;
            }
        }
    }

    bail!(
        "VAT's docker exec JSON snapshot accepts only docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...] (or --format=json/--timeout=SECONDS)"
    )
}

fn parse_docker_exec_json_timeout(value: &str) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("VAT's docker exec JSON snapshot --timeout requires positive whole decimal seconds");
    }
    let timeout_seconds = value.parse::<u64>().with_context(|| {
        "VAT's docker exec JSON snapshot --timeout requires positive whole decimal seconds"
    })?;
    if !(1..=MAX_DOCKER_EXEC_JSON_TIMEOUT_SECONDS).contains(&timeout_seconds) {
        bail!(
            "VAT's docker exec JSON snapshot --timeout must be between 1 and {MAX_DOCKER_EXEC_JSON_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(timeout_seconds)
}

fn validate_docker_exec_json_container_id(container_id: &str) -> Result<()> {
    let mut characters = container_id.bytes();
    let Some(first) = characters.next() else {
        bail!("VAT's docker exec JSON snapshot requires one non-empty container id");
    };
    if !first.is_ascii_alphanumeric()
        || !characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, b'.' | b'_' | b'-')
        })
    {
        bail!(
            "VAT's docker exec JSON snapshot requires one safe container id/name matching [A-Za-z0-9][A-Za-z0-9_.-]*"
        );
    }
    Ok(())
}

/// Identify only direct `docker run` requests whose JSON selector appears
/// before the image. A format-looking command argument after IMAGE stays on
/// the older raw translation path, so this strict parser never steals child
/// argv merely because it resembles a Docker option.
fn docker_run_json_request_from_argv(args: &[OsString]) -> Result<Option<DockerRunJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, run_args)) = args.split_first() else {
        return Ok(None);
    };
    if verb != "run" || !docker_run_json_selector_before_image(run_args) {
        return Ok(None);
    }
    parse_docker_run_json_args(run_args).map(Some)
}

fn docker_run_json_selector_before_image(args: &[String]) -> bool {
    let mut index = 0;
    while let Some(argument) = args.get(index) {
        if argument == "--" || !is_option(argument) {
            return false;
        }
        if argument == "--format" || argument.starts_with("--format=") {
            return true;
        }
        if argument == "--timeout" {
            index += 2;
            continue;
        }
        if argument.starts_with("--timeout=") {
            index += 1;
            continue;
        }
        if let Some((flag, _)) = inline_long_option(argument) {
            if is_process_value_option(ProcessKind::Run, flag)
                || is_process_boolean(ProcessKind::Run, flag)
            {
                index += 1;
                continue;
            }
        }
        if is_process_value_option(ProcessKind::Run, argument) {
            index += 2;
            continue;
        }
        index += 1;
    }
    false
}

/// Parse one foreground image invocation. Only the two JSON selectors may
/// appear before IMAGE. Every caller-provided run option is rejected before
/// the Apple CLI exists, preserving VAT's generated ownership boundary.
fn parse_docker_run_json_args(args: &[String]) -> Result<DockerRunJsonRequest> {
    let mut json_format = false;
    let mut timeout_seconds = None;
    let mut image = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if image.is_some() {
            if argument == "--" {
                bail!(
                    "VAT's docker run JSON one-shot does not accept a Docker literal -- separator"
                );
            }
            return Ok(DockerRunJsonRequest {
                image: image.expect("checked above"),
                timeout_seconds: timeout_seconds.expect("image follows timeout"),
                command: args[index..].to_vec(),
            });
        }

        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker run JSON one-shot accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker run JSON one-shot requires --format json before --timeout SECONDS and IMAGE",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker run JSON one-shot accepts only --format json; it emits VAT's vat.docker.run.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker run JSON one-shot accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker run JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker run JSON one-shot accepts only --format=json; it emits VAT's vat.docker.run.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--timeout" => {
                if timeout_seconds.is_some() {
                    bail!("VAT's docker run JSON one-shot accepts --timeout SECONDS exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker run JSON one-shot requires --timeout SECONDS before IMAGE",
                )?;
                timeout_seconds = Some(parse_docker_run_json_timeout(value)?);
                index += 2;
            }
            _ if argument.starts_with("--timeout=") => {
                if timeout_seconds.is_some() {
                    bail!("VAT's docker run JSON one-shot accepts --timeout=SECONDS exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker run JSON timeout option")?;
                timeout_seconds = Some(parse_docker_run_json_timeout(value)?);
                index += 1;
            }
            "--" => bail!(
                "VAT's docker run JSON one-shot does not accept a Docker literal -- separator; pass IMAGE followed directly by optional command argv"
            ),
            _ if is_option(argument) => bail!(
                "unsupported docker run JSON one-shot option {argument:?}; VAT accepts only one --format json and one --timeout SECONDS before IMAGE"
            ),
            _ => {
                if !json_format || timeout_seconds.is_none() {
                    bail!(
                        "VAT's docker run JSON one-shot requires --format json and --timeout SECONDS before IMAGE"
                    );
                }
                validate_docker_run_json_image(argument)?;
                image = Some(argument.clone());
                index += 1;
            }
        }
    }

    if let Some(image) = image {
        return Ok(DockerRunJsonRequest {
            image,
            timeout_seconds: timeout_seconds.expect("image follows timeout"),
            command: Vec::new(),
        });
    }
    bail!(
        "VAT's docker run JSON one-shot accepts only docker run --format json --timeout SECONDS IMAGE [COMMAND...] (or --format=json/--timeout=SECONDS)"
    )
}

fn parse_docker_run_json_timeout(value: &str) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("VAT's docker run JSON one-shot --timeout requires positive whole decimal seconds");
    }
    let timeout_seconds = value.parse::<u64>().with_context(|| {
        "VAT's docker run JSON one-shot --timeout requires positive whole decimal seconds"
    })?;
    if !(1..=MAX_DOCKER_RUN_JSON_TIMEOUT_SECONDS).contains(&timeout_seconds) {
        bail!(
            "VAT's docker run JSON one-shot --timeout must be between 1 and {MAX_DOCKER_RUN_JSON_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(timeout_seconds)
}

fn validate_docker_run_json_image(image: &str) -> Result<()> {
    if image.is_empty()
        || image.starts_with('-')
        || image
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!(
            "VAT's docker run JSON one-shot requires one non-empty IMAGE without whitespace, control characters, or a leading '-'"
        );
    }
    Ok(())
}

/// Identify only direct `docker pull` requests that explicitly opt into the
/// bounded receipt grammar. Any appearance of either selector transfers the
/// whole direct argv to the strict parser, including a selector after IMAGE,
/// so malformed input cannot fall through to the inherited raw translator.
fn docker_pull_json_request_from_argv(args: &[OsString]) -> Result<Option<DockerPullJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, pull_args)) = args.split_first() else {
        return Ok(None);
    };
    if verb != "pull"
        || !pull_args.iter().any(|argument| {
            matches!(argument.as_str(), "--format" | "--timeout")
                || argument.starts_with("--format=")
                || argument.starts_with("--timeout=")
        })
    {
        return Ok(None);
    }
    parse_docker_pull_json_args(pull_args).map(Some)
}

/// Parse the one finite pull-receipt grammar. JSON/deadline selectors are
/// validation-only and never reach Apple's public image-pull command.
fn parse_docker_pull_json_args(args: &[String]) -> Result<DockerPullJsonRequest> {
    let mut json_format = false;
    let mut timeout_seconds = None;
    let mut image_reference = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if image_reference.is_some() {
            if argument == "--" {
                bail!("VAT's docker pull JSON receipt does not accept `--`");
            }
            if is_option(argument) {
                bail!(
                    "VAT's docker pull JSON receipt requires its one --format json and one --timeout SECONDS selector before exactly one image reference"
                );
            }
            bail!("VAT's docker pull JSON receipt accepts exactly one image reference");
        }

        match argument.as_str() {
            "--" => bail!("VAT's docker pull JSON receipt does not accept `--`"),
            "--format" => {
                if json_format {
                    bail!("VAT's docker pull JSON receipt accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker pull JSON receipt requires --format json before one image reference",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker pull JSON receipt accepts only --format json; it emits VAT's vat.docker.pull.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker pull JSON receipt accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker pull JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker pull JSON receipt accepts only --format=json; it emits VAT's vat.docker.pull.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--timeout" => {
                if timeout_seconds.is_some() {
                    bail!(
                        "VAT's docker pull JSON receipt accepts --timeout SECONDS exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker pull JSON receipt requires --timeout SECONDS before one image reference",
                )?;
                timeout_seconds = Some(parse_docker_pull_json_timeout(value)?);
                index += 2;
            }
            _ if argument.starts_with("--timeout=") => {
                if timeout_seconds.is_some() {
                    bail!(
                        "VAT's docker pull JSON receipt accepts --timeout=SECONDS exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker pull JSON timeout option")?;
                timeout_seconds = Some(parse_docker_pull_json_timeout(value)?);
                index += 1;
            }
            _ if is_option(argument) => bail!(
                "unsupported docker pull JSON receipt option {argument:?}; VAT accepts only --format json and --timeout SECONDS before one image reference"
            ),
            _ => {
                if !json_format || timeout_seconds.is_none() {
                    bail!(
                        "VAT's docker pull JSON receipt requires --format json and --timeout SECONDS before one image reference"
                    );
                }
                validate_docker_pull_json_image_reference(argument)?;
                image_reference = Some(argument.clone());
                index += 1;
            }
        }
    }

    if !json_format || timeout_seconds.is_none() || image_reference.is_none() {
        bail!(
            "VAT's docker pull JSON receipt accepts only docker pull --format json --timeout SECONDS IMAGE (or --format=json/--timeout=SECONDS)"
        );
    }
    Ok(DockerPullJsonRequest {
        image_reference: image_reference.expect("checked above"),
        timeout_seconds: timeout_seconds.expect("checked above"),
    })
}

fn parse_docker_pull_json_timeout(value: &str) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("VAT's docker pull JSON receipt --timeout requires positive whole decimal seconds");
    }
    let timeout_seconds = value.parse::<u64>().with_context(|| {
        "VAT's docker pull JSON receipt --timeout requires positive whole decimal seconds"
    })?;
    if !(1..=MAX_DOCKER_PULL_JSON_TIMEOUT_SECONDS).contains(&timeout_seconds) {
        bail!(
            "VAT's docker pull JSON receipt --timeout must be between 1 and {MAX_DOCKER_PULL_JSON_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(timeout_seconds)
}

/// Keep the reference opaque rather than rebuilding Docker's registry parser,
/// but reject ambiguous argv plus URL-style remote syntax before echoing it in
/// a receipt handoff. OCI paths and digest references remain opaque here.
fn validate_docker_pull_json_image_reference(image_reference: &str) -> Result<()> {
    if image_reference.is_empty()
        || image_reference.starts_with('-')
        || image_reference
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
    {
        bail!(
            "VAT's docker pull JSON receipt requires one non-empty image reference without a leading `-`, whitespace, or control characters"
        );
    }
    if image_reference.contains("://") || image_reference.starts_with("git@") {
        bail!(
            "VAT's docker pull JSON receipt requires an opaque image reference, not a URL or userinfo-style remote"
        );
    }
    Ok(())
}

/// Identify only direct `docker build` requests that explicitly opt into the
/// bounded receipt grammar. Raw Docker-build translations without either
/// selector continue unchanged through the generic translator below.
fn docker_build_json_request_from_argv(
    args: &[OsString],
) -> Result<Option<DockerBuildJsonRequest>> {
    let args = utf8_args(args)?;
    let Some((verb, build_args)) = args.split_first() else {
        return Ok(None);
    };
    if verb != "build"
        || !build_args.iter().any(|argument| {
            matches!(argument.as_str(), "--format" | "--timeout")
                || argument.starts_with("--format=")
                || argument.starts_with("--timeout=")
        })
    {
        return Ok(None);
    }
    parse_docker_build_json_args(build_args).map(Some)
}

/// Parse the one finite build-receipt grammar. Every supported option must
/// precede exactly one local directory context; JSON/deadline selectors are
/// validation-only and never reach the Apple Container client.
fn parse_docker_build_json_args(args: &[String]) -> Result<DockerBuildJsonRequest> {
    let mut json_format = false;
    let mut timeout_seconds = None;
    let mut tag = None;
    let mut dockerfile = None;
    let mut build_args = Vec::new();
    let mut target = None;
    let mut platform = None;
    let mut labels = Vec::new();
    let mut context = None;
    let mut index = 0;

    while let Some(argument) = args.get(index) {
        if context.is_some() {
            if is_option(argument) {
                bail!(
                    "VAT's docker build JSON receipt requires every option before exactly one local directory CONTEXT"
                );
            }
            bail!("VAT's docker build JSON receipt accepts exactly one local directory CONTEXT");
        }

        match argument.as_str() {
            "--format" => {
                if json_format {
                    bail!("VAT's docker build JSON receipt accepts --format json exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --format json before CONTEXT",
                )?;
                if value != "json" {
                    bail!(
                        "VAT's docker build JSON receipt accepts only --format json; it emits VAT's vat.docker.build.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 2;
            }
            _ if argument.starts_with("--format=") => {
                if json_format {
                    bail!("VAT's docker build JSON receipt accepts --format=json exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON format option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker build JSON receipt accepts only --format=json; it emits VAT's vat.docker.build.v1 schema, not Docker templates"
                    );
                }
                json_format = true;
                index += 1;
            }
            "--timeout" => {
                if timeout_seconds.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --timeout SECONDS exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --timeout SECONDS before CONTEXT",
                )?;
                timeout_seconds = Some(parse_docker_build_json_timeout(value)?);
                index += 2;
            }
            _ if argument.starts_with("--timeout=") => {
                if timeout_seconds.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --timeout=SECONDS exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON timeout option")?;
                timeout_seconds = Some(parse_docker_build_json_timeout(value)?);
                index += 1;
            }
            "--tag" => {
                if tag.is_some() {
                    bail!("VAT's docker build JSON receipt accepts --tag TAG exactly once");
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --tag TAG before CONTEXT",
                )?;
                validate_docker_build_json_tag(value)?;
                tag = Some(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--tag=") => {
                if tag.is_some() {
                    bail!("VAT's docker build JSON receipt accepts --tag=TAG exactly once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON tag option")?;
                validate_docker_build_json_tag(value)?;
                tag = Some(value.to_string());
                index += 1;
            }
            "--file" => {
                if dockerfile.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --file DOCKERFILE exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --file DOCKERFILE before CONTEXT",
                )?;
                validate_docker_build_json_dockerfile(value)?;
                dockerfile = Some(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--file=") => {
                if dockerfile.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --file=DOCKERFILE exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON file option")?;
                validate_docker_build_json_dockerfile(value)?;
                dockerfile = Some(value.to_string());
                index += 1;
            }
            "--build-arg" => {
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --build-arg K=V before CONTEXT",
                )?;
                validate_docker_build_json_assignment(value, "--build-arg")?;
                build_args.push(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--build-arg=") => {
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON build-arg option")?;
                validate_docker_build_json_assignment(value, "--build-arg")?;
                build_args.push(value.to_string());
                index += 1;
            }
            "--target" => {
                if target.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --target STAGE exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --target STAGE before CONTEXT",
                )?;
                validate_docker_build_json_value(value, "--target")?;
                target = Some(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--target=") => {
                if target.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --target=STAGE exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON target option")?;
                validate_docker_build_json_value(value, "--target")?;
                target = Some(value.to_string());
                index += 1;
            }
            "--platform" => {
                if platform.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --platform PLATFORM exactly once"
                    );
                }
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --platform PLATFORM before CONTEXT",
                )?;
                validate_docker_build_json_value(value, "--platform")?;
                platform = Some(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--platform=") => {
                if platform.is_some() {
                    bail!(
                        "VAT's docker build JSON receipt accepts --platform=PLATFORM exactly once"
                    );
                }
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON platform option")?;
                validate_docker_build_json_value(value, "--platform")?;
                platform = Some(value.to_string());
                index += 1;
            }
            "--label" => {
                let value = args.get(index + 1).context(
                    "VAT's docker build JSON receipt requires --label K=V before CONTEXT",
                )?;
                validate_docker_build_json_assignment(value, "--label")?;
                labels.push(value.clone());
                index += 2;
            }
            _ if argument.starts_with("--label=") => {
                let (_, value) = inline_long_option(argument)
                    .context("parse inline docker build JSON label option")?;
                validate_docker_build_json_assignment(value, "--label")?;
                labels.push(value.to_string());
                index += 1;
            }
            "--" => bail!(
                "VAT's docker build JSON receipt does not accept a Docker literal -- separator or stdin context"
            ),
            _ if is_option(argument) => bail!(
                "unsupported docker build JSON receipt option {argument:?}; VAT accepts only --format, --timeout, --tag, --file, --build-arg, --target, --platform, and --label before CONTEXT"
            ),
            _ => {
                if !json_format || timeout_seconds.is_none() || tag.is_none() {
                    bail!(
                        "VAT's docker build JSON receipt requires --format json, --timeout SECONDS, and --tag TAG before one local directory CONTEXT"
                    );
                }
                context = Some(canonical_docker_build_json_context(argument)?);
                index += 1;
            }
        }
    }

    let Some(context) = context else {
        bail!(
            "VAT's docker build JSON receipt accepts only docker build --format json --timeout SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform PLATFORM] [--label K=V ...] CONTEXT"
        );
    };
    if !json_format || timeout_seconds.is_none() || tag.is_none() {
        bail!(
            "VAT's docker build JSON receipt requires --format json, --timeout SECONDS, and --tag TAG before one local directory CONTEXT"
        );
    }
    Ok(DockerBuildJsonRequest {
        tag: tag.expect("checked above"),
        context,
        dockerfile,
        build_args,
        target,
        platform,
        labels,
        timeout_seconds: timeout_seconds.expect("checked above"),
    })
}

fn parse_docker_build_json_timeout(value: &str) -> Result<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("VAT's docker build JSON receipt --timeout requires positive whole decimal seconds");
    }
    let timeout_seconds = value.parse::<u64>().with_context(|| {
        "VAT's docker build JSON receipt --timeout requires positive whole decimal seconds"
    })?;
    if !(1..=MAX_DOCKER_BUILD_JSON_TIMEOUT_SECONDS).contains(&timeout_seconds) {
        bail!(
            "VAT's docker build JSON receipt --timeout must be between 1 and {MAX_DOCKER_BUILD_JSON_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(timeout_seconds)
}

fn validate_docker_build_json_tag(tag: &str) -> Result<()> {
    if tag.is_empty()
        || tag.starts_with('-')
        || tag
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        bail!(
            "VAT's docker build JSON receipt requires one non-empty TAG without whitespace, control characters, or a leading '-'"
        );
    }
    Ok(())
}

fn validate_docker_build_json_dockerfile(dockerfile: &str) -> Result<()> {
    if dockerfile.is_empty()
        || dockerfile.starts_with('-')
        || dockerfile.chars().any(char::is_control)
    {
        bail!(
            "VAT's docker build JSON receipt requires --file DOCKERFILE to name a non-empty non-option path without control characters, not stdin"
        );
    }
    Ok(())
}

fn validate_docker_build_json_assignment(value: &str, flag: &str) -> Result<()> {
    let Some((key, value)) = value.split_once('=') else {
        bail!("VAT's docker build JSON receipt option `{flag}` requires opaque K=V");
    };
    if key.is_empty()
        || key.starts_with('-')
        || key.chars().any(char::is_control)
        || value.chars().any(char::is_control)
    {
        bail!(
            "VAT's docker build JSON receipt option `{flag}` requires opaque K=V with a non-option key and no control characters"
        );
    }
    Ok(())
}

fn validate_docker_build_json_value(value: &str, flag: &str) -> Result<()> {
    if value.is_empty() || value.starts_with('-') || value.chars().any(char::is_control) {
        bail!(
            "VAT's docker build JSON receipt option `{flag}` requires a non-empty non-option value without control characters"
        );
    }
    Ok(())
}

fn canonical_docker_build_json_context(context: &str) -> Result<String> {
    if context.starts_with('-') || context.chars().any(char::is_control) {
        bail!(
            "VAT's docker build JSON receipt requires one existing local directory non-option CONTEXT without control characters, not stdin"
        );
    }
    if context.contains("://") || context.starts_with("git@") {
        bail!(
            "VAT's docker build JSON receipt requires one existing local directory CONTEXT, not a remote URL"
        );
    }
    let path = Path::new(context);
    if !path.is_dir() {
        bail!("VAT's docker build JSON receipt requires one existing local directory CONTEXT");
    }
    let canonical = std::fs::canonicalize(path)
        .with_context(|| "canonicalize Docker build JSON local context")?;
    canonical
        .to_str()
        .map(ToOwned::to_owned)
        .context("Docker build JSON local context must be valid UTF-8 after canonicalization")
}

fn observe_docker_stats(request: &DockerStatsRequest) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_stats_argv(request);
    let program = translated
        .first()
        .context("Docker stats shim generated an empty Apple Container command")?;
    // This is intentionally Apple's native `stats` shape. The parser has
    // already rejected streaming and every Docker template/filter form.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_STATS_NATIVE_JSON,
    )
}

fn translate_stats(args: &[String]) -> Result<Vec<String>> {
    Ok(apple_container_stats_argv(&parse_docker_stats_args(args)?))
}

fn apple_container_stats_argv(request: &DockerStatsRequest) -> Vec<String> {
    let mut translated = command(&["container", "stats", "--format", "json", "--no-stream"]);
    translated.extend(request.container_ids.iter().cloned());
    translated
}

fn observe_docker_system_df_json(
    _request: &DockerSystemDfJsonRequest,
) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_system_df_json_argv();
    let program = translated
        .first()
        .context("Docker system df JSON shim generated an empty Apple Container command")?;
    // This is one global, read-only Apple Container report. The parser has
    // already rejected Docker's verbose/template/action surface; success-only
    // replay prevents a failed client from publishing stale-looking evidence.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_SYSTEM_DF_NATIVE_JSON,
    )
}

fn apple_container_system_df_json_argv() -> Vec<String> {
    command(&["container", "system", "df", "--format", "json"])
}

fn observe_docker_ps_json(request: &DockerPsJsonRequest) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_ps_json_argv(request);
    let program = translated
        .first()
        .context("Docker ps JSON shim generated an empty Apple Container command")?;
    // The parser rejects Docker filters, templates, selectors, and quiet
    // output before this native `container list` invocation exists.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_PS_NATIVE_JSON,
    )
}

fn apple_container_ps_json_argv(request: &DockerPsJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "list", "--format", "json"]);
    if request.all {
        translated.push("--all".to_string());
    }
    translated
}

fn observe_docker_images_json(
    request: &DockerImagesJsonRequest,
) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_images_json_argv(request);
    let program = translated
        .first()
        .context("Docker images JSON shim generated an empty Apple Container command")?;
    // The strict parser deliberately rejects image filters, templates, and
    // selectors before this read-only native image-list observation exists.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_IMAGES_NATIVE_JSON,
    )
}

fn apple_container_images_json_argv(_request: &DockerImagesJsonRequest) -> Vec<String> {
    command(&["container", "image", "list", "--format", "json"])
}

fn observe_docker_inspect_json(
    request: &DockerInspectJsonRequest,
) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_inspect_json_argv(request);
    let program = translated
        .first()
        .context("Docker inspect JSON shim generated an empty Apple Container command")?;
    // Apple Container inspect is already a native JSON document. The strict
    // parser ensures the Docker-shaped selector was exact and never forwards
    // it as though Apple implemented Docker's inspect-format protocol.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_INSPECT_NATIVE_JSON,
    )
}

fn apple_container_inspect_json_argv(request: &DockerInspectJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "inspect"]);
    translated.push(request.container_id.clone());
    translated
}

fn observe_docker_image_inspect_json(
    request: &DockerImageInspectJsonRequest,
) -> Result<DockerNativeJsonObservation> {
    let translated = apple_container_image_inspect_json_argv(request);
    let program = translated
        .first()
        .context("Docker image inspect JSON shim generated an empty Apple Container command")?;
    // Apple Container image inspect is already a native JSON document. The
    // strict parser ensures the Docker-shaped selector was exact and never
    // forwards it as though Apple implemented Docker's format protocol.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_native_json_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_IMAGE_INSPECT_NATIVE_JSON,
    )
}

fn apple_container_image_inspect_json_argv(request: &DockerImageInspectJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "image", "inspect"]);
    translated.push(request.image_reference.clone());
    translated
}

fn observe_docker_logs_json(
    request: &DockerLogsJsonRequest,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    let translated = apple_container_logs_json_argv(request);
    let program = translated
        .first()
        .context("Docker logs shim generated an empty Apple Container command")?;
    // Apple Container logs has no JSON flag or stream-multiplex contract. The
    // strict parser has already made this a finite one-container snapshot;
    // only the bounded VAT wrapper below handles its arbitrary text output.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_bounded_text_command(
        &mut command,
        DOCKER_BOUNDED_OBSERVATION_TIMEOUT,
        DOCKER_LOGS_JSON,
    )
}

fn apple_container_logs_json_argv(request: &DockerLogsJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "logs", "-n"]);
    translated.push(request.tail_lines.to_string());
    translated.push(request.container_id.clone());
    translated
}

fn observe_docker_exec_json(
    request: &DockerExecJsonRequest,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    let translated = apple_container_exec_json_argv(request);
    let program = translated
        .first()
        .context("Docker exec shim generated an empty Apple Container command")?;
    // This deadline bounds VAT's host-side Apple Container client observation
    // and its copied stdio pipes. It does not claim control over a guest
    // process that a remote/container runtime may leave behind.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_bounded_text_command(
        &mut command,
        Duration::from_secs(request.timeout_seconds),
        DOCKER_EXEC_JSON,
    )
}

fn apple_container_exec_json_argv(request: &DockerExecJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "exec"]);
    translated.push(request.container_id.clone());
    translated.extend(request.command.iter().cloned());
    translated
}

fn observe_docker_run_json(
    request: &DockerRunJsonRequest,
    ownership: &DockerRunJsonOwnership,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    let translated = apple_container_run_json_argv(request, ownership);
    let program = translated
        .first()
        .context("Docker run shim generated an empty Apple Container command")?;
    // The deadline bounds only VAT's local Apple Container client and the
    // client-owned capture pipes. It is not a claim that all guest processes
    // have been terminated; exact owner-checked container cleanup follows.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_bounded_text_command(
        &mut command,
        Duration::from_secs(request.timeout_seconds),
        DOCKER_RUN_JSON,
    )
}

fn observe_docker_build_json(
    request: &DockerBuildJsonRequest,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    let translated = apple_container_build_json_argv(request);
    let program = translated
        .first()
        .context("Docker build JSON shim generated an empty Apple Container command")?;
    // This deadline bounds VAT's host-side Apple Container build client and
    // its copied pipes. It does not claim to cancel builder work, roll back a
    // partial/replaced tag, or remove any image after the client stops.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_bounded_text_command(
        &mut command,
        Duration::from_secs(request.timeout_seconds),
        DOCKER_BUILD_JSON,
    )
}

fn observe_docker_pull_json(
    request: &DockerPullJsonRequest,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    let translated = apple_container_pull_json_argv(request);
    let program = translated
        .first()
        .context("Docker pull JSON shim generated an empty Apple Container command")?;
    // This deadline bounds only VAT's host-side Apple Container pull client
    // and copied pipes. It does not claim to cancel a registry transfer,
    // establish image state, or roll back any local/backend-side result.
    let mut command = Command::new(program);
    command.args(&translated[1..]);
    capture_docker_bounded_text_command(
        &mut command,
        Duration::from_secs(request.timeout_seconds),
        DOCKER_PULL_JSON,
    )
}

fn apple_container_pull_json_argv(request: &DockerPullJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "image", "pull"]);
    translated.push(request.image_reference.clone());
    translated
}

fn apple_container_build_json_argv(request: &DockerBuildJsonRequest) -> Vec<String> {
    let mut translated = command(&["container", "build", "--tag"]);
    translated.push(request.tag.clone());
    if let Some(dockerfile) = &request.dockerfile {
        translated.push("--file".to_string());
        translated.push(dockerfile.clone());
    }
    for build_arg in &request.build_args {
        translated.push("--build-arg".to_string());
        translated.push(build_arg.clone());
    }
    if let Some(target) = &request.target {
        translated.push("--target".to_string());
        translated.push(target.clone());
    }
    if let Some(platform) = &request.platform {
        translated.push("--platform".to_string());
        translated.push(platform.clone());
    }
    for label in &request.labels {
        translated.push("--label".to_string());
        translated.push(label.clone());
    }
    translated.push(request.context.clone());
    translated
}

fn apple_container_run_json_argv(
    request: &DockerRunJsonRequest,
    ownership: &DockerRunJsonOwnership,
) -> Vec<String> {
    let mut translated = command(&["container", "run", "--name"]);
    translated.push(ownership.name.clone());
    translated.push("--label".to_string());
    translated.push(format!("{DOCKER_RUN_OWNER_LABEL}={}", ownership.token));
    translated.push(request.image.clone());
    translated.extend(request.command.iter().cloned());
    translated
}

/// Preserve the native JSON policy as a type-specific wrapper around the
/// common lifecycle guard. Native output still retains a prefix and rejects a
/// capped stdout rather than letting a suffix accidentally validate as JSON.
fn capture_docker_native_json_command(
    command: &mut Command,
    timeout: Duration,
    surface: DockerBoundedObservationSurface,
) -> Result<DockerNativeJsonObservation> {
    let observation = capture_docker_bounded_command(
        command,
        timeout,
        surface,
        |stdout| capture_docker_native_json_stream(stdout),
        |stderr| capture_docker_native_json_stream(stderr),
    )?;
    Ok(DockerNativeJsonObservation {
        status: observation.status,
        stdout: observation.stdout,
        stderr: observation.stderr,
    })
}

/// Capture arbitrary textual output with a suffix-retaining policy. This is
/// intentionally distinct from native JSON capture: arbitrary output remains
/// valid once bounded because VAT wraps and escapes it itself.
fn capture_docker_bounded_text_command(
    command: &mut Command,
    timeout: Duration,
    surface: DockerBoundedObservationSurface,
) -> Result<DockerBoundedObservation<DockerBoundedTextCapturedStream>> {
    capture_docker_bounded_command(
        command,
        timeout,
        surface,
        |stdout| capture_docker_bounded_text_stream(stdout),
        |stderr| capture_docker_bounded_text_stream(stderr),
    )
}

/// Drain both child pipes concurrently under one isolated process group and
/// one deadline. Stream-specific retention/validation remains outside this
/// helper so native JSON cannot inherit log suffix semantics (or vice versa).
fn capture_docker_bounded_command<T, CaptureStdout, CaptureStderr>(
    command: &mut Command,
    timeout: Duration,
    surface: DockerBoundedObservationSurface,
    capture_stdout: CaptureStdout,
    capture_stderr: CaptureStderr,
) -> Result<DockerBoundedObservation<T>>
where
    T: Send + 'static,
    CaptureStdout: FnOnce(ChildStdout) -> Result<T> + Send + 'static,
    CaptureStderr: FnOnce(ChildStderr) -> Result<T> + Send + 'static,
{
    // The deadline covers both the direct Apple Container process and EOF on
    // both capture pipes. A backend can fork a helper that outlives its root
    // process while retaining stdout/stderr; without an isolated group plus
    // this shared deadline, joining the readers could hang after `try_wait`
    // observed the root exit.
    let deadline = Instant::now() + timeout;
    set_docker_bounded_process_group(command);
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!(
                "spawn Apple Container {} observation",
                surface.apple_operation
            )
        })?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            terminate_docker_bounded_child(&mut child, surface).with_context(|| {
                format!(
                    "stop Apple Container {} child after stdout capture setup failed",
                    surface.apple_operation
                )
            })?;
            bail!(
                "Apple Container {} child did not expose stdout capture",
                surface.apple_operation
            );
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            terminate_docker_bounded_child(&mut child, surface).with_context(|| {
                format!(
                    "stop Apple Container {} child after stderr capture setup failed",
                    surface.apple_operation
                )
            })?;
            bail!(
                "Apple Container {} child did not expose stderr capture",
                surface.apple_operation
            );
        }
    };
    let stdout_reader = match thread::Builder::new()
        .name(format!("vat-docker-{}-stdout", surface.apple_operation))
        .spawn(move || capture_stdout(stdout))
    {
        Ok(reader) => reader,
        Err(error) => {
            if let Err(cleanup_error) = terminate_docker_bounded_child(&mut child, surface) {
                return Err(cleanup_error).context(format!(
                    "start Apple Container {} stdout reader failed and VAT could not stop its isolated process group: {error}",
                    surface.apple_operation
                ));
            }
            return Err(error).with_context(|| {
                format!(
                    "start Apple Container {} stdout reader",
                    surface.apple_operation
                )
            });
        }
    };
    let stderr_reader = match thread::Builder::new()
        .name(format!("vat-docker-{}-stderr", surface.apple_operation))
        .spawn(move || capture_stderr(stderr))
    {
        Ok(reader) => reader,
        Err(error) => {
            if let Err(cleanup_error) = terminate_docker_bounded_child(&mut child, surface) {
                return Err(cleanup_error).context(format!(
                    "start Apple Container {} stderr reader failed and VAT could not stop its isolated process group: {error}",
                    surface.apple_operation
                ));
            }
            if !wait_for_docker_bounded_reader_eof(&stdout_reader) {
                return Err(error).with_context(|| format!(
                    "start Apple Container {} stderr reader; the stopped root left stdout open after bounded cleanup, so its reader was detached rather than joined",
                    surface.apple_operation
                ));
            }
            // EOF was observed above, so this join cannot wait on a
            // pipe-owning descendant. The primary failure remains the stderr
            // reader startup error even if this best-effort drain panics.
            let _ = stdout_reader.join();
            return Err(error).with_context(|| {
                format!(
                    "start Apple Container {} stderr reader",
                    surface.apple_operation
                )
            });
        }
    };

    let outcome = wait_for_docker_bounded_completion(
        &mut child,
        &stdout_reader,
        &stderr_reader,
        deadline,
        surface,
    )?;
    let stdout = join_docker_bounded_reader(stdout_reader, "stdout", surface);
    let stderr = join_docker_bounded_reader(stderr_reader, "stderr", surface);
    match outcome {
        DockerBoundedWaitOutcome::Complete(status) => Ok(DockerBoundedObservation {
            status,
            stdout: stdout?,
            stderr: stderr?,
        }),
        DockerBoundedWaitOutcome::TimedOut => {
            // `wait_for_docker_bounded_completion` reaped the direct root and
            // observed EOF on both capture pipes before this join. If a
            // detached descendant keeps either pipe open, that function
            // returns an error instead of allowing this join to hang.
            let _ = stdout?;
            let _ = stderr?;
            bail!(
                "Apple Container {} observation timed out after {}ms; its isolated process group was killed and its root process reaped",
                surface.apple_operation,
                timeout.as_millis(),
            );
        }
    }
}

fn terminate_docker_bounded_child(
    child: &mut Child,
    surface: DockerBoundedObservationSurface,
) -> Result<()> {
    stop_docker_bounded_process_group(child, surface)
}

enum DockerBoundedWaitOutcome {
    Complete(ExitStatus),
    TimedOut,
}

fn wait_for_docker_bounded_completion<T>(
    child: &mut Child,
    stdout_reader: &thread::JoinHandle<Result<T>>,
    stderr_reader: &thread::JoinHandle<Result<T>>,
    deadline: Instant,
    surface: DockerBoundedObservationSurface,
) -> Result<DockerBoundedWaitOutcome>
where
    T: Send + 'static,
{
    loop {
        if Instant::now() >= deadline {
            if let Err(cleanup_error) = stop_docker_bounded_process_group(child, surface) {
                // The cleanup helper boundedly reaped the direct root after
                // attempting the one safe group signal. Do not join readers
                // here: a descendant may have escaped that group and still
                // own a pipe.
                return Err(cleanup_error).with_context(|| format!(
                    "Apple Container {} observation timed out; VAT could not safely finish isolated process-group cleanup, so capture readers were detached",
                    surface.apple_operation
                ));
            }
            if !wait_for_docker_bounded_readers_eof(stdout_reader, stderr_reader) {
                bail!(
                    "Apple Container {} observation timed out; its isolated process group was killed and root process reaped, but stdout/stderr remained open after the bounded cleanup window (an escaped pipe holder was not joined)",
                    surface.apple_operation
                );
            }
            return Ok(DockerBoundedWaitOutcome::TimedOut);
        }

        // Do not call `try_wait` until both readers reached EOF. `try_wait`
        // reaps the direct root, releasing the numeric PID that names this
        // isolated process group. While a descendant can still own either
        // capture pipe, retaining the root zombie pins that PID/PGID so a
        // deadline KILL cannot be redirected to a recycled process group.
        if docker_bounded_readers_finished(stdout_reader, stderr_reader) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if Instant::now() >= deadline {
                        bail!(
                            "Apple Container {} observation timed out after root and pipe EOF; VAT did not signal a process group after reaping its root",
                            surface.apple_operation
                        );
                    }
                    return Ok(DockerBoundedWaitOutcome::Complete(status));
                }
                Ok(None) => {}
                Err(error) => {
                    let cleanup = stop_docker_bounded_process_group(child, surface);
                    return match cleanup {
                        Ok(()) => Err(error).with_context(|| {
                            format!(
                                "poll Apple Container {} observation",
                                surface.apple_operation
                            )
                        }),
                        Err(cleanup_error) => Err(cleanup_error).context(format!(
                            "poll Apple Container {} observation failed and VAT could not stop its isolated process group: {error}",
                            surface.apple_operation
                        )),
                    };
                }
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            continue;
        }
        thread::sleep(remaining.min(DOCKER_BOUNDED_OBSERVATION_POLL_INTERVAL));
    }
}

fn docker_bounded_readers_finished<T>(
    stdout_reader: &thread::JoinHandle<Result<T>>,
    stderr_reader: &thread::JoinHandle<Result<T>>,
) -> bool
where
    T: Send + 'static,
{
    stdout_reader.is_finished() && stderr_reader.is_finished()
}

/// After a timeout group KILL, give the pipe readers only a fixed cleanup
/// window to observe EOF. A descendant that escaped the dedicated process
/// group can retain a pipe forever; dropping its JoinHandle is safer than
/// waiting indefinitely or reusing a root PGID after it has been reaped.
fn wait_for_docker_bounded_readers_eof<T>(
    stdout_reader: &thread::JoinHandle<Result<T>>,
    stderr_reader: &thread::JoinHandle<Result<T>>,
) -> bool
where
    T: Send + 'static,
{
    wait_for_docker_bounded_eof(|| docker_bounded_readers_finished(stdout_reader, stderr_reader))
}

fn wait_for_docker_bounded_reader_eof<T>(reader: &thread::JoinHandle<Result<T>>) -> bool
where
    T: Send + 'static,
{
    wait_for_docker_bounded_eof(|| reader.is_finished())
}

fn wait_for_docker_bounded_eof(is_finished: impl Fn() -> bool) -> bool {
    let cleanup_deadline = Instant::now() + DOCKER_BOUNDED_OBSERVATION_STOP_TIMEOUT;
    loop {
        if is_finished() {
            return true;
        }
        let remaining = cleanup_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return false;
        }
        thread::sleep(remaining.min(DOCKER_BOUNDED_OBSERVATION_POLL_INTERVAL));
    }
}

#[cfg(unix)]
fn set_docker_bounded_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn set_docker_bounded_process_group(_command: &mut Command) {}

/// Signal the dedicated group before reaping the direct child. Callers must
/// retain the root unreaped until this signal has been attempted: its zombie
/// PID pins the numeric PGID and prevents a recycled-group KILL.
#[cfg(unix)]
fn stop_docker_bounded_process_group(
    child: &mut Child,
    surface: DockerBoundedObservationSurface,
) -> Result<()> {
    let pgid = child.id();
    let signal_result = unsafe { libc::kill(-(pgid as libc::pid_t), libc::SIGKILL) };
    let (group_was_present, group_signal_error) = if signal_result == 0 {
        (true, None)
    } else {
        let error = std::io::Error::last_os_error();
        match error.raw_os_error() {
            Some(libc::ESRCH) => (false, None),
            _ => (false, Some(error)),
        }
    };

    let mut root_exited = child
        .try_wait()
        .with_context(|| {
            format!(
                "poll Apple Container {} root after group KILL",
                surface.apple_operation
            )
        })?
        .is_some();
    let mut direct_kill_error = None;
    if (!group_was_present || group_signal_error.is_some()) && !root_exited {
        if let Err(error) = child.kill() {
            direct_kill_error = Some(error);
        }
    }
    let stop_deadline = Instant::now() + DOCKER_BOUNDED_OBSERVATION_STOP_TIMEOUT;
    while !root_exited && Instant::now() < stop_deadline {
        root_exited = child
            .try_wait()
            .with_context(|| {
                format!(
                    "poll Apple Container {} root after KILL",
                    surface.apple_operation
                )
            })?
            .is_some();
        if !root_exited {
            thread::sleep(DOCKER_BOUNDED_OBSERVATION_POLL_INTERVAL);
        }
    }
    if !root_exited {
        if let Some(error) = direct_kill_error {
            return Err(error).with_context(|| {
                format!(
                    "kill Apple Container {} root after missing or failed group KILL",
                    surface.apple_operation
                )
            });
        }
        bail!(
            "Apple Container {} root did not exit after its process group received KILL",
            surface.apple_operation
        );
    }
    if let Some(error) = group_signal_error {
        return Err(error).with_context(|| {
            format!(
                "send KILL to Apple Container {} process group {pgid}; its direct root was reaped, but descendants may remain outside VAT's control",
                surface.apple_operation
            )
        });
    }
    Ok(())
}

#[cfg(not(unix))]
fn stop_docker_bounded_process_group(
    child: &mut Child,
    surface: DockerBoundedObservationSurface,
) -> Result<()> {
    if child
        .try_wait()
        .with_context(|| {
            format!(
                "poll Apple Container {} root before KILL",
                surface.apple_operation
            )
        })?
        .is_none()
    {
        child.kill().with_context(|| {
            format!(
                "kill Apple Container {} root after timeout",
                surface.apple_operation
            )
        })?;
    }
    let stop_deadline = Instant::now() + DOCKER_BOUNDED_OBSERVATION_STOP_TIMEOUT;
    while Instant::now() < stop_deadline {
        if child
            .try_wait()
            .with_context(|| {
                format!(
                    "poll Apple Container {} root after KILL",
                    surface.apple_operation
                )
            })?
            .is_some()
        {
            return Ok(());
        }
        thread::sleep(DOCKER_BOUNDED_OBSERVATION_POLL_INTERVAL);
    }
    bail!(
        "Apple Container {} root did not exit after KILL",
        surface.apple_operation
    )
}

fn join_docker_bounded_reader<T>(
    reader: thread::JoinHandle<Result<T>>,
    stream: &str,
    surface: DockerBoundedObservationSurface,
) -> Result<T>
where
    T: Send + 'static,
{
    reader
        .join()
        .map_err(|_| {
            anyhow::anyhow!(
                "Apple Container {} {stream} reader panicked",
                surface.apple_operation
            )
        })?
        .with_context(|| {
            format!(
                "capture bounded Apple Container {} {stream}",
                surface.apple_operation
            )
        })
}

fn capture_docker_native_json_stream(reader: impl Read) -> Result<DockerNativeJsonCapturedStream> {
    capture_docker_native_json_stream_with_limit(reader, MAX_DOCKER_NATIVE_JSON_CAPTURE_BYTES)
}

fn capture_docker_native_json_stream_with_limit(
    mut reader: impl Read,
    capture_limit: usize,
) -> Result<DockerNativeJsonCapturedStream> {
    let mut bytes = Vec::with_capacity(capture_limit.min(8 * 1024));
    let mut buffer = [0_u8; 8 * 1024];
    let mut capped = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .context("read Apple Container native JSON child stream")?;
        if read == 0 {
            break;
        }
        let available = capture_limit.saturating_sub(bytes.len());
        let retained = read.min(available);
        bytes.extend_from_slice(&buffer[..retained]);
        capped |= retained != read;
    }
    Ok(DockerNativeJsonCapturedStream { bytes, capped })
}

/// Continuously drain an arbitrary Apple Container text pipe while retaining
/// only its newest bounded suffix. Byte and JSON-serialization caps remain
/// necessary because command output has no bounded record shape.
fn capture_docker_bounded_text_stream(
    reader: impl Read,
) -> Result<DockerBoundedTextCapturedStream> {
    capture_docker_bounded_text_stream_with_limits(
        reader,
        MAX_DOCKER_BOUNDED_TEXT_STREAM_CAPTURE_BYTES,
        MAX_DOCKER_BOUNDED_TEXT_JSON_STRING_BYTES,
    )
}

fn capture_docker_bounded_text_stream_with_limits(
    mut reader: impl Read,
    capture_limit: usize,
    json_string_limit: usize,
) -> Result<DockerBoundedTextCapturedStream> {
    let mut retained = Vec::with_capacity(capture_limit.min(8 * 1024));
    let mut buffer = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .context("read bounded Apple Container text child stream")?;
        if read == 0 {
            break;
        }
        let bytes = &buffer[..read];
        if bytes.len() > capture_limit {
            retained.clear();
            retained.extend_from_slice(&bytes[bytes.len() - capture_limit..]);
            truncated = true;
            continue;
        }
        let overflow = retained
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(capture_limit);
        if overflow > 0 {
            retained.drain(..overflow);
            truncated = true;
        }
        retained.extend_from_slice(bytes);
    }

    let (decoded, utf8_lossy) = match String::from_utf8_lossy(&retained) {
        std::borrow::Cow::Borrowed(text) => (text.to_string(), false),
        std::borrow::Cow::Owned(text) => (text, true),
    };
    let (text, json_truncated) = cap_docker_bounded_text_json_string(decoded, json_string_limit)?;
    Ok(DockerBoundedTextCapturedStream {
        text,
        truncated: truncated || json_truncated,
        utf8_lossy,
    })
}

/// Retain a valid UTF-8 suffix whose encoded JSON-string representation stays
/// inside the advertised cap. Escaping control characters can make a decoded
/// string larger on the wire, so byte count alone is not a safe bound.
fn cap_docker_bounded_text_json_string(
    text: String,
    json_string_limit: usize,
) -> Result<(String, bool)> {
    if docker_bounded_text_json_string_len("")? > json_string_limit {
        bail!("VAT's bounded Docker text JSON string cap cannot encode an empty string");
    }
    if docker_bounded_text_json_string_len(&text)? <= json_string_limit {
        return Ok((text, false));
    }

    let mut boundaries = text
        .char_indices()
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    boundaries.push(text.len());
    let mut lower = 0;
    let mut upper = boundaries.len() - 1;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        let suffix = &text[boundaries[middle]..];
        if docker_bounded_text_json_string_len(suffix)? <= json_string_limit {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    let suffix = text[boundaries[lower]..].to_string();
    debug_assert!(docker_bounded_text_json_string_len(&suffix)
        .is_ok_and(|length| length <= json_string_limit));
    Ok((suffix, true))
}

fn docker_bounded_text_json_string_len(text: &str) -> Result<usize> {
    serde_json::to_vec(text)
        .map(|encoded| encoded.len())
        .context("serialize bounded Docker text JSON stream")
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DockerComposeCommand {
    /// A strict validation-only form for agent preflight. It intentionally
    /// captures no registry/runtime state, so the subsequent real `up` parses
    /// and validates the file again before it can launch anything.
    DryRunUp {
        file: PathBuf,
        project: String,
        build: bool,
    },
    Up {
        file: PathBuf,
        project: String,
        options: DockerComposeUpOptions,
    },
    Ps {
        project: String,
        format: DockerComposePsFormat,
    },
    Logs {
        project: String,
        service: String,
        format: DockerComposeLogsFormat,
    },
    /// A deliberately non-interactive command in one ready service from the
    /// strict Compose profile. Keeping the command as raw argv avoids a shell
    /// layer, and requiring `-T` prevents us from silently claiming Docker
    /// Compose's default TTY semantics over Apple Container.
    Exec {
        project: String,
        service: String,
        command: Vec<String>,
        format: DockerComposeExecFormat,
    },
    Down {
        project: String,
    },
}

/// The intentionally narrow `up` option surface. `wait_timeout_seconds` is
/// present only when an explicit `--wait` was parsed; it controls observation
/// of VAT's durable runner evidence, never Docker healthchecks or a host TCP
/// probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DockerComposeUpOptions {
    build: bool,
    wait_timeout_seconds: Option<u64>,
}

/// No-format preserves Docker-shaped human output plus an additive VAT record.
/// The exact JSON form is an agent surface with a VAT schema, not a claim that
/// VAT implements Docker Compose's JSON output schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockerComposePsFormat {
    Text,
    VatJson,
}

/// Text logs preserve the existing Docker-shaped stream followed by its
/// additive result. VAT-native JSON is a bounded snapshot of VAT's captured
/// stdout/stderr files, not a Docker Compose log format implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockerComposeLogsFormat {
    Text,
    VatJson { tail_lines: usize },
}

/// Text exec keeps its historical inherited child streams followed by an
/// additive handoff. The exact JSON form is an agent-native, bounded capture
/// of one non-interactive child execution; it is not Docker Compose's exec
/// output protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DockerComposeExecFormat {
    Text,
    VatJson,
}

const DEFAULT_COMPOSE_WAIT_TIMEOUT_SECONDS: u64 = 300;
const MAX_COMPOSE_WAIT_TIMEOUT_SECONDS: u64 = 1200;
const COMPOSE_WAIT_POLL_INTERVAL: Duration = Duration::from_millis(100);

fn parse_docker_compose_up_profile(
    file: &Path,
    build: bool,
) -> Result<crate::compose::ParsedDockerComposeProfile> {
    if build {
        crate::compose::parse_docker_compose_build_compat_profile(file)
    } else {
        crate::compose::parse_docker_compose_compat_profile(file)
    }
}

/// Run the supported Compose shapes directly through VAT's typed lifecycle
/// rather than pretending that a multi-step project operation is one Apple
/// Container argv. Every `up` captures its profile before it writes a VAT
/// registry or invokes the runtime.
fn run_compose(args: &[OsString]) -> Result<ExitCode> {
    let command = parse_docker_compose_command(&utf8_args(args)?)?;
    match command {
        DockerComposeCommand::DryRunUp {
            file,
            project,
            build,
        } => {
            let parsed_profile = parse_docker_compose_up_profile(&file, build)?;
            // Return the parser's canonical source identity, rather than the
            // caller's possibly-relative spelling. An agent may execute the
            // returned `next` from another cwd; it must revalidate this exact
            // source file rather than a same-named file in that new cwd.
            let source_path = parsed_profile.file.source_path().to_path_buf();
            print_compose_dry_run_result(&project, &source_path, parsed_profile.profile, build);
            Ok(ExitCode::SUCCESS)
        }
        DockerComposeCommand::Up {
            file,
            project,
            options,
        } => {
            let parsed_profile = parse_docker_compose_up_profile(&file, options.build)?;
            let profile = parsed_profile.profile;
            let imported = crate::commands::compose::import_docker_shim_profile(
                parsed_profile.file,
                project.clone(),
                profile,
            )?;
            let built_images = if options.build {
                if imported.built_images.len() != 1 {
                    bail!(
                        "strict Docker Compose source-build project `{project}` must materialize exactly one VAT-owned image"
                    );
                }
                imported.built_images
            } else {
                Vec::new()
            };
            let cleanup_next = built_images.first().map(|image| {
                format!("docker compose -p {project} down && docker image rm {image}")
            });
            let wait_deadline = options
                .wait_timeout_seconds
                .map(|seconds| Instant::now() + Duration::from_secs(seconds));
            let launch = crate::commands::compose::docker_shim_up(
                project.clone(),
                profile,
                wait_deadline,
                // The wait path owns the one final Docker-shaped result. Keep
                // the regular detached handoff JSON for historical non-wait
                // behavior only.
                options.wait_timeout_seconds.is_none(),
            )?;
            let images = (!built_images.is_empty()).then_some(built_images.as_slice());
            match (options.wait_timeout_seconds, launch) {
                (None, crate::commands::compose::DockerShimLaunch::Launched { .. }) => {
                    print_compose_result(
                        "up",
                        &project,
                        Some(format!("docker compose -p {project} ps")),
                        None,
                        images,
                        cleanup_next.clone(),
                        Some(profile),
                        None,
                    );
                    Ok(ExitCode::SUCCESS)
                }
                (None, crate::commands::compose::DockerShimLaunch::DeadlineElapsedBeforeLaunch) => {
                    bail!(
                        "non-wait Docker-shaped compose launch returned an impossible readiness-deadline outcome"
                    )
                }
                (
                    Some(timeout_seconds),
                    crate::commands::compose::DockerShimLaunch::DeadlineElapsedBeforeLaunch,
                ) => {
                    print_compose_wait_result(
                        &project,
                        profile,
                        images,
                        cleanup_next.clone(),
                        timeout_seconds,
                        DockerComposeWaitResult::timeout_before_launch(&project),
                    );
                    Ok(ExitCode::from(1))
                }
                (
                    Some(timeout_seconds),
                    crate::commands::compose::DockerShimLaunch::Launched {
                        target,
                        deadline_elapsed,
                    },
                ) => {
                    let result = if deadline_elapsed {
                        DockerComposeWaitResult::timeout_after_launch()
                    } else {
                        wait_for_docker_shim_compose_ready(
                            &project,
                            &target,
                            wait_deadline.expect("wait timeout created deadline"),
                        )?
                    };
                    let success = result.is_ready();
                    print_compose_wait_result(
                        &project,
                        profile,
                        images,
                        cleanup_next,
                        timeout_seconds,
                        result,
                    );
                    Ok(if success {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    })
                }
            }
        }
        DockerComposeCommand::Ps { project, format } => {
            // The typed observation was gathered while compose held its
            // StartupClaim and proved Docker shim provenance. Do not reopen
            // the registry here: that could pair text from one project state
            // with a later import's public JSON topology.
            let snapshot = match format {
                DockerComposePsFormat::Text => {
                    crate::commands::compose::docker_shim_ps(project.clone())?
                }
                DockerComposePsFormat::VatJson => {
                    crate::commands::compose::docker_shim_ps_json(project.clone())?
                }
            };
            match format {
                DockerComposePsFormat::Text => print_compose_result(
                    "ps",
                    &project,
                    None,
                    Some("observed"),
                    None,
                    None,
                    Some(snapshot.profile),
                    Some(&snapshot.topology),
                ),
                DockerComposePsFormat::VatJson => print_compose_ps_json_result(&project, &snapshot),
            }
            Ok(ExitCode::SUCCESS)
        }
        DockerComposeCommand::Logs {
            project,
            service,
            format,
        } => match format {
            DockerComposeLogsFormat::Text => {
                let code = crate::commands::compose::docker_shim_logs(project.clone(), service)?;
                if code == ExitCode::SUCCESS {
                    // Stored log replay may end on an arbitrary byte. Keep
                    // the additive handoff on a distinct line for agents
                    // that scan terminal records from mixed text output.
                    print_compose_terminal_record(compose_result(
                        "logs",
                        &project,
                        None,
                        Some("observed"),
                        None,
                        None,
                        None,
                        None,
                    ));
                }
                Ok(code)
            }
            DockerComposeLogsFormat::VatJson { tail_lines } => {
                let snapshot = crate::commands::compose::docker_shim_logs_json(
                    project.clone(),
                    service.clone(),
                    tail_lines,
                )?;
                print_compose_logs_json_result(&project, &service, tail_lines, &snapshot);
                Ok(ExitCode::SUCCESS)
            }
        },
        DockerComposeCommand::Exec {
            project,
            service,
            command,
            format,
        } => match format {
            DockerComposeExecFormat::Text => {
                let status =
                    crate::commands::compose::docker_shim_exec(&project, &service, &command)?;
                print_compose_exec_result(&project, &service, &status);
                Ok(exit_code(status))
            }
            DockerComposeExecFormat::VatJson => {
                let snapshot =
                    crate::commands::compose::docker_shim_exec_json(&project, &service, &command)?;
                print_compose_exec_json_result(&project, &service, &snapshot);
                Ok(exit_code(snapshot.status))
            }
        },
        DockerComposeCommand::Down { project } => {
            let code = crate::commands::compose::docker_shim_down(project.clone())?;
            if code == ExitCode::SUCCESS {
                print_compose_result(
                    "down",
                    &project,
                    None,
                    Some("cleaned_up"),
                    None,
                    None,
                    None,
                    None,
                );
            }
            Ok(code)
        }
    }
}

fn print_compose_result(
    command: &str,
    project: &str,
    next: Option<String>,
    terminal: Option<&str>,
    images: Option<&[String]>,
    cleanup_next: Option<String>,
    profile: Option<crate::compose::DockerComposeProfile>,
    topology: Option<&crate::commands::compose::DockerShimTopologySnapshot>,
) {
    println!(
        "{}",
        compose_result(
            command,
            project,
            next,
            terminal,
            images,
            cleanup_next,
            profile,
            topology,
        )
    );
}

/// Text-mode Compose paths can replay arbitrary child or stored log output.
/// Always separate their additive terminal record from the final byte of that
/// output; agent-native JSON modes own stdout and do not use this helper.
fn print_compose_terminal_record(result: serde_json::Value) {
    println!();
    println!("{result}");
}

fn compose_result(
    command: &str,
    project: &str,
    next: Option<String>,
    terminal: Option<&str>,
    images: Option<&[String]>,
    cleanup_next: Option<String>,
    profile: Option<crate::compose::DockerComposeProfile>,
    topology: Option<&crate::commands::compose::DockerShimTopologySnapshot>,
) -> serde_json::Value {
    let mut result = serde_json::json!({
        "type": "vat_docker_compose",
        "command": command,
        "project": project,
        "backend": "apple-container",
    });
    if let Some(next) = next {
        result["next"] = serde_json::Value::String(next);
    }
    if let Some(terminal) = terminal {
        result["terminal"] = serde_json::Value::String(terminal.to_string());
    }
    if let Some(images) = images {
        result["images"] = serde_json::json!(images);
    }
    if let Some(cleanup_next) = cleanup_next {
        result["cleanup_next"] = serde_json::Value::String(cleanup_next);
    }
    if let Some(profile) = profile {
        result["profile"] = serde_json::Value::String(profile.as_str().to_string());
        // This profile marker is an explicit negative contract: services are
        // independently reachable only from the host's loopback interface;
        // VAT does not claim Compose bridge-network or service-name DNS.
        result["service_name_dns"] = serde_json::Value::Bool(false);
        result["host_loopback_only"] = serde_json::Value::Bool(true);
    }
    if let Some(topology) = topology {
        result["topology"] = serde_json::json!({
            "phase": topology.phase.as_str(),
            "ready": topology.ready,
            "services": topology.services.iter().map(|service| {
                let mut value = serde_json::json!({
                    "name": service.name,
                    "state": service.state.as_str(),
                });
                if let Some(endpoint) = service.endpoint.as_ref() {
                    value["endpoint"] = serde_json::Value::String(endpoint.clone());
                }
                value
            }).collect::<Vec<_>>(),
        });
    }
    result
}

/// Build the one document emitted by `docker compose --dry-run ... up -d`.
/// The dry run is only a file/profile preflight: it deliberately retains no
/// parsed state, and the returned launch command revalidates the file before
/// it can import a registry or start an Apple Container runtime.
fn compose_dry_run_result(
    project: &str,
    file: &Path,
    profile: crate::compose::DockerComposeProfile,
    build: bool,
) -> serde_json::Value {
    let mut launch_argv = vec![
        "docker".to_string(),
        "compose".to_string(),
        "-f".to_string(),
        file.to_string_lossy().into_owned(),
        "-p".to_string(),
        project.to_string(),
        "up".to_string(),
        "-d".to_string(),
    ];
    if build {
        launch_argv.push("--build".to_string());
    }
    let next = launch_argv
        .iter()
        .map(|argument| shell_quote_command_argument(argument))
        .collect::<Vec<_>>()
        .join(" ");
    let mut result = compose_result(
        "up",
        project,
        Some(next),
        Some("preflighted"),
        None,
        None,
        Some(profile),
        None,
    );
    result["schema"] = serde_json::Value::String("vat.docker-compose.preflight.v1".to_string());
    result["dry_run"] = serde_json::Value::Bool(true);
    result["build"] = serde_json::Value::Bool(build);
    result["validated"] = serde_json::Value::Bool(true);
    result["runtime_started"] = serde_json::Value::Bool(false);
    result["registry_written"] = serde_json::Value::Bool(false);
    result["image_built"] = serde_json::Value::Bool(false);
    result["launch_revalidates"] = serde_json::Value::Bool(true);
    result["launch_argv"] = serde_json::json!(launch_argv);
    result
}

fn print_compose_dry_run_result(
    project: &str,
    file: &Path,
    profile: crate::compose::DockerComposeProfile,
    build: bool,
) {
    println!("{}", compose_dry_run_result(project, file, profile, build));
}

/// Each dry-run `next` is a shell command an agent may execute verbatim. Keep
/// every argv element quoted even when simple, so a whitespace/quote-bearing
/// validated file path cannot change the command shape when copied to a shell.
fn shell_quote_command_argument(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\\''"))
}

/// Emit the exact `docker compose ps --format json` agent shape. Its schema is
/// deliberately VAT-owned: accepting a familiar flag does not imply Docker
/// Compose JSON/table/template compatibility.
fn print_compose_ps_json_result(
    project: &str,
    snapshot: &crate::commands::compose::DockerShimPsSnapshot,
) {
    println!("{}", compose_ps_json_result(project, snapshot));
}

fn compose_ps_json_result(
    project: &str,
    snapshot: &crate::commands::compose::DockerShimPsSnapshot,
) -> serde_json::Value {
    let mut result = compose_result(
        "ps",
        project,
        None,
        Some("observed"),
        None,
        None,
        Some(snapshot.profile),
        Some(&snapshot.topology),
    );
    result["schema"] = serde_json::Value::String("vat.docker-compose.ps.v1".to_string());
    result["format"] = serde_json::Value::String("vat_json".to_string());
    result
}

/// Emit one VAT-native bounded log snapshot. It deliberately serializes the
/// two durable VAT capture streams separately rather than claiming Docker
/// Compose's interleaving, follow, timestamp, or template semantics.
fn print_compose_logs_json_result(
    project: &str,
    service: &str,
    tail_lines: usize,
    snapshot: &crate::commands::compose::DockerShimLogSnapshot,
) {
    println!(
        "{}",
        compose_logs_json_result(project, service, tail_lines, snapshot)
    );
}

fn compose_logs_json_result(
    project: &str,
    service: &str,
    tail_lines: usize,
    snapshot: &crate::commands::compose::DockerShimLogSnapshot,
) -> serde_json::Value {
    let mut result = compose_result(
        "logs",
        project,
        Some(format!("docker compose -p {project} ps --format json")),
        Some("observed"),
        None,
        None,
        Some(snapshot.profile),
        None,
    );
    result["schema"] = serde_json::Value::String("vat.docker-compose.logs.v1".to_string());
    result["format"] = serde_json::Value::String("vat_json".to_string());
    result["service"] = serde_json::Value::String(service.to_string());
    result["tail_lines"] = serde_json::json!(tail_lines);
    result["stdout"] = serde_json::Value::String(snapshot.stdout.text.clone());
    result["stderr"] = serde_json::Value::String(snapshot.stderr.text.clone());
    result["stdout_truncated"] = serde_json::Value::Bool(snapshot.stdout.truncated);
    result["stderr_truncated"] = serde_json::Value::Bool(snapshot.stderr.truncated);
    result["stdout_utf8_lossy"] = serde_json::Value::Bool(snapshot.stdout.utf8_lossy);
    result["stderr_utf8_lossy"] = serde_json::Value::Bool(snapshot.stderr.utf8_lossy);
    result["capture_only"] = serde_json::Value::Bool(true);
    result["runtime_invoked"] = serde_json::Value::Bool(false);
    result["compose_record_mutated"] = serde_json::Value::Bool(false);
    result
}

/// Terminal result of the bounded readiness observation attached to
/// `docker compose up -d --wait`. It deliberately contains VAT's typed
/// topology only; this module never opens or probes the advertised host port.
#[derive(Debug, Clone)]
struct DockerComposeWaitResult {
    outcome: &'static str,
    detail: Option<String>,
    topology: Option<crate::commands::compose::DockerShimTopologySnapshot>,
    next: Option<String>,
}

impl DockerComposeWaitResult {
    fn ready(
        project: &str,
        topology: crate::commands::compose::DockerShimTopologySnapshot,
    ) -> Self {
        Self {
            outcome: "ready",
            detail: None,
            topology: Some(topology),
            next: Some(format!("docker compose -p {project} ps")),
        }
    }

    fn failure(
        outcome: &'static str,
        detail: impl Into<String>,
        topology: Option<crate::commands::compose::DockerShimTopologySnapshot>,
        next: Option<String>,
    ) -> Self {
        Self {
            outcome,
            detail: Some(detail.into()),
            topology,
            next,
        }
    }

    fn timeout_before_launch(project: &str) -> Self {
        Self {
            outcome: "timeout",
            detail: Some(
                "readiness observation deadline elapsed before VAT spawned the detached runner; no runtime was launched"
                    .to_string(),
            ),
            topology: None,
            next: Some(format!("docker compose -p {project} ps")),
        }
    }

    fn timeout_after_launch() -> Self {
        Self {
            outcome: "timeout",
            detail: Some(
                "readiness observation deadline elapsed during the detached runner handoff; VAT retained the launch and registry"
                    .to_string(),
            ),
            topology: None,
            // The deadline elapsed before a final claimed observation could
            // prove this exact registry still exists. Do not hand an agent a
            // potentially stale shim command; `terminal=wait_failed` plus
            // the retained-state detail is the honest handoff.
            next: None,
        }
    }

    fn is_ready(&self) -> bool {
        self.outcome == "ready"
    }
}

fn wait_for_docker_shim_compose_ready(
    project: &str,
    target: &crate::commands::compose::DockerShimWaitTarget,
    deadline: Instant,
) -> Result<DockerComposeWaitResult> {
    wait_for_docker_shim_compose_ready_with(project, deadline, || {
        crate::commands::compose::docker_shim_wait_observe(project.to_string(), target, deadline)
    })
}

/// Shared bounded observation loop. Keeping the state machine independent of
/// runtime I/O makes the timeout/degraded/replacement terminal behavior
/// deterministic to unit-test; production supplies the claim-held VAT
/// observation closure above.
fn wait_for_docker_shim_compose_ready_with<F>(
    project: &str,
    deadline: Instant,
    mut observe: F,
) -> Result<DockerComposeWaitResult>
where
    F: FnMut() -> Result<crate::commands::compose::DockerShimWaitObservation>,
{
    let mut last_topology = None;
    loop {
        if Instant::now() >= deadline {
            return Ok(DockerComposeWaitResult::failure(
                "timeout",
                "readiness observation deadline elapsed; VAT retained the detached services and registry",
                None,
                None,
            ));
        }

        let observation = match observe() {
            Ok(observation) => observation,
            Err(error) if Instant::now() >= deadline => {
                return Ok(DockerComposeWaitResult::failure(
                    "timeout",
                    format!(
                        "readiness observation deadline elapsed while waiting for VAT's compose claim: {error}"
                    ),
                    None,
                    None,
                ));
            }
            Err(error) => return Err(error),
        };

        match observation {
            crate::commands::compose::DockerShimWaitObservation::DeadlineElapsedBeforeClaim => {
                return Ok(DockerComposeWaitResult::failure(
                    "timeout",
                    "readiness observation deadline elapsed before VAT acquired a target-matching compose claim",
                    None,
                    None,
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::Ready(snapshot) => {
                if Instant::now() >= deadline {
                    return Ok(DockerComposeWaitResult::failure(
                        "timeout",
                        "VAT reported ready only after the requested readiness observation deadline; endpoints are withheld",
                        None,
                        Some(format!("docker compose -p {project} ps")),
                    ));
                }
                return Ok(DockerComposeWaitResult::ready(project, snapshot.topology));
            }
            crate::commands::compose::DockerShimWaitObservation::Starting(snapshot) => {
                last_topology = Some(snapshot.topology);
            }
            crate::commands::compose::DockerShimWaitObservation::EvidenceUnavailable(detail) => {
                if Instant::now() >= deadline {
                    return Ok(DockerComposeWaitResult::failure(
                        "timeout",
                        format!(
                            "readiness observation deadline elapsed while VAT evidence was unavailable: {detail}"
                        ),
                        last_topology,
                        Some(format!("docker compose -p {project} ps")),
                    ));
                }
            }
            crate::commands::compose::DockerShimWaitObservation::Degraded(snapshot) => {
                return Ok(DockerComposeWaitResult::failure(
                    "degraded",
                    "VAT runner reported Ready but could not prove every exact loopback endpoint; no endpoint is exposed",
                    Some(snapshot.topology),
                    Some(format!("docker compose -p {project} ps")),
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::Inactive(snapshot) => {
                return Ok(DockerComposeWaitResult::failure(
                    "inactive",
                    "the captured launch became inactive before readiness completed",
                    Some(snapshot.topology),
                    Some(format!("docker compose -p {project} ps")),
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::Stopping(snapshot) => {
                return Ok(DockerComposeWaitResult::failure(
                    "stopping",
                    "VAT runner teardown began before readiness completed",
                    Some(snapshot.topology),
                    Some(format!("docker compose -p {project} down")),
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::LifecycleReplaced(detail) => {
                return Ok(DockerComposeWaitResult::failure(
                    "lifecycle_replaced",
                    detail,
                    None,
                    None,
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::Terminal(detail) => {
                return Ok(DockerComposeWaitResult::failure(
                    "terminal", detail, None, None,
                ));
            }
            crate::commands::compose::DockerShimWaitObservation::CleanupUnconfirmed(detail) => {
                return Ok(DockerComposeWaitResult::failure(
                    "cleanup_unconfirmed",
                    detail,
                    None,
                    Some(format!("docker compose -p {project} down")),
                ));
            }
        }

        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(DockerComposeWaitResult::failure(
                "timeout",
                "readiness observation deadline elapsed; VAT retained the detached services and registry",
                last_topology,
                Some(format!("docker compose -p {project} ps")),
            ));
        }
        // `docker_shim_wait_observe` dropped its StartupClaim before it
        // returned. Never sleep while the lifecycle lock is held.
        std::thread::sleep(remaining.min(COMPOSE_WAIT_POLL_INTERVAL));
    }
}

fn print_compose_wait_result(
    project: &str,
    profile: crate::compose::DockerComposeProfile,
    images: Option<&[String]>,
    cleanup_next: Option<String>,
    timeout_seconds: u64,
    wait: DockerComposeWaitResult,
) {
    let ready = wait.is_ready();
    // A source-build image cleanup command starts with Docker-shaped `down`.
    // It is actionable only after this exact wait target was observed Ready;
    // terminal/replaced/bare-timeout outcomes must not point at a possibly
    // different or already-reset lifecycle.
    let cleanup_next = ready.then_some(cleanup_next).flatten();
    let mut result = compose_result(
        "up",
        project,
        wait.next,
        Some(if ready { "wait_ready" } else { "wait_failed" }),
        images,
        cleanup_next,
        Some(profile),
        wait.topology.as_ref(),
    );
    let mut wait_json = serde_json::json!({
        "requested": true,
        "timeout_seconds": timeout_seconds,
        "outcome": wait.outcome,
    });
    if let Some(detail) = wait.detail {
        wait_json["detail"] = serde_json::Value::String(detail);
    }
    result["wait"] = wait_json;
    println!("{result}");
}

/// Compose `exec` forwards the child result but still emits a structured
/// handoff. This matters most on a nonzero child exit: the agent gets a
/// concrete status command rather than having to infer whether the service or
/// its command failed.
fn print_compose_exec_result(project: &str, service: &str, status: &ExitStatus) {
    let child_exit_code = status.code().unwrap_or(1);
    let outcome = if status.success() {
        "completed"
    } else {
        "failed"
    };
    print_compose_terminal_record(serde_json::json!({
        "type": "vat_docker_compose",
        "command": "exec",
        "project": project,
        "service": service,
        "backend": "apple-container",
        "outcome": outcome,
        "child_exit_code": child_exit_code,
        "next": format!("docker compose -p {project} ps"),
    }));
}

/// Emit one bounded, VAT-native capture for an explicitly agent-facing
/// non-interactive `compose exec`. Child stdout and stderr were both drained
/// before this point, so the shim must not replay either raw stream around the
/// one public JSON document.
fn print_compose_exec_json_result(
    project: &str,
    service: &str,
    snapshot: &crate::commands::compose::DockerShimExecSnapshot,
) {
    println!("{}", compose_exec_json_result(project, service, snapshot));
}

fn compose_exec_json_result(
    project: &str,
    service: &str,
    snapshot: &crate::commands::compose::DockerShimExecSnapshot,
) -> serde_json::Value {
    let child_exit_code = snapshot.status.code().unwrap_or(1);
    let outcome = if snapshot.status.success() {
        "completed"
    } else {
        "failed"
    };
    let mut result = compose_result(
        "exec",
        project,
        Some(format!("docker compose -p {project} ps --format json")),
        None,
        None,
        None,
        Some(snapshot.profile),
        None,
    );
    result["schema"] = serde_json::Value::String("vat.docker-compose.exec.v1".to_string());
    result["format"] = serde_json::Value::String("vat_json".to_string());
    result["service"] = serde_json::Value::String(service.to_string());
    result["outcome"] = serde_json::Value::String(outcome.to_string());
    result["child_exit_code"] = serde_json::json!(child_exit_code);
    result["stdout"] = serde_json::Value::String(snapshot.stdout.text.clone());
    result["stderr"] = serde_json::Value::String(snapshot.stderr.text.clone());
    result["stdout_truncated"] = serde_json::Value::Bool(snapshot.stdout.truncated);
    result["stderr_truncated"] = serde_json::Value::Bool(snapshot.stderr.truncated);
    result["stdout_utf8_lossy"] = serde_json::Value::Bool(snapshot.stdout.utf8_lossy);
    result["stderr_utf8_lossy"] = serde_json::Value::Bool(snapshot.stderr.utf8_lossy);
    result["runtime_invoked"] = serde_json::Value::Bool(true);
    result["compose_record_mutated"] = serde_json::Value::Bool(false);
    result
}

fn parse_docker_compose_command(args: &[String]) -> Result<DockerComposeCommand> {
    let mut file = None;
    let mut project = None;
    let mut dry_run = false;
    let mut index = 0;
    while let Some(argument) = args.get(index) {
        match argument.as_str() {
            "--dry-run" => {
                if dry_run {
                    bail!("VAT's docker compose profile accepts --dry-run at most once");
                }
                dry_run = true;
                index += 1;
            }
            "-f" | "--file" => {
                if file.is_some() {
                    bail!("VAT's docker compose profile accepts exactly one -f/--file");
                }
                file = Some(PathBuf::from(next_value(
                    args,
                    &mut index,
                    "docker compose",
                    argument,
                )?));
                index += 1;
            }
            "-p" | "--project-name" => {
                if project.is_some() {
                    bail!("VAT's docker compose profile accepts exactly one -p/--project-name");
                }
                project =
                    Some(next_value(args, &mut index, "docker compose", argument)?.to_string());
                index += 1;
            }
            _ if argument.starts_with("--file=") => {
                if file.is_some() {
                    bail!("VAT's docker compose profile accepts exactly one -f/--file");
                }
                let (_, value) =
                    inline_long_option(argument).context("docker compose --file inline option")?;
                if value.is_empty() {
                    bail!("docker compose option `--file` requires a non-empty value");
                }
                file = Some(PathBuf::from(value));
                index += 1;
            }
            _ if argument.starts_with("--project-name=") => {
                if project.is_some() {
                    bail!("VAT's docker compose profile accepts exactly one -p/--project-name");
                }
                let (_, value) = inline_long_option(argument)
                    .context("docker compose --project-name inline option")?;
                if value.is_empty() {
                    bail!("docker compose option `--project-name` requires a non-empty value");
                }
                project = Some(value.to_string());
                index += 1;
            }
            "--env-file" => bail!(
                "VAT's docker compose profile does not support --env-file or Compose interpolation"
            ),
            _ if is_option(argument) => bail!(
                "unsupported docker compose global option `{argument}` in VAT's strict Apple Container profile"
            ),
            _ => break,
        }
    }

    let verb = args.get(index).context(
        "docker compose requires one of: [--dry-run] up -d [--build] [--wait [--wait-timeout SECONDS]], ps [--format json], logs SERVICE, exec -T SERVICE -- COMMAND, exec -T --format json SERVICE -- COMMAND, down",
    )?;
    index += 1;
    let remaining = &args[index..];
    let project = project.context(
        "VAT's docker compose profile requires an explicit -p/--project-name so it never derives or sanitizes a potentially colliding registry name",
    )?;
    validate_docker_compose_project(&project)?;

    if dry_run && verb != "up" {
        bail!(
            "VAT's docker compose --dry-run supports only `docker compose --dry-run -f FILE -p PROJECT up -d [--build]`"
        );
    }

    match verb.as_str() {
        "up" => {
            let file = file.context(
                "VAT's docker compose up profile requires exactly one -f/--file with a VAT strict Compose profile",
            )?;
            let options = parse_docker_compose_up_options(remaining)?;
            if dry_run {
                if options.wait_timeout_seconds.is_some() {
                    bail!(
                        "VAT's docker compose --dry-run preflight does not accept --wait or --wait-timeout; it never starts a runtime"
                    );
                }
                return Ok(DockerComposeCommand::DryRunUp {
                    file,
                    project,
                    build: options.build,
                });
            }
            Ok(DockerComposeCommand::Up {
                file,
                project,
                options,
            })
        }
        "ps" => {
            if file.is_some() {
                bail!(
                    "VAT's docker compose ps profile accepts only `docker compose -p PROJECT ps [--format json]`"
                );
            }
            let format = parse_docker_compose_ps_format(remaining)?;
            Ok(DockerComposeCommand::Ps { project, format })
        }
        "logs" => {
            if file.is_some() {
                bail!(
                    "VAT's docker compose logs profile accepts only `docker compose -p PROJECT logs SERVICE` or `docker compose -p PROJECT logs --format json [--tail LINES] SERVICE`"
                );
            }
            let (service, format) = parse_docker_compose_logs_format(remaining)?;
            Ok(DockerComposeCommand::Logs {
                project,
                service,
                format,
            })
        }
        "exec" => {
            if file.is_some() {
                bail!(
                    "VAT's docker compose exec profile accepts only `docker compose -p PROJECT exec -T SERVICE -- COMMAND` or `docker compose -p PROJECT exec -T --format json SERVICE -- COMMAND`"
                );
            }
            let (service, command, format) = parse_docker_compose_exec_format(remaining)?;
            Ok(DockerComposeCommand::Exec {
                project,
                service,
                command,
                format,
            })
        }
        "down" => {
            if file.is_some() || !remaining.is_empty() {
                bail!(
                    "VAT's docker compose down profile accepts only `docker compose -p PROJECT down`"
                );
            }
            Ok(DockerComposeCommand::Down { project })
        }
        unsupported => bail!(
            "unsupported docker compose command `{unsupported}`; VAT supports only bounded profile up -d [--wait], ps [--format json], logs [--format json [--tail LINES]] SERVICE, exec -T [--format json] SERVICE -- COMMAND, and down"
        ),
    }
}

/// Parse text exec without changing its historical optional `--` separator,
/// while keeping the JSON form intentionally exact. The JSON marker must
/// appear immediately after `-T`, and its command must follow `--`, so a
/// malformed/misordered option cannot be mistaken for a service or command
/// before the strict runtime proof is acquired.
fn parse_docker_compose_exec_format(
    remaining: &[String],
) -> Result<(String, Vec<String>, DockerComposeExecFormat)> {
    let (tty, after_tty) = remaining.split_first().context(
        "VAT's docker compose exec profile requires explicit non-interactive `-T`: `docker compose -p PROJECT exec -T SERVICE -- COMMAND`",
    )?;
    if tty != "-T" {
        bail!(
            "VAT's docker compose exec profile requires exact non-interactive `-T`; default TTY, --no-tty, --index, --privileged, and other exec flags are unsupported"
        );
    }

    let (format, service_and_command) = match after_tty {
        [flag, value, rest @ ..] if flag == "--format" => {
            if value != "json" {
                bail!(
                    "VAT's docker compose exec JSON mode accepts only `--format json`; it emits VAT's vat.docker-compose.exec.v1 schema, not Docker Compose output templates"
                );
            }
            (DockerComposeExecFormat::VatJson, rest)
        }
        [inline, rest @ ..] if inline.starts_with("--format=") => {
            let (_, value) =
                inline_long_option(inline).context("docker compose exec --format inline option")?;
            if value != "json" {
                bail!(
                    "VAT's docker compose exec JSON mode accepts only `--format=json`; it emits VAT's vat.docker-compose.exec.v1 schema, not Docker Compose output templates"
                );
            }
            (DockerComposeExecFormat::VatJson, rest)
        }
        _ => (DockerComposeExecFormat::Text, after_tty),
    };

    let (service, raw_command) = service_and_command.split_first().context(
        "VAT's docker compose exec profile requires a service and command: `docker compose -p PROJECT exec -T SERVICE -- COMMAND`",
    )?;
    if service.is_empty() || is_option(service) {
        bail!(
            "VAT's docker compose exec profile requires one service immediately after -T or `--format json`"
        );
    }

    let command = match format {
        DockerComposeExecFormat::Text => {
            if raw_command.first().is_some_and(|argument| argument == "--") {
                &raw_command[1..]
            } else {
                raw_command
            }
        }
        DockerComposeExecFormat::VatJson => {
            if !raw_command.first().is_some_and(|argument| argument == "--") {
                bail!(
                    "VAT's docker compose exec JSON mode requires the exact delimiter: `docker compose -p PROJECT exec -T --format json SERVICE -- COMMAND`"
                );
            }
            &raw_command[1..]
        }
    };
    if command.is_empty() || command[0].is_empty() {
        bail!("VAT's docker compose exec profile requires a non-empty command after the service");
    }
    Ok((service.clone(), command.to_vec(), format))
}

/// Support one machine-readable status spelling without inheriting Docker
/// Compose's table/template/filter surface. The resulting document is marked
/// with a VAT schema at emission time.
fn parse_docker_compose_ps_format(remaining: &[String]) -> Result<DockerComposePsFormat> {
    match remaining {
        [] => Ok(DockerComposePsFormat::Text),
        [flag, value] if flag == "--format" && value == "json" => {
            Ok(DockerComposePsFormat::VatJson)
        }
        [inline] if inline == "--format=json" => Ok(DockerComposePsFormat::VatJson),
        _ => bail!(
            "VAT's docker compose ps profile accepts only `docker compose -p PROJECT ps` or `docker compose -p PROJECT ps --format json`; VAT emits its own vat.docker-compose.ps.v1 schema, not Docker Compose templates"
        ),
    }
}

/// Parse the one agent-native log snapshot shape without accepting Docker
/// Compose's streaming, timestamps, prefixes, or generic log formatting.
/// The service is intentionally final so a malformed option cannot be treated
/// as a service name after observation has begun.
fn parse_docker_compose_logs_format(
    remaining: &[String],
) -> Result<(String, DockerComposeLogsFormat)> {
    let mut json = false;
    let mut tail_lines = None;
    let mut index = 0;
    while index < remaining.len() {
        let argument = &remaining[index];
        match argument.as_str() {
            "--format" => {
                if json {
                    bail!("VAT's docker compose logs profile accepts --format json at most once");
                }
                let value = next_value(remaining, &mut index, "docker compose logs", argument)?;
                if value != "json" {
                    bail!(
                        "VAT's docker compose logs JSON mode accepts only `--format json`; it emits VAT's vat.docker-compose.logs.v1 schema, not Docker Compose templates"
                    );
                }
                json = true;
                index += 1;
            }
            _ if argument.starts_with("--format=") => {
                if json {
                    bail!("VAT's docker compose logs profile accepts --format json at most once");
                }
                let (_, value) =
                    inline_long_option(argument).context("docker compose logs --format inline option")?;
                if value != "json" {
                    bail!(
                        "VAT's docker compose logs JSON mode accepts only `--format=json`; it emits VAT's vat.docker-compose.logs.v1 schema, not Docker Compose templates"
                    );
                }
                json = true;
                index += 1;
            }
            "--tail" => {
                if tail_lines.is_some() {
                    bail!("VAT's docker compose logs JSON mode accepts --tail at most once");
                }
                let value = next_value(remaining, &mut index, "docker compose logs", argument)?;
                tail_lines = Some(parse_docker_compose_logs_tail(value)?);
                index += 1;
            }
            _ if argument.starts_with("--tail=") => {
                if tail_lines.is_some() {
                    bail!("VAT's docker compose logs JSON mode accepts --tail at most once");
                }
                let (_, value) =
                    inline_long_option(argument).context("docker compose logs --tail inline option")?;
                tail_lines = Some(parse_docker_compose_logs_tail(value)?);
                index += 1;
            }
            _ if is_option(argument) => bail!(
                "VAT's docker compose logs profile accepts only `logs SERVICE` or `logs --format json [--tail LINES] SERVICE`; --follow, timestamps, prefixes, and other Compose log flags are unsupported"
            ),
            _ => {
                if argument.is_empty() {
                    bail!("VAT's docker compose logs profile requires a non-empty service name");
                }
                if index + 1 != remaining.len() {
                    bail!(
                        "VAT's docker compose logs profile requires the service last: `docker compose -p PROJECT logs --format json [--tail LINES] SERVICE`"
                    );
                }
                let service = argument.clone();
                return match (json, tail_lines) {
                    (false, None) => Ok((service, DockerComposeLogsFormat::Text)),
                    (true, tail_lines) => Ok((
                        service,
                        DockerComposeLogsFormat::VatJson {
                            tail_lines: tail_lines.unwrap_or(
                                crate::commands::compose::DEFAULT_DOCKER_SHIM_LOG_TAIL_LINES,
                            ),
                        },
                    )),
                    (false, Some(_)) => bail!(
                        "VAT's docker compose logs --tail is available only with `--format json`; text mode remains exact `logs SERVICE`"
                    ),
                };
            }
        }
    }
    bail!(
        "VAT's docker compose logs profile requires `docker compose -p PROJECT logs SERVICE` or `docker compose -p PROJECT logs --format json [--tail LINES] SERVICE`"
    )
}

fn parse_docker_compose_logs_tail(value: &str) -> Result<usize> {
    let tail_lines = value
        .parse::<usize>()
        .with_context(|| "VAT's docker compose logs JSON `--tail` requires positive whole lines")?;
    if !(1..=crate::commands::compose::MAX_DOCKER_SHIM_LOG_TAIL_LINES).contains(&tail_lines) {
        bail!(
            "VAT's docker compose logs JSON `--tail` must be between 1 and {} lines",
            crate::commands::compose::MAX_DOCKER_SHIM_LOG_TAIL_LINES
        );
    }
    Ok(tail_lines)
}

/// Parse the strict `up` flags without accepting the rest of Docker Compose's
/// recreate/pull/scale surface. The order is intentionally harmless (Docker
/// users commonly write either `-d --build` or `--build -d`), but both flags
/// must be explicit and unique so no default TTY/foreground semantics leak in.
fn parse_docker_compose_up_options(remaining: &[String]) -> Result<DockerComposeUpOptions> {
    let mut detach = false;
    let mut build = false;
    let mut wait = false;
    let mut wait_timeout_seconds = None;
    let mut index = 0;
    while let Some(argument) = remaining.get(index) {
        match argument.as_str() {
            "-d" | "--detach" if !detach => detach = true,
            "--build" if !build => build = true,
            "--wait" if !wait => wait = true,
            "--wait-timeout" => {
                if wait_timeout_seconds.is_some() {
                    bail!("VAT's docker compose up profile accepts `--wait-timeout` at most once");
                }
                index += 1;
                let value = remaining.get(index).context(
                    "VAT's docker compose up `--wait-timeout` requires positive integer seconds",
                )?;
                wait_timeout_seconds = Some(parse_docker_compose_wait_timeout(value)?);
            }
            _ if argument.starts_with("--wait-timeout=") => {
                if wait_timeout_seconds.is_some() {
                    bail!("VAT's docker compose up profile accepts `--wait-timeout` at most once");
                }
                let (_, value) = inline_long_option(argument)
                    .context("docker compose --wait-timeout inline option")?;
                wait_timeout_seconds = Some(parse_docker_compose_wait_timeout(value)?);
            }
            _ => bail!(
                "VAT's docker compose up profile supports only one explicit -d/--detach, optional one --build, and optional `--wait [--wait-timeout SECONDS]`"
            ),
        }
        index += 1;
    }
    if !detach {
        bail!(
            "VAT's docker compose up profile requires explicit -d/--detach; foreground/recreate semantics are unsupported"
        );
    }
    if wait_timeout_seconds.is_some() && !wait {
        bail!("VAT's docker compose up `--wait-timeout` requires explicit `--wait`");
    }
    Ok(DockerComposeUpOptions {
        build,
        wait_timeout_seconds: wait
            .then_some(wait_timeout_seconds.unwrap_or(DEFAULT_COMPOSE_WAIT_TIMEOUT_SECONDS)),
    })
}

fn parse_docker_compose_wait_timeout(value: &str) -> Result<u64> {
    let seconds = value.parse::<u64>().with_context(|| {
        "VAT's docker compose up `--wait-timeout` requires positive integer seconds"
    })?;
    if !(1..=MAX_COMPOSE_WAIT_TIMEOUT_SECONDS).contains(&seconds) {
        bail!(
            "VAT's docker compose up `--wait-timeout` must be between 1 and {MAX_COMPOSE_WAIT_TIMEOUT_SECONDS} seconds"
        );
    }
    Ok(seconds)
}

fn validate_docker_compose_project(project: &str) -> Result<()> {
    let mut bytes = project.bytes();
    let Some(first) = bytes.next() else {
        bail!("docker compose project name must not be empty");
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit())
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        bail!(
            "VAT's docker compose profile requires a project name already valid without VAT sanitization: [a-z0-9][a-z0-9_-]*"
        );
    }
    Ok(())
}

/// Translate Docker-shaped UTF-8 argv to one explicit public `container`
/// command.  This function is pure so the allowlist is unit-testable without
/// a running Apple Container runtime.
pub fn translate(args: &[OsString]) -> Result<Vec<String>> {
    translate_text(&utf8_args(args)?)
}

fn utf8_args(args: &[OsString]) -> Result<Vec<String>> {
    args.iter()
        .map(|arg| {
            arg.to_str()
                .map(str::to_owned)
                .context("VAT's Docker command shim currently accepts UTF-8 argv only")
        })
        .collect()
}

fn translate_text(args: &[String]) -> Result<Vec<String>> {
    let (verb, rest) = args.split_first().context(
        "missing Docker command; run `docker --help` for VAT's bounded compatibility corpus",
    )?;
    if is_option(verb) {
        bail!(
            "unsupported Docker global option `{verb}`; VAT's shim does not expose Docker contexts, hosts, or Engine configuration"
        );
    }

    match verb.as_str() {
        "build" => translate_build(rest),
        "pull" => translate_image_transfer("pull", rest),
        "push" => translate_image_transfer("push", rest),
        "run" => translate_process(ProcessKind::Run, rest),
        "create" => translate_process(ProcessKind::Create, rest),
        "ps" => translate_ps(rest),
        "images" => translate_images(rest),
        "logs" => translate_logs(rest),
        "stats" => translate_stats(rest),
        "exec" => translate_process(ProcessKind::Exec, rest),
        "inspect" => translate_simple(&["container", "inspect"], rest, 1, None, "docker inspect"),
        "start" => translate_start(rest),
        "stop" => translate_signal("stop", rest, true),
        "kill" => translate_signal("kill", rest, false),
        "rm" => translate_rm(rest),
        "cp" => translate_simple(&["container", "copy"], rest, 2, Some(2), "docker cp"),
        "login" => translate_login(rest),
        "logout" => translate_simple(
            &["container", "registry", "logout"],
            rest,
            1,
            Some(1),
            "docker logout",
        ),
        "image" => translate_image_group(rest),
        "container" => translate_container_group(rest),
        "network" => translate_resource_group("network", rest),
        "volume" => translate_resource_group("volume", rest),
        unsupported => bail!(
            "unsupported Docker command `{unsupported}`; VAT's Apple Container shim supports only: {SUPPORTED_COMMANDS}. It does not implement Docker Engine/API, Compose, buildx, contexts, SDKs, Testcontainers, or devcontainers"
        ),
    }
}

fn translate_build(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "build"]);
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if !is_option(arg) {
            if i + 1 != args.len() {
                bail!("docker build accepts exactly one build context in VAT's shim");
            }
            out.push(arg.clone());
            return Ok(out);
        }
        if matches!(arg.as_str(), "--no-cache" | "--pull" | "-q" | "--quiet") {
            out.push(arg.clone());
            i += 1;
            continue;
        }
        if let Some((flag, value)) = inline_long_option(arg) {
            if is_build_value_option(flag) {
                push_option_value(&mut out, flag, value, "docker build")?;
                i += 1;
                continue;
            }
            bail!("unsupported Docker build option `{flag}` in VAT's fail-closed shim");
        }
        if is_build_value_option(arg) {
            let value = next_value(args, &mut i, "docker build", arg)?;
            push_option_value(&mut out, arg, value, "docker build")?;
            i += 1;
            continue;
        }
        bail!("unsupported Docker build option `{arg}` in VAT's fail-closed shim");
    }
    bail!("docker build requires one explicit build context in VAT's shim")
}

fn is_build_value_option(flag: &str) -> bool {
    matches!(
        flag,
        "-f" | "--file" | "-t" | "--tag" | "--build-arg" | "--platform" | "--target" | "--label"
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessKind {
    Run,
    Create,
    Exec,
}

impl ProcessKind {
    fn apple_verb(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Create => "create",
            Self::Exec => "exec",
        }
    }

    fn docker_verb(self) -> &'static str {
        match self {
            Self::Run => "docker run",
            Self::Create => "docker create",
            Self::Exec => "docker exec",
        }
    }
}

fn translate_process(kind: ProcessKind, args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", kind.apple_verb()]);
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            i += 1;
            break;
        }
        if !is_option(arg) {
            break;
        }
        if matches!(arg.as_str(), "-it" | "-ti") {
            out.push("-i".to_string());
            out.push("-t".to_string());
            i += 1;
            continue;
        }
        if is_process_boolean(kind, arg) {
            out.push(arg.clone());
            i += 1;
            continue;
        }
        if let Some((flag, value)) = inline_long_option(arg) {
            if is_process_value_option(kind, flag) {
                push_process_option_value(&mut out, flag, value, kind)?;
                i += 1;
                continue;
            }
            bail!(
                "unsupported {} option `{flag}` in VAT's fail-closed Apple Container shim",
                kind.docker_verb()
            );
        }
        if is_process_value_option(kind, arg) {
            let value = next_value(args, &mut i, kind.docker_verb(), arg)?;
            push_process_option_value(&mut out, arg, value, kind)?;
            i += 1;
            continue;
        }
        bail!(
            "unsupported {} option `{arg}` in VAT's fail-closed Apple Container shim",
            kind.docker_verb()
        );
    }

    let remaining = &args[i..];
    match kind {
        ProcessKind::Run | ProcessKind::Create => {
            if remaining.is_empty() {
                bail!("{} requires an image", kind.docker_verb());
            }
        }
        ProcessKind::Exec => {
            if remaining.len() < 2 {
                bail!("docker exec requires a container id and a command");
            }
        }
    }
    out.extend_from_slice(remaining);
    Ok(out)
}

fn is_process_boolean(kind: ProcessKind, flag: &str) -> bool {
    match flag {
        "-d" | "--detach" | "-i" | "--interactive" | "-t" | "--tty" => true,
        "--rm" => kind == ProcessKind::Run,
        "--read-only" | "--init" => kind != ProcessKind::Exec,
        _ => false,
    }
}

fn is_process_value_option(kind: ProcessKind, flag: &str) -> bool {
    let common = matches!(
        flag,
        "-e" | "--env" | "--env-file" | "-w" | "--workdir" | "-u" | "--user" | "--ulimit"
    );
    if common {
        return true;
    }
    kind != ProcessKind::Exec
        && matches!(
            flag,
            "-p" | "--publish"
                | "-v"
                | "--volume"
                | "--mount"
                | "--name"
                | "--network"
                | "--label"
                | "--cpus"
                | "-m"
                | "--memory"
                | "--entrypoint"
                | "--dns"
                | "--dns-option"
                | "--dns-search"
                | "--platform"
                | "--shm-size"
                | "--tmpfs"
        )
}

fn push_process_option_value(
    out: &mut Vec<String>,
    flag: &str,
    value: &str,
    kind: ProcessKind,
) -> Result<()> {
    match flag {
        "-p" | "--publish" => validate_publish(value)?,
        "--network" => validate_network(value)?,
        _ => {}
    }
    push_option_value(out, flag, value, kind.docker_verb())
}

fn validate_publish(value: &str) -> Result<()> {
    let endpoint = value.split_once('/').map(|(port, _)| port).unwrap_or(value);
    let (host, container_port) = endpoint.rsplit_once(':').ok_or_else(|| {
        anyhow::anyhow!(
            "unsupported Docker publish spec `{value}`: Apple Container requires an explicit host port in [host-ip:]host-port:container-port form; Docker's bare or dynamic host-port mapping is not supported"
        )
    })?;
    if host.is_empty() || container_port.is_empty() {
        bail!(
            "unsupported Docker publish spec `{value}`: Apple Container requires [host-ip:]host-port:container-port"
        );
    }
    let host_port = host.rsplit_once(':').map(|(_, port)| port).unwrap_or(host);
    if host_port == "0" {
        bail!(
            "unsupported Docker publish spec `{value}`: VAT does not claim undocumented dynamic host-port allocation from Apple Container; choose a concrete host port"
        );
    }
    Ok(())
}

fn validate_network(value: &str) -> Result<()> {
    if matches!(value, "bridge" | "host" | "none") {
        bail!(
            "unsupported Docker network `{value}`: VAT maps only explicitly created Apple Container networks; omit --network for the runtime default or create an Apple Container network first"
        );
    }
    Ok(())
}

fn translate_ps(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "list"]);
    for arg in args {
        match arg.as_str() {
            "-a" | "--all" | "-q" | "--quiet" => out.push(arg.clone()),
            _ if is_option(arg) => {
                bail!(
                    "unsupported docker ps option `{arg}`; Docker format/filter/template output is not compatible with Apple Container"
                )
            }
            _ => bail!("docker ps does not accept positional arguments in VAT's shim"),
        }
    }
    Ok(out)
}

fn translate_images(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "image", "list"]);
    for arg in args {
        match arg.as_str() {
            "-q" | "--quiet" => out.push(arg.clone()),
            _ if is_option(arg) => bail!(
                "unsupported docker images option `{arg}`; Docker's image formatting/filter schema is not compatible with Apple Container"
            ),
            _ => bail!("docker images does not accept positional arguments in VAT's shim"),
        }
    }
    Ok(out)
}

fn translate_logs(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "logs"]);
    let mut i = 0;
    while i < args.len() && is_option(&args[i]) {
        let arg = &args[i];
        if matches!(arg.as_str(), "-f" | "--follow") {
            out.push(arg.clone());
            i += 1;
            continue;
        }
        if let Some((flag, value)) = inline_long_option(arg) {
            if flag == "--tail" {
                push_tail(&mut out, value)?;
                i += 1;
                continue;
            }
        }
        if arg == "--tail" {
            let value = next_value(args, &mut i, "docker logs", arg)?;
            push_tail(&mut out, value)?;
            i += 1;
            continue;
        }
        bail!(
            "unsupported docker logs option `{arg}`; only --follow and a numeric --tail map to Apple Container"
        );
    }
    let remaining = &args[i..];
    if remaining.len() != 1 {
        bail!("docker logs requires exactly one container id in VAT's shim");
    }
    out.push(remaining[0].clone());
    Ok(out)
}

fn push_tail(out: &mut Vec<String>, value: &str) -> Result<()> {
    if value == "all" || value.is_empty() {
        bail!(
            "unsupported docker logs --tail `{value}`: Apple Container accepts a numeric trailing line count; omit --tail for all logs"
        );
    }
    out.push("-n".to_string());
    out.push(value.to_string());
    Ok(())
}

fn translate_start(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "start"]);
    let mut i = 0;
    while i < args.len() && is_option(&args[i]) {
        match args[i].as_str() {
            "-a" | "--attach" | "-i" | "--interactive" => out.push(args[i].clone()),
            unsupported => bail!("unsupported docker start option `{unsupported}`"),
        }
        i += 1;
    }
    let remaining = &args[i..];
    if remaining.len() != 1 {
        bail!(
            "VAT's Apple Container shim supports docker start for exactly one container id per invocation"
        );
    }
    out.push(remaining[0].clone());
    Ok(out)
}

fn translate_signal(action: &str, args: &[String], has_time: bool) -> Result<Vec<String>> {
    let mut out = command(&["container", action]);
    let mut i = 0;
    while i < args.len() && is_option(&args[i]) {
        let arg = &args[i];
        if let Some((flag, value)) = inline_long_option(arg) {
            if flag == "--signal" {
                push_option_value(&mut out, flag, value, &format!("docker {action}"))?;
                i += 1;
                continue;
            }
            if has_time && flag == "--timeout" {
                push_option_value(&mut out, "--time", value, &format!("docker {action}"))?;
                i += 1;
                continue;
            }
        }
        if matches!(arg.as_str(), "-s" | "--signal") {
            let value = next_value(args, &mut i, &format!("docker {action}"), arg)?;
            push_option_value(&mut out, arg, value, &format!("docker {action}"))?;
            i += 1;
            continue;
        }
        if has_time && matches!(arg.as_str(), "-t" | "--timeout") {
            let value = next_value(args, &mut i, &format!("docker {action}"), arg)?;
            push_option_value(&mut out, "--time", value, &format!("docker {action}"))?;
            i += 1;
            continue;
        }
        bail!("unsupported docker {action} option `{arg}`");
    }
    if i == args.len() {
        bail!("docker {action} requires at least one container id");
    }
    out.extend_from_slice(&args[i..]);
    Ok(out)
}

fn translate_rm(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "delete"]);
    let mut i = 0;
    while i < args.len() && is_option(&args[i]) {
        match args[i].as_str() {
            "-f" | "--force" => out.push(args[i].clone()),
            unsupported => bail!("unsupported docker rm option `{unsupported}`"),
        }
        i += 1;
    }
    if i == args.len() {
        bail!("docker rm requires at least one container id");
    }
    out.extend_from_slice(&args[i..]);
    Ok(out)
}

fn translate_login(args: &[String]) -> Result<Vec<String>> {
    let mut out = command(&["container", "registry", "login"]);
    let mut i = 0;
    while i < args.len() && is_option(&args[i]) {
        let arg = &args[i];
        if arg == "--password-stdin" {
            out.push(arg.clone());
            i += 1;
            continue;
        }
        if let Some((flag, value)) = inline_long_option(arg) {
            if flag == "--username" {
                push_option_value(&mut out, flag, value, "docker login")?;
                i += 1;
                continue;
            }
        }
        if matches!(arg.as_str(), "-u" | "--username") {
            let value = next_value(args, &mut i, "docker login", arg)?;
            push_option_value(&mut out, arg, value, "docker login")?;
            i += 1;
            continue;
        }
        bail!("unsupported docker login option `{arg}`");
    }
    let remaining = &args[i..];
    if remaining.len() != 1 {
        bail!("docker login requires exactly one registry server in VAT's shim");
    }
    out.push(remaining[0].clone());
    Ok(out)
}

fn translate_image_group(args: &[String]) -> Result<Vec<String>> {
    let (verb, rest) = args
        .split_first()
        .context("docker image requires a subcommand")?;
    match verb.as_str() {
        "ls" | "list" => translate_images(rest),
        "rm" | "delete" => translate_simple(
            &["container", "image", "delete"],
            rest,
            1,
            None,
            "docker image rm",
        ),
        "inspect" => translate_simple(
            &["container", "image", "inspect"],
            rest,
            1,
            None,
            "docker image inspect",
        ),
        "pull" | "push" => translate_image_transfer(verb, rest),
        "tag" => translate_simple(
            &["container", "image", "tag"],
            rest,
            2,
            Some(2),
            "docker image tag",
        ),
        "prune" => bail!(
            "docker image prune is unsupported in VAT's fail-closed shim because Apple Container prune is global; remove explicit image references instead"
        ),
        unsupported => bail!("unsupported docker image command `{unsupported}`"),
    }
}

fn translate_image_transfer(action: &str, args: &[String]) -> Result<Vec<String>> {
    translate_simple(
        &["container", "image", action],
        args,
        1,
        Some(1),
        &format!("docker {action}"),
    )
}

fn translate_container_group(args: &[String]) -> Result<Vec<String>> {
    let (verb, rest) = args
        .split_first()
        .context("docker container requires a subcommand")?;
    match verb.as_str() {
        "ls" | "list" | "ps" => translate_ps(rest),
        "run" => translate_process(ProcessKind::Run, rest),
        "create" => translate_process(ProcessKind::Create, rest),
        "start" => translate_start(rest),
        "stop" => translate_signal("stop", rest, true),
        "kill" => translate_signal("kill", rest, false),
        "rm" | "delete" => translate_rm(rest),
        "logs" => translate_logs(rest),
        "exec" => translate_process(ProcessKind::Exec, rest),
        "inspect" => translate_simple(
            &["container", "inspect"],
            rest,
            1,
            None,
            "docker container inspect",
        ),
        "cp" => translate_simple(
            &["container", "copy"],
            rest,
            2,
            Some(2),
            "docker container cp",
        ),
        unsupported => bail!("unsupported docker container command `{unsupported}`"),
    }
}

fn translate_resource_group(resource: &str, args: &[String]) -> Result<Vec<String>> {
    let (verb, rest) = args
        .split_first()
        .with_context(|| format!("docker {resource} requires a subcommand"))?;
    let apple_verb = match verb.as_str() {
        "ls" | "list" => "list",
        "rm" | "delete" => "delete",
        "create" => "create",
        "inspect" => "inspect",
        "prune" => bail!(
            "docker {resource} prune is unsupported in VAT's fail-closed shim because Apple Container prune is global; delete explicit VAT-owned {resource} names instead"
        ),
        unsupported => bail!("unsupported docker {resource} command `{unsupported}`"),
    };
    let (minimum, maximum) = match apple_verb {
        "list" => (0, Some(0)),
        "create" => (1, Some(1)),
        "delete" | "inspect" => (1, None),
        _ => unreachable!(),
    };
    translate_simple(
        &["container", resource, apple_verb],
        rest,
        minimum,
        maximum,
        &format!("docker {resource} {verb}"),
    )
}

fn translate_simple(
    prefix: &[&str],
    args: &[String],
    minimum: usize,
    maximum: Option<usize>,
    docker_command: &str,
) -> Result<Vec<String>> {
    if args.iter().any(|arg| is_option(arg)) {
        let option = args
            .iter()
            .find(|arg| is_option(arg))
            .expect("known option");
        bail!("unsupported {docker_command} option `{option}` in VAT's fail-closed shim");
    }
    if args.len() < minimum || maximum.is_some_and(|maximum| args.len() > maximum) {
        let expectation = match (minimum, maximum) {
            (0, Some(0)) => "no positional arguments".to_string(),
            (count, Some(maximum)) if count == maximum => {
                format!("exactly {count} positional argument(s)")
            }
            (count, None) => format!("at least {count} positional argument(s)"),
            (count, Some(maximum)) => {
                format!("between {count} and {maximum} positional argument(s)")
            }
        };
        bail!("{docker_command} requires {expectation} in VAT's shim");
    }
    let mut out = command(prefix);
    out.extend_from_slice(args);
    Ok(out)
}

fn next_value<'a>(
    args: &'a [String],
    index: &mut usize,
    command_name: &str,
    flag: &str,
) -> Result<&'a str> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .with_context(|| format!("{command_name} option `{flag}` requires a value"))
}

fn push_option_value(
    out: &mut Vec<String>,
    flag: &str,
    value: &str,
    command_name: &str,
) -> Result<()> {
    if value.is_empty() {
        bail!("{command_name} option `{flag}` requires a non-empty value");
    }
    out.push(flag.to_string());
    out.push(value.to_string());
    Ok(())
}

fn inline_long_option(value: &str) -> Option<(&str, &str)> {
    value.strip_prefix("--").and_then(|_| value.split_once('='))
}

fn command(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn is_option(value: &str) -> bool {
    value.starts_with('-') && value != "-"
}

fn exit_code(status: ExitStatus) -> ExitCode {
    let code = status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .unwrap_or(1);
    ExitCode::from(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::io::Cursor;
    #[cfg(unix)]
    use std::{ffi::CString, os::unix::ffi::OsStrExt};

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn translates_common_run_to_public_container_cli() {
        assert_eq!(
            translate(&args(&[
                "run",
                "--name",
                "web",
                "-p",
                "127.0.0.1:8080:80",
                "-e",
                "MODE=test",
                "nginx:alpine",
            ]))
            .expect("translate run"),
            [
                "container",
                "run",
                "--name",
                "web",
                "-p",
                "127.0.0.1:8080:80",
                "-e",
                "MODE=test",
                "nginx:alpine",
            ]
        );
    }

    #[test]
    fn maps_lifecycle_and_image_aliases() {
        assert_eq!(
            translate(&args(&["ps", "-a", "-q"])).expect("translate ps"),
            ["container", "list", "-a", "-q"]
        );
        assert_eq!(
            translate(&args(&["image", "rm", "demo:latest"])).expect("translate image rm"),
            ["container", "image", "delete", "demo:latest"]
        );
        assert_eq!(
            translate(&args(&["cp", "demo:/tmp/out", "./out"])).expect("translate copy"),
            ["container", "copy", "demo:/tmp/out", "./out"]
        );
    }

    #[test]
    fn maps_the_documented_agent_command_corpus() {
        let cases: &[(&[&str], &[&str])] = &[
            (
                &[
                    "build",
                    "-t",
                    "demo:latest",
                    "--build-arg",
                    "MODE=test",
                    ".",
                ],
                &[
                    "container",
                    "build",
                    "-t",
                    "demo:latest",
                    "--build-arg",
                    "MODE=test",
                    ".",
                ],
            ),
            (
                &["pull", "alpine:3.20"],
                &["container", "image", "pull", "alpine:3.20"],
            ),
            (
                &["push", "demo:latest"],
                &["container", "image", "push", "demo:latest"],
            ),
            (
                &["create", "--name", "demo", "alpine:3.20"],
                &["container", "create", "--name", "demo", "alpine:3.20"],
            ),
            (
                &["logs", "--tail", "5", "demo"],
                &["container", "logs", "-n", "5", "demo"],
            ),
            (
                &[
                    "stats",
                    "--no-stream",
                    "--format=json",
                    "agent-web",
                    "agent-db",
                ],
                &[
                    "container",
                    "stats",
                    "--format",
                    "json",
                    "--no-stream",
                    "agent-web",
                    "agent-db",
                ],
            ),
            (
                &["exec", "-it", "demo", "sh"],
                &["container", "exec", "-i", "-t", "demo", "sh"],
            ),
            (
                &["stop", "--timeout", "2", "demo"],
                &["container", "stop", "--time", "2", "demo"],
            ),
            (
                &["kill", "--signal", "TERM", "demo"],
                &["container", "kill", "--signal", "TERM", "demo"],
            ),
            (
                &["rm", "--force", "demo"],
                &["container", "delete", "--force", "demo"],
            ),
            (
                &["login", "--username", "agent", "registry.example"],
                &[
                    "container",
                    "registry",
                    "login",
                    "--username",
                    "agent",
                    "registry.example",
                ],
            ),
            (
                &["network", "create", "agent-net"],
                &["container", "network", "create", "agent-net"],
            ),
            (
                &["volume", "create", "agent-cache"],
                &["container", "volume", "create", "agent-cache"],
            ),
            (
                &["container", "ls", "--all"],
                &["container", "list", "--all"],
            ),
        ];

        for (docker, container) in cases {
            assert_eq!(
                translate(&args(docker)).expect("translate documented command"),
                *container,
                "docker argv: {docker:?}"
            );
        }
    }

    #[test]
    fn docker_stats_accepts_only_the_non_streaming_apple_native_json_shape() {
        let parse = |values: &[&str]| {
            parse_docker_stats_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };
        for values in [
            ["--no-stream", "--format", "json", "agent-web", "agent-db"].as_slice(),
            ["--format=json", "--no-stream", "agent-web"].as_slice(),
        ] {
            assert!(
                parse(values).is_ok(),
                "exact Docker stats argv must parse: {values:?}"
            );
        }
        for values in [
            ["--no-stream", "--format", "json"].as_slice(),
            ["--format", "table", "--no-stream", "agent-web"].as_slice(),
            ["--no-stream", "--format=json", "--format=json", "agent-web"].as_slice(),
            [
                "--no-stream",
                "--no-stream",
                "--format",
                "json",
                "agent-web",
            ]
            .as_slice(),
            ["--stream", "--format", "json", "agent-web"].as_slice(),
            ["--all", "--no-stream", "--format", "json", "agent-web"].as_slice(),
            ["--no-stream", "--format", "json", "agent-web", "--all"].as_slice(),
            ["--no-stream", "--format", "json", "--", "agent-web"].as_slice(),
            ["--no-stream", "--format", "json", ""].as_slice(),
            ["--no-stream", "--format", "json", "two words"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker stats argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_system_df_json_accepts_only_one_success_only_global_report_shape() {
        let parse = |values: &[&str]| {
            parse_docker_system_df_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        for values in [
            ["--format", "json"].as_slice(),
            ["--format=json"].as_slice(),
        ] {
            assert_eq!(
                parse(values).expect("exact Docker system df JSON argv"),
                DockerSystemDfJsonRequest,
                "docker system df JSON argv: {values:?}"
            );
        }

        for values in [
            [].as_slice(),
            ["--format", "table"].as_slice(),
            ["--format=yaml"].as_slice(),
            ["--format=json", "--format=json"].as_slice(),
            ["--verbose", "--format=json"].as_slice(),
            ["-v", "--format=json"].as_slice(),
            ["--format=json", "unexpected"].as_slice(),
            ["--format=json", "--"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker system df JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_system_df_json_dispatches_only_the_direct_selector_form() {
        for values in [
            ["system", "df", "--format", "json"].as_slice(),
            ["system", "df", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_system_df_json_request_from_argv(&args(values))
                    .expect("decode documented Docker system df JSON argv")
                    .is_some(),
                "documented system df JSON form must use bounded native capture: {values:?}"
            );
        }
        for values in [
            ["system", "df"].as_slice(),
            ["system", "info", "--format=json"].as_slice(),
            ["stats", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_system_df_json_request_from_argv(&args(values))
                    .expect("leave unsupported/non-selector argv outside strict system df parser")
                    .is_none(),
                "only direct selector-bearing docker system df may widen native capture: {values:?}"
            );
        }
        assert!(
            docker_system_df_json_request_from_argv(&args(&[
                "system",
                "df",
                "--format=json",
                "unexpected",
            ]))
            .is_err(),
            "a selector-bearing malformed system df argv must be claimed and rejected before raw dispatch"
        );
        assert!(
            translate(&args(&["system", "df"])).is_err(),
            "raw docker system df must remain unsupported rather than inheriting unbounded text output"
        );
    }

    #[test]
    fn docker_system_df_json_uses_the_canonical_apple_global_disk_argv() {
        assert_eq!(
            apple_container_system_df_json_argv(),
            ["container", "system", "df", "--format", "json"],
            "strict Docker system df must invoke only Apple's public global JSON report argv"
        );
    }

    #[test]
    fn docker_ps_json_accepts_only_the_documented_apple_native_inventory_forms() {
        let parse = |values: &[&str]| {
            parse_docker_ps_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        assert_eq!(
            parse(["--format", "json"].as_slice()).expect("bare JSON inventory"),
            DockerPsJsonRequest { all: false }
        );
        for values in [
            ["--format=json", "--all"].as_slice(),
            ["-a", "--format=json"].as_slice(),
            ["--all", "--format", "json"].as_slice(),
        ] {
            assert_eq!(
                parse(values).expect("exact all JSON inventory"),
                DockerPsJsonRequest { all: true },
                "docker ps JSON argv: {values:?}"
            );
        }

        for values in [
            [].as_slice(),
            ["--format", "table"].as_slice(),
            ["--format=json", "--format=json"].as_slice(),
            ["--all", "-a", "--format=json"].as_slice(),
            ["--quiet", "--format=json"].as_slice(),
            ["-q", "--format=json"].as_slice(),
            ["--filter", "status=running", "--format=json"].as_slice(),
            ["--format=json", "agent-web"].as_slice(),
            ["--format=json", "agent-web", "--all"].as_slice(),
            ["--format=json", "--unknown"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker ps JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_ps_json_dispatches_only_documented_direct_and_container_list_aliases() {
        for values in [
            ["ps", "--format=json"].as_slice(),
            ["container", "ls", "--all", "--format", "json"].as_slice(),
            ["container", "list", "-a", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_ps_json_request_from_argv(&args(values))
                    .expect("decode documented Docker ps JSON argv")
                    .is_some(),
                "documented direct/alias JSON form must use bounded native capture: {values:?}"
            );
        }
        for values in [
            ["ps", "--all"].as_slice(),
            ["container", "ps", "--format=json"].as_slice(),
            ["images", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_ps_json_request_from_argv(&args(values))
                    .expect("decode non-JSON or undocumented Docker argv")
                    .is_none(),
                "non-JSON text form or undocumented alias must not widen the capture surface: {values:?}"
            );
        }
    }

    #[test]
    fn docker_images_json_accepts_only_one_exact_apple_native_inventory_shape() {
        let parse = |values: &[&str]| {
            parse_docker_images_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        for values in [
            ["--format", "json"].as_slice(),
            ["--format=json"].as_slice(),
        ] {
            assert_eq!(
                parse(values).expect("exact image JSON inventory"),
                DockerImagesJsonRequest,
                "docker images JSON argv: {values:?}"
            );
        }

        for values in [
            [].as_slice(),
            ["--format", "table"].as_slice(),
            ["--format", "yaml"].as_slice(),
            ["--format=toml"].as_slice(),
            ["--format={{.Repository}}"].as_slice(),
            ["--format=json", "--format=json"].as_slice(),
            ["--quiet", "--format=json"].as_slice(),
            ["-q", "--format=json"].as_slice(),
            ["--all", "--format=json"].as_slice(),
            ["--filter", "dangling=true", "--format=json"].as_slice(),
            ["--format=json", "demo:latest"].as_slice(),
            ["demo:latest", "--format=json"].as_slice(),
            ["--format=json", "--"].as_slice(),
            ["--format=json", "--no-trunc"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker images JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_images_json_dispatches_only_direct_and_image_group_list_aliases() {
        for values in [
            ["images", "--format=json"].as_slice(),
            ["image", "ls", "--format", "json"].as_slice(),
            ["image", "list", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_images_json_request_from_argv(&args(values))
                    .expect("decode documented Docker images JSON argv")
                    .is_some(),
                "documented image JSON alias must use bounded native capture: {values:?}"
            );
        }

        for values in [
            ["images", "-q"].as_slice(),
            ["image", "ls", "--quiet"].as_slice(),
            ["image", "list", "-q"].as_slice(),
            ["image", "images", "--format=json"].as_slice(),
            ["ps", "--format=json"].as_slice(),
        ] {
            assert!(
                docker_images_json_request_from_argv(&args(values))
                    .expect("decode non-JSON or undocumented Docker argv")
                    .is_none(),
                "text/quiet or undocumented image form must not widen native capture: {values:?}"
            );
        }

        for (values, expected) in [
            (
                ["images"].as_slice(),
                ["container", "image", "list"].as_slice(),
            ),
            (
                ["images", "-q"].as_slice(),
                ["container", "image", "list", "-q"].as_slice(),
            ),
            (
                ["image", "ls"].as_slice(),
                ["container", "image", "list"].as_slice(),
            ),
            (
                ["image", "ls", "--quiet"].as_slice(),
                ["container", "image", "list", "--quiet"].as_slice(),
            ),
            (
                ["image", "list"].as_slice(),
                ["container", "image", "list"].as_slice(),
            ),
            (
                ["image", "list", "-q"].as_slice(),
                ["container", "image", "list", "-q"].as_slice(),
            ),
        ] {
            assert_eq!(
                translate(&args(values)).expect("preserve pre-existing text image translation"),
                expected,
                "text/quiet image form must retain generic translation: {values:?}"
            );
        }
    }

    #[test]
    fn docker_images_json_uses_the_canonical_apple_image_list_argv() {
        assert_eq!(
            apple_container_images_json_argv(&DockerImagesJsonRequest),
            ["container", "image", "list", "--format", "json"],
            "strict Docker image inventory must invoke only Apple's public JSON list argv"
        );
    }

    #[test]
    fn docker_inspect_json_accepts_only_one_safe_direct_container_shape() {
        let parse = |values: &[&str]| {
            parse_docker_inspect_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        assert_eq!(
            parse(["--format", "json", "agent-web"].as_slice())
                .expect("separated direct container inspect JSON"),
            DockerInspectJsonRequest {
                container_id: "agent-web".to_string(),
            }
        );
        assert_eq!(
            parse(["--format=json", "agent_web.2"].as_slice())
                .expect("inline direct container inspect JSON"),
            DockerInspectJsonRequest {
                container_id: "agent_web.2".to_string(),
            }
        );

        for values in [
            [].as_slice(),
            ["--format", "json"].as_slice(),
            ["--format", "table", "agent-web"].as_slice(),
            ["--format", "yaml", "agent-web"].as_slice(),
            ["--format=toml", "agent-web"].as_slice(),
            ["--format={{.Id}}", "agent-web"].as_slice(),
            ["--format=json", "--format=json", "agent-web"].as_slice(),
            ["--type", "container", "--format=json", "agent-web"].as_slice(),
            ["--size", "--format=json", "agent-web"].as_slice(),
            ["--filter", "name=agent-web", "--format=json", "agent-web"].as_slice(),
            ["--format=json", "agent-web", "agent-db"].as_slice(),
            ["agent-web", "--format=json"].as_slice(),
            ["--format=json", "--"].as_slice(),
            ["--format=json", "-agent-web"].as_slice(),
            ["--format=json", "two words"].as_slice(),
            ["--format=json", "agent/web"].as_slice(),
            ["--format=json", "--unknown"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker inspect JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_inspect_json_dispatches_only_direct_and_container_inspect_aliases() {
        for values in [
            ["inspect", "--format=json", "agent-web"].as_slice(),
            ["container", "inspect", "--format", "json", "agent-web"].as_slice(),
        ] {
            assert!(
                docker_inspect_json_request_from_argv(&args(values))
                    .expect("decode documented Docker inspect JSON argv")
                    .is_some(),
                "documented inspect JSON alias must use bounded native capture: {values:?}"
            );
        }

        for values in [
            ["inspect", "agent-web"].as_slice(),
            ["container", "inspect", "agent-web"].as_slice(),
            ["image", "inspect", "--format=json", "agent-web"].as_slice(),
            ["network", "inspect", "--format=json", "default"].as_slice(),
            ["volume", "inspect", "--format=json", "cache"].as_slice(),
        ] {
            assert!(
                docker_inspect_json_request_from_argv(&args(values))
                    .expect("decode text or out-of-scope Docker inspect argv")
                    .is_none(),
                "only direct container inspect aliases may widen to native capture: {values:?}"
            );
        }

        for values in [
            ["inspect", "agent-web"].as_slice(),
            ["container", "inspect", "agent-web"].as_slice(),
        ] {
            assert_eq!(
                translate(&args(values)).expect("preserve pre-existing text inspect translation"),
                ["container", "inspect", "agent-web"],
                "unformatted inspect must retain generic translation: {values:?}"
            );
        }
    }

    #[test]
    fn docker_inspect_json_uses_the_canonical_apple_container_inspect_argv() {
        assert_eq!(
            apple_container_inspect_json_argv(&DockerInspectJsonRequest {
                container_id: "agent-web".to_string(),
            }),
            ["container", "inspect", "agent-web"],
            "strict Docker inspect JSON must strip its selector and invoke only Apple's public inspect argv"
        );
    }

    #[test]
    fn docker_image_inspect_json_accepts_only_one_safe_direct_image_shape() {
        let parse = |values: &[&str]| {
            parse_docker_image_inspect_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        assert_eq!(
            parse(["--format", "json", "alpine:3.20"].as_slice())
                .expect("separated direct image inspect JSON"),
            DockerImageInspectJsonRequest {
                image_reference: "alpine:3.20".to_string(),
            }
        );
        assert_eq!(
            parse(
                [
                    "--format=json",
                    "registry.example:5000/team/image@sha256:opaque",
                ]
                .as_slice(),
            )
            .expect("inline direct image inspect JSON"),
            DockerImageInspectJsonRequest {
                image_reference: "registry.example:5000/team/image@sha256:opaque".to_string(),
            }
        );

        for values in [
            [].as_slice(),
            ["--format", "json"].as_slice(),
            ["--format", "table", "alpine:3.20"].as_slice(),
            ["--format", "yaml", "alpine:3.20"].as_slice(),
            ["--format=toml", "alpine:3.20"].as_slice(),
            ["--format={{.Id}}", "alpine:3.20"].as_slice(),
            ["--format=json", "--format=json", "alpine:3.20"].as_slice(),
            ["--platform", "linux/arm64", "--format=json", "alpine:3.20"].as_slice(),
            ["--format=json", "alpine:3.20", "busybox:latest"].as_slice(),
            ["alpine:3.20", "--format=json"].as_slice(),
            ["--format=json", "--", "alpine:3.20"].as_slice(),
            ["--format=json", "alpine:3.20", "--"].as_slice(),
            ["--format=json", "-alpine:3.20"].as_slice(),
            ["--format=json", "two words"].as_slice(),
            ["--format=json", ""].as_slice(),
            ["--format=json", "--unknown"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact Docker image inspect JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_image_inspect_json_dispatches_only_direct_image_inspect() {
        for values in [
            ["image", "inspect", "--format=json", "alpine:3.20"].as_slice(),
            [
                "image",
                "inspect",
                "--format",
                "json",
                "registry.example/team/image:latest",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_image_inspect_json_request_from_argv(&args(values))
                    .expect("decode documented Docker image inspect JSON argv")
                    .is_some(),
                "documented direct image inspect JSON must use bounded native capture: {values:?}"
            );
        }

        for values in [
            ["image", "inspect", "alpine:3.20"].as_slice(),
            ["inspect", "--format=json", "alpine:3.20"].as_slice(),
            [
                "container",
                "image",
                "inspect",
                "--format=json",
                "alpine:3.20",
            ]
            .as_slice(),
            ["image", "ls", "--format=json"].as_slice(),
            ["network", "inspect", "--format=json", "default"].as_slice(),
        ] {
            assert!(
                docker_image_inspect_json_request_from_argv(&args(values))
                    .expect("decode text or out-of-scope Docker argv")
                    .is_none(),
                "only direct docker image inspect may widen to native capture: {values:?}"
            );
        }

        assert_eq!(
            translate(&args(["image", "inspect", "alpine:3.20"].as_slice()))
                .expect("preserve pre-existing unformatted image inspect translation"),
            ["container", "image", "inspect", "alpine:3.20"],
            "unformatted image inspect must retain generic translation"
        );
    }

    #[test]
    fn docker_image_inspect_json_uses_the_canonical_apple_image_inspect_argv() {
        assert_eq!(
            apple_container_image_inspect_json_argv(&DockerImageInspectJsonRequest {
                image_reference: "alpine:3.20".to_string(),
            }),
            ["container", "image", "inspect", "alpine:3.20"],
            "strict Docker image inspect JSON must strip its selector and invoke only Apple's public image inspect argv"
        );
    }

    #[test]
    fn docker_exec_json_accepts_only_one_bounded_noninteractive_shape() {
        let parse = |values: &[&str]| {
            parse_docker_exec_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        assert_eq!(
            parse(
                [
                    "--format",
                    "json",
                    "--timeout",
                    "17",
                    "agent-web",
                    "--",
                    "echo",
                    "--format=json",
                ]
                .as_slice()
            )
            .expect("separated strict direct exec JSON"),
            DockerExecJsonRequest {
                container_id: "agent-web".to_string(),
                timeout_seconds: 17,
                command: vec!["echo".to_string(), "--format=json".to_string()],
            }
        );
        assert_eq!(
            parse(
                [
                    "--timeout=1200",
                    "--format=json",
                    "agent_web.2",
                    "--",
                    "fixture-command",
                ]
                .as_slice()
            )
            .expect("inline strict direct exec JSON"),
            DockerExecJsonRequest {
                container_id: "agent_web.2".to_string(),
                timeout_seconds: 1200,
                command: vec!["fixture-command".to_string()],
            }
        );

        for values in [
            [].as_slice(),
            ["--format=json", "--timeout=2", "agent-web"].as_slice(),
            ["--format=json", "--timeout=2", "agent-web", "--"].as_slice(),
            ["--format=json", "--timeout=2", "--", "echo"].as_slice(),
            ["--format=table", "--timeout=2", "agent-web", "--", "echo"].as_slice(),
            [
                "--format=json",
                "--format=json",
                "--timeout=2",
                "agent-web",
                "--",
                "echo",
            ]
            .as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--timeout=3",
                "agent-web",
                "--",
                "echo",
            ]
            .as_slice(),
            ["--format=json", "--timeout=0", "agent-web", "--", "echo"].as_slice(),
            ["--format=json", "--timeout=1201", "agent-web", "--", "echo"].as_slice(),
            ["--format=json", "--timeout=+2", "agent-web", "--", "echo"].as_slice(),
            ["--format=json", "--timeout=2", "agent-web", "echo"].as_slice(),
            ["--format=json", "--timeout=2", "agent/web", "--", "echo"].as_slice(),
            ["--format=json", "--timeout=2", "-agent-web", "--", "echo"].as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "agent-web",
                "-i",
                "--",
                "echo",
            ]
            .as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact direct exec JSON argv must fail before spawn: {values:?}"
            );
        }

        for forbidden in [
            "-i",
            "--interactive",
            "-t",
            "--tty",
            "-it",
            "-ti",
            "-d",
            "--detach",
            "-e",
            "--env",
            "--env-file",
            "-u",
            "--user",
            "-w",
            "--workdir",
            "--ulimit",
            "--privileged",
        ] {
            let values = vec![
                "--format=json".to_string(),
                "--timeout=2".to_string(),
                forbidden.to_string(),
                "agent-web".to_string(),
                "--".to_string(),
                "echo".to_string(),
            ];
            assert!(
                parse_docker_exec_json_args(&values).is_err(),
                "forbidden direct exec option must fail before spawn: {forbidden}"
            );
        }
    }

    #[test]
    fn docker_exec_json_dispatches_only_the_direct_aliases_and_preserves_raw_commands() {
        for values in [
            [
                "exec",
                "--format=json",
                "--timeout=17",
                "agent-web",
                "--",
                "fixture-command",
            ]
            .as_slice(),
            [
                "container",
                "exec",
                "--timeout",
                "17",
                "--format",
                "json",
                "agent-web",
                "--",
                "fixture-command",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_exec_json_request_from_argv(&args(values))
                    .expect("decode documented Docker exec JSON argv")
                    .is_some(),
                "documented exec JSON alias must use bounded VAT capture: {values:?}"
            );
        }

        for values in [
            ["exec", "agent-web", "fixture-command"].as_slice(),
            [
                "exec",
                "agent-web",
                "--",
                "fixture-command",
                "--format=json",
            ]
            .as_slice(),
            [
                "container",
                "exec",
                "agent-web",
                "--",
                "fixture-command",
                "--format",
                "json",
            ]
            .as_slice(),
            [
                "image",
                "exec",
                "--format=json",
                "--timeout=2",
                "agent-web",
                "--",
                "echo",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_exec_json_request_from_argv(&args(values))
                    .expect("decode raw or out-of-scope Docker exec argv")
                    .is_none(),
                "only a pre-container direct JSON selector may widen bounded capture: {values:?}"
            );
        }

        for values in [
            ["exec", "agent-web", "fixture-command"].as_slice(),
            ["container", "exec", "agent-web", "fixture-command"].as_slice(),
            [
                "exec",
                "agent-web",
                "--",
                "fixture-command",
                "--format=json",
            ]
            .as_slice(),
        ] {
            assert!(
                translate(&args(values)).is_ok(),
                "unformatted direct exec must retain the generic raw translation path: {values:?}"
            );
        }
    }

    #[test]
    fn docker_exec_json_strips_docker_only_separator_and_emits_one_vat_wrapper_schema() {
        let request = DockerExecJsonRequest {
            container_id: "agent-web".to_string(),
            timeout_seconds: 17,
            command: vec!["fixture-command".to_string(), "--literal".to_string()],
        };
        assert_eq!(
            apple_container_exec_json_argv(&request),
            [
                "container",
                "exec",
                "agent-web",
                "fixture-command",
                "--literal",
            ],
            "strict Docker exec JSON must strip the Docker-only literal separator before Apple argv"
        );

        let observation = DockerBoundedObservation {
            status: exit_status(43),
            stdout: DockerBoundedTextCapturedStream {
                text: "untrusted command output".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "backend diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let result = docker_exec_json_result(&request, &observation);
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker.exec.v1")
        );
        assert_eq!(result.get("format"), Some(&serde_json::json!("vat_json")));
        assert_eq!(
            result.get("container"),
            Some(&serde_json::json!("agent-web"))
        );
        assert_eq!(
            result.get("requested_timeout_seconds"),
            Some(&serde_json::json!(17))
        );
        assert_eq!(
            result.get("timeout_scope"),
            Some(&serde_json::json!("host-container-client-observation"))
        );
        assert_eq!(result.get("outcome"), Some(&serde_json::json!("failed")));
        assert_eq!(result.get("child_exit_code"), Some(&serde_json::json!(43)));
        assert_eq!(
            result.get("stdout"),
            Some(&serde_json::json!("untrusted command output"))
        );
        assert_eq!(
            result.get("stdout_truncated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("stdout_utf8_lossy"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("stderr"),
            Some(&serde_json::json!("backend diagnostic"))
        );
        assert_eq!(
            result.get("runtime_invoked"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("untrusted_command_output"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("secret_redaction_guaranteed"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            result.get("next"),
            Some(&serde_json::json!("docker inspect --format json agent-web"))
        );
    }

    #[test]
    fn docker_pull_json_accepts_only_the_direct_bounded_receipt_shape() {
        let parse = |values: &[&str]| {
            parse_docker_pull_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };
        let digest_reference = "registry.example/agent@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        assert_eq!(
            parse(
                [
                    "--timeout",
                    "17",
                    "--format",
                    "json",
                    "registry.example/agent:latest",
                ]
                .as_slice()
            )
            .expect("reordered direct pull JSON selectors"),
            DockerPullJsonRequest {
                image_reference: "registry.example/agent:latest".to_string(),
                timeout_seconds: 17,
            }
        );
        assert_eq!(
            parse(["--format=json", "--timeout=1200", digest_reference].as_slice())
                .expect("inline direct pull JSON selectors"),
            DockerPullJsonRequest {
                image_reference: digest_reference.to_string(),
                timeout_seconds: 1200,
            }
        );

        for values in [
            [].as_slice(),
            ["--format=json", "--timeout=2"].as_slice(),
            ["--format=table", "--timeout=2", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=0", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=1201", "agent:latest"].as_slice(),
            [
                "--format=json",
                "--format=json",
                "--timeout=2",
                "agent:latest",
            ]
            .as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--timeout=3",
                "agent:latest",
            ]
            .as_slice(),
            ["--format=json", "--timeout=2", "--all", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=2", "--", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=2", "-agent"].as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "https://registry.example/agent:latest",
            ]
            .as_slice(),
            ["--format=json", "--timeout=2", "git@registry.example:agent"].as_slice(),
            ["agent:latest", "--format=json", "--timeout=2"].as_slice(),
            ["--format=json", "agent:latest", "--timeout=2"].as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "agent:latest",
                "other:latest",
            ]
            .as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact pull receipt argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_pull_json_dispatches_only_direct_selector_forms_and_preserves_raw_pull() {
        for values in [
            ["pull", "--format=json", "--timeout=17", "agent:latest"].as_slice(),
            [
                "pull",
                "--timeout",
                "17",
                "--format",
                "json",
                "agent:latest",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_pull_json_request_from_argv(&args(values))
                    .expect("decode strict direct pull receipt argv")
                    .is_some(),
                "documented direct pull receipt must use bounded capture: {values:?}"
            );
        }
        for values in [
            ["pull", "agent:latest"].as_slice(),
            ["image", "pull", "agent:latest"].as_slice(),
            [
                "image",
                "pull",
                "--format=json",
                "--timeout=2",
                "agent:latest",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_pull_json_request_from_argv(&args(values))
                    .expect("leave raw or image-group pull outside strict direct parser")
                    .is_none(),
                "only direct docker pull may widen to bounded capture: {values:?}"
            );
        }
        assert!(
            docker_pull_json_request_from_argv(&args(&[
                "pull",
                "agent:latest",
                "--format=json",
                "--timeout=2",
            ]))
            .is_err(),
            "a direct selector after image must be claimed and rejected rather than reaching raw pull"
        );
        assert_eq!(
            translate(&args(&["pull", "alpine:3.20"])).expect("preserve raw pull translation"),
            ["container", "image", "pull", "alpine:3.20"],
            "unformatted direct pull must retain generic translation"
        );
        assert_eq!(
            translate(&args(&["image", "pull", "alpine:3.20"]))
                .expect("preserve raw image pull translation"),
            ["container", "image", "pull", "alpine:3.20"],
            "unformatted image-group pull must retain generic translation"
        );
    }

    #[test]
    fn docker_pull_json_strips_selectors_and_emits_bounded_receipts() {
        let request = DockerPullJsonRequest {
            image_reference: "agent's-image:latest".to_string(),
            timeout_seconds: 17,
        };
        assert_eq!(
            apple_container_pull_json_argv(&request),
            ["container", "image", "pull", "agent's-image:latest"],
            "strict pull receipt must strip Docker-only selectors and invoke only public Apple image pull argv"
        );

        let success = DockerBoundedObservation {
            status: exit_status(0),
            stdout: DockerBoundedTextCapturedStream {
                text: "untrusted pull output".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "backend diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let success_result = docker_pull_json_result(&request, &success);
        assert_eq!(success_result["schema"], "vat.docker.pull.v1");
        assert_eq!(success_result["format"], "vat_json");
        assert_eq!(success_result["image"], "agent's-image:latest");
        assert_eq!(success_result["requested_timeout_seconds"], 17);
        assert_eq!(
            success_result["timeout_scope"],
            "host-container-client-observation"
        );
        assert_eq!(success_result["stdout_truncated"], true);
        assert_eq!(success_result["stdout_utf8_lossy"], true);
        assert_eq!(
            success_result["image_lifecycle"],
            "not_owned_no_auto_cleanup"
        );
        assert_eq!(success_result["registry_management_implemented"], false);
        assert_eq!(success_result["image_state_verified"], false);
        assert_eq!(success_result["secret_redaction_guaranteed"], false);
        assert_eq!(success_result["cancellation_guaranteed"], false);
        assert_eq!(success_result["download_completion_guaranteed"], false);
        assert_eq!(success_result["rollback_guaranteed"], false);
        assert_eq!(
            success_result["next"], "docker image inspect --format json 'agent'\\''s-image:latest'",
            "success handoff must quote the image as one shell argv element"
        );

        let failure = DockerBoundedObservation {
            status: exit_status(41),
            stdout: DockerBoundedTextCapturedStream {
                text: "failed pull output".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "failed pull diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let failure_result = docker_pull_json_result(&request, &failure);
        assert_eq!(failure_result["outcome"], "failed");
        assert_eq!(failure_result["child_exit_code"], 41);
        assert_eq!(failure_result["terminal"], "pull_failed");
        assert_eq!(failure_result["next"], "docker --help");
    }

    #[test]
    fn docker_build_json_accepts_only_the_direct_bounded_receipt_shape() {
        let root = tempfile::tempdir().expect("build JSON tempdir");
        let context = root.path().join("context");
        std::fs::create_dir(&context).expect("create local build context");
        let context = context
            .to_str()
            .expect("UTF-8 local build context")
            .to_string();
        let canonical_context = std::fs::canonicalize(&context)
            .expect("canonicalize local build context")
            .to_str()
            .expect("UTF-8 canonical local build context")
            .to_string();

        let separated = vec![
            "--timeout".to_string(),
            "17".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--tag".to_string(),
            "registry.example/agent:latest".to_string(),
            "--file".to_string(),
            "Dockerfile.agent".to_string(),
            "--build-arg".to_string(),
            "MODE=test".to_string(),
            "--build-arg".to_string(),
            "TOKEN=-opaque".to_string(),
            "--target".to_string(),
            "release".to_string(),
            "--platform".to_string(),
            "linux/arm64".to_string(),
            "--label".to_string(),
            "io.cclab.vat.test=-opaque".to_string(),
            context.clone(),
        ];
        assert_eq!(
            parse_docker_build_json_args(&separated).expect("separated build JSON receipt"),
            DockerBuildJsonRequest {
                tag: "registry.example/agent:latest".to_string(),
                context: canonical_context.clone(),
                dockerfile: Some("Dockerfile.agent".to_string()),
                build_args: vec!["MODE=test".to_string(), "TOKEN=-opaque".to_string()],
                target: Some("release".to_string()),
                platform: Some("linux/arm64".to_string()),
                labels: vec!["io.cclab.vat.test=-opaque".to_string()],
                timeout_seconds: 17,
            }
        );

        let inline = vec![
            "--format=json".to_string(),
            "--timeout=1200".to_string(),
            "--tag=agent:latest".to_string(),
            "--file=Dockerfile".to_string(),
            "--build-arg=MODE=release".to_string(),
            "--target=final".to_string(),
            "--platform=linux/arm64".to_string(),
            "--label=io.cclab.vat.test=inline".to_string(),
            context.clone(),
        ];
        let request = parse_docker_build_json_args(&inline).expect("inline build JSON receipt");
        assert_eq!(request.tag, "agent:latest");
        assert_eq!(request.context, canonical_context);
        assert_eq!(request.dockerfile.as_deref(), Some("Dockerfile"));
        assert_eq!(request.build_args, ["MODE=release"]);
        assert_eq!(request.target.as_deref(), Some("final"));
        assert_eq!(request.platform.as_deref(), Some("linux/arm64"));
        assert_eq!(request.labels, ["io.cclab.vat.test=inline"]);
        assert_eq!(request.timeout_seconds, 1200);

        let file_context = root.path().join("not-a-directory");
        std::fs::write(&file_context, "not a directory").expect("write file context fixture");
        let file_context = file_context
            .to_str()
            .expect("UTF-8 file context fixture")
            .to_string();
        let invalid_cases = vec![
            vec!["--format=json", "--timeout=2", "--tag=agent:latest"],
            vec![
                "--format=table",
                "--timeout=2",
                "--tag=agent:latest",
                &context,
            ],
            vec![
                "--format=json",
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=0",
                "--tag=agent:latest",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=1201",
                "--tag=agent:latest",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--timeout=3",
                "--tag=agent:latest",
                &context,
            ],
            vec!["--format=json", "--timeout=2", &context],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:one",
                "--tag=agent:two",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--file=-",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--file",
                "--help",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--file=--help",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--build-arg=MODE",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--build-arg",
                "--TOKEN=opaque",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--label=--owner=opaque",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--label=owner",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--target",
                "--help",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--target=--help",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--platform",
                "--",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--platform=--help",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--progress=plain",
                &context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--",
                &context,
            ],
            vec!["--format=json", "--timeout=2", "--tag=agent:latest", "-"],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "--help",
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                "https://example.invalid/repo.git",
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                &file_context,
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                &context,
                "--label=late=x",
            ],
            vec![
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                &context,
                "extra-context",
            ],
        ];
        for values in invalid_cases {
            let values = values.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(
                parse_docker_build_json_args(&values).is_err(),
                "non-exact build receipt argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_build_json_dispatches_only_direct_selector_forms_and_preserves_raw_build() {
        let root = tempfile::tempdir().expect("build JSON dispatch tempdir");
        let context = root.path().join("context");
        std::fs::create_dir(&context).expect("create local build context");
        let context = context.to_str().expect("UTF-8 local build context");

        for values in [
            [
                "build",
                "--format=json",
                "--timeout=17",
                "--tag=agent:latest",
                context,
            ]
            .as_slice(),
            [
                "build",
                "--timeout",
                "17",
                "--format",
                "json",
                "--tag",
                "agent:latest",
                context,
            ]
            .as_slice(),
        ] {
            assert!(
                docker_build_json_request_from_argv(&args(values))
                    .expect("decode strict direct build receipt argv")
                    .is_some(),
                "documented direct build receipt must use bounded capture: {values:?}"
            );
        }

        for values in [
            ["build", "--tag", "agent:latest", context].as_slice(),
            ["build", "--target", "release", context].as_slice(),
            [
                "image",
                "build",
                "--format=json",
                "--timeout=2",
                "--tag=agent:latest",
                context,
            ]
            .as_slice(),
        ] {
            assert!(
                docker_build_json_request_from_argv(&args(values))
                    .expect("decode raw or out-of-scope Docker build argv")
                    .is_none(),
                "only direct build selectors may widen to bounded capture: {values:?}"
            );
        }

        assert_eq!(
            translate(&args(&["build", "-t", "agent:latest", context]))
                .expect("preserve pre-existing raw build translation"),
            ["container", "build", "-t", "agent:latest", context],
            "unformatted generic build must retain its raw translation"
        );
    }

    #[test]
    fn docker_build_json_strips_selectors_and_emits_bounded_receipts() {
        let request = DockerBuildJsonRequest {
            tag: "agent's-image:latest".to_string(),
            context: "/private/tmp/vat-build-context".to_string(),
            dockerfile: Some("Dockerfile.agent".to_string()),
            build_args: vec!["MODE=test".to_string(), "TOKEN=opaque".to_string()],
            target: Some("release".to_string()),
            platform: Some("linux/arm64".to_string()),
            labels: vec!["io.cclab.vat.test=opaque".to_string()],
            timeout_seconds: 17,
        };
        assert_eq!(
            apple_container_build_json_argv(&request),
            [
                "container",
                "build",
                "--tag",
                "agent's-image:latest",
                "--file",
                "Dockerfile.agent",
                "--build-arg",
                "MODE=test",
                "--build-arg",
                "TOKEN=opaque",
                "--target",
                "release",
                "--platform",
                "linux/arm64",
                "--label",
                "io.cclab.vat.test=opaque",
                "/private/tmp/vat-build-context",
            ],
            "strict build receipt must strip Docker-only selectors and invoke only public Apple build argv"
        );

        let success = DockerBoundedObservation {
            status: exit_status(0),
            stdout: DockerBoundedTextCapturedStream {
                text: "untrusted build output".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "backend diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let success_result = docker_build_json_result(&request, &success);
        assert_eq!(success_result["schema"], "vat.docker.build.v1");
        assert_eq!(success_result["format"], "vat_json");
        assert_eq!(success_result["tag"], "agent's-image:latest");
        assert_eq!(success_result["context"], "/private/tmp/vat-build-context");
        assert_eq!(success_result["dockerfile"], "Dockerfile.agent");
        assert_eq!(success_result["requested_timeout_seconds"], 17);
        assert_eq!(
            success_result["timeout_scope"],
            "host-container-client-observation"
        );
        assert_eq!(success_result["stdout_truncated"], true);
        assert_eq!(success_result["stdout_utf8_lossy"], true);
        assert_eq!(
            success_result["image_lifecycle"],
            "retained_no_auto_cleanup"
        );
        assert_eq!(success_result["secret_redaction_guaranteed"], false);
        assert_eq!(success_result["cancellation_guaranteed"], false);
        assert_eq!(success_result["rollback_guaranteed"], false);
        assert_eq!(
            success_result["next"], "docker image inspect --format json 'agent'\\''s-image:latest'",
            "success handoff must quote the tag as one shell argv element"
        );

        let failure = DockerBoundedObservation {
            status: exit_status(42),
            stdout: DockerBoundedTextCapturedStream {
                text: "failed build output".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "failed build diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let failure_result = docker_build_json_result(&request, &failure);
        assert_eq!(failure_result["outcome"], "failed");
        assert_eq!(failure_result["child_exit_code"], 42);
        assert_eq!(failure_result["terminal"], "build_failed");
        assert_eq!(failure_result["next"], "docker --help");
    }

    #[test]
    fn docker_run_json_accepts_only_the_direct_bounded_ephemeral_shape() {
        let parse = |values: &[&str]| {
            parse_docker_run_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        assert_eq!(
            parse(
                [
                    "--timeout",
                    "17",
                    "--format",
                    "json",
                    "registry.example/agent:latest",
                    "fixture-command",
                    "--literal",
                ]
                .as_slice()
            )
            .expect("reordered direct run JSON selectors"),
            DockerRunJsonRequest {
                image: "registry.example/agent:latest".to_string(),
                timeout_seconds: 17,
                command: vec!["fixture-command".to_string(), "--literal".to_string()],
            }
        );
        assert_eq!(
            parse(["--format=json", "--timeout=1200", "agent:latest"].as_slice())
                .expect("inline direct run JSON selectors"),
            DockerRunJsonRequest {
                image: "agent:latest".to_string(),
                timeout_seconds: 1200,
                command: Vec::new(),
            }
        );

        for values in [
            [].as_slice(),
            ["--format=json", "--timeout=2"].as_slice(),
            ["--format=table", "--timeout=2", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=0", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=1201", "agent:latest"].as_slice(),
            ["--format=json", "--timeout=2", "--detach", "agent:latest"].as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--name",
                "caller",
                "agent:latest",
            ]
            .as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--label",
                "caller=x",
                "agent:latest",
            ]
            .as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--publish",
                "8080:80",
                "agent:latest",
            ]
            .as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "--env",
                "A=B",
                "agent:latest",
            ]
            .as_slice(),
            ["--format=json", "--timeout=2", "--", "agent:latest"].as_slice(),
            [
                "--format=json",
                "--timeout=2",
                "agent:latest",
                "--",
                "fixture-command",
            ]
            .as_slice(),
            ["--format=json", "--timeout=2", "two words"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact direct run JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_run_json_dispatches_only_direct_selector_before_image_and_wraps_cleaned_result() {
        for values in [
            [
                "run",
                "--format=json",
                "--timeout=17",
                "agent:latest",
                "fixture-command",
            ]
            .as_slice(),
            ["run", "--timeout", "17", "--format", "json", "agent:latest"].as_slice(),
        ] {
            assert!(
                docker_run_json_request_from_argv(&args(values))
                    .expect("decode documented Docker run JSON argv")
                    .is_some(),
                "documented direct run JSON form must use bounded capture: {values:?}"
            );
        }
        for values in [
            ["run", "agent:latest", "fixture-command", "--format=json"].as_slice(),
            [
                "container",
                "run",
                "--format=json",
                "--timeout=17",
                "agent:latest",
            ]
            .as_slice(),
            ["create", "--format=json", "--timeout=17", "agent:latest"].as_slice(),
        ] {
            assert!(
                docker_run_json_request_from_argv(&args(values))
                    .expect("decode raw or out-of-scope Docker run argv")
                    .is_none(),
                "only direct pre-image run selectors may widen bounded capture: {values:?}"
            );
        }

        let request = DockerRunJsonRequest {
            image: "agent:latest".to_string(),
            timeout_seconds: 17,
            command: vec!["fixture-command".to_string(), "--literal".to_string()],
        };
        let ownership = DockerRunJsonOwnership {
            name: "vat-docker-run-0123456789abcdef0123456789abcdef".to_string(),
            token: "vat-run-private-owner-token".to_string(),
        };
        assert_eq!(
            apple_container_run_json_argv(&request, &ownership),
            [
                "container",
                "run",
                "--name",
                "vat-docker-run-0123456789abcdef0123456789abcdef",
                "--label",
                "io.cclab.vat.docker-run-owner=vat-run-private-owner-token",
                "agent:latest",
                "fixture-command",
                "--literal",
            ],
            "strict run JSON must inject only its generated name/label before IMAGE"
        );
        let observation = DockerBoundedObservation {
            status: exit_status(43),
            stdout: DockerBoundedTextCapturedStream {
                text: "untrusted command output".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "backend diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let result = docker_run_json_result(&request, &ownership, &observation);
        assert_eq!(
            result.get("schema"),
            Some(&serde_json::json!("vat.docker.run.v1"))
        );
        assert_eq!(result.get("format"), Some(&serde_json::json!("vat_json")));
        assert_eq!(
            result.get("generated_container_name"),
            Some(&serde_json::json!(ownership.name))
        );
        assert_eq!(result.get("outcome"), Some(&serde_json::json!("failed")));
        assert_eq!(result.get("child_exit_code"), Some(&serde_json::json!(43)));
        assert_eq!(
            result.get("cleanup"),
            Some(&serde_json::json!("confirmed_absent"))
        );
        assert_eq!(
            result.get("terminal"),
            Some(&serde_json::json!("cleaned_up"))
        );
        assert!(
            !result.to_string().contains(&ownership.token),
            "the owner token must not leak into the agent wrapper"
        );
    }

    #[test]
    fn docker_run_json_accepts_only_the_exact_apple_not_found_cleanup_diagnostic() {
        let name = "vat-docker-run-0123456789abcdef0123456789abcdef";
        let not_found = DockerNativeJsonObservation {
            status: exit_status(1),
            stdout: DockerNativeJsonCapturedStream {
                bytes: Vec::new(),
                capped: false,
            },
            stderr: DockerNativeJsonCapturedStream {
                bytes: format!("Error: container not found: {name}\n").into_bytes(),
                capped: false,
            },
        };
        assert!(docker_run_inspect_reports_not_found(&not_found, name));

        for diagnostic in [
            format!("Error: container not found: other-{name}\n"),
            format!("Error: backend not found for {name}\n"),
            format!("Error: container not found: {name}\n"),
        ] {
            let observation = DockerNativeJsonObservation {
                status: exit_status(1),
                stdout: DockerNativeJsonCapturedStream {
                    bytes: Vec::new(),
                    capped: false,
                },
                stderr: DockerNativeJsonCapturedStream {
                    bytes: diagnostic.into_bytes(),
                    capped: true,
                },
            };
            assert!(
                !docker_run_inspect_reports_not_found(&observation, name),
                "truncated or unrelated inspect diagnostics must fail closed"
            );
        }
    }

    #[test]
    fn docker_logs_json_accepts_only_one_bounded_safe_snapshot_shape() {
        let parse = |values: &[&str]| {
            parse_docker_logs_json_args(
                &values
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>(),
            )
        };

        for values in [
            ["--format", "json", "--tail", "17", "agent-web"].as_slice(),
            ["--tail=17", "--format=json", "agent_web.2"].as_slice(),
        ] {
            assert_eq!(
                parse(values).expect("exact direct logs JSON snapshot"),
                DockerLogsJsonRequest {
                    container_id: if values.last() == Some(&"agent-web") {
                        "agent-web".to_string()
                    } else {
                        "agent_web.2".to_string()
                    },
                    tail_lines: 17,
                },
                "documented direct logs snapshot argv must parse: {values:?}"
            );
        }

        for values in [
            [].as_slice(),
            ["--format=json", "--tail=17"].as_slice(),
            ["--format=table", "--tail=17", "agent-web"].as_slice(),
            ["--format=json", "--tail=all", "agent-web"].as_slice(),
            ["--format=json", "--tail=0", "agent-web"].as_slice(),
            ["--format=json", "--tail=1001", "agent-web"].as_slice(),
            ["--format=json", "--tail=+17", "agent-web"].as_slice(),
            ["--format=json", "--tail=17", "--follow", "agent-web"].as_slice(),
            ["--boot", "--format=json", "--tail=17", "agent-web"].as_slice(),
            ["--format=json", "--tail=17", "agent-web", "agent-db"].as_slice(),
            ["agent-web", "--format=json", "--tail=17"].as_slice(),
            ["--format=json", "--tail=17", "agent-web", "--"].as_slice(),
            ["--format=json", "--tail=17", "agent/web"].as_slice(),
        ] {
            assert!(
                parse(values).is_err(),
                "non-exact direct logs JSON argv must fail before spawn: {values:?}"
            );
        }
    }

    #[test]
    fn docker_logs_json_dispatches_only_direct_and_container_logs_aliases() {
        for values in [
            ["logs", "--format=json", "--tail=17", "agent-web"].as_slice(),
            [
                "container",
                "logs",
                "--tail",
                "17",
                "--format",
                "json",
                "agent-web",
            ]
            .as_slice(),
        ] {
            assert!(
                docker_logs_json_request_from_argv(&args(values))
                    .expect("decode documented Docker logs JSON argv")
                    .is_some(),
                "documented logs JSON alias must use bounded VAT capture: {values:?}"
            );
        }

        for values in [
            ["logs", "agent-web"].as_slice(),
            ["logs", "--tail", "17", "agent-web"].as_slice(),
            ["container", "logs", "agent-web"].as_slice(),
            ["container", "exec", "--format=json", "agent-web", "true"].as_slice(),
            ["image", "logs", "--format=json", "--tail=17", "agent-web"].as_slice(),
        ] {
            assert!(
                docker_logs_json_request_from_argv(&args(values))
                    .expect("decode text or out-of-scope Docker argv")
                    .is_none(),
                "text or out-of-scope form must not widen to bounded logs capture: {values:?}"
            );
        }
    }

    #[test]
    fn docker_logs_json_uses_the_canonical_apple_logs_argv_and_wrapper_schema() {
        let request = DockerLogsJsonRequest {
            container_id: "agent-web".to_string(),
            tail_lines: 17,
        };
        assert_eq!(
            apple_container_logs_json_argv(&request),
            ["container", "logs", "-n", "17", "agent-web"],
            "strict Docker logs JSON must strip selectors and invoke only Apple's public logs argv"
        );

        let observation = DockerBoundedObservation {
            status: exit_status(43),
            stdout: DockerBoundedTextCapturedStream {
                text: "untrusted\u{1} logs".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: DockerBoundedTextCapturedStream {
                text: "backend diagnostic".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let result = docker_logs_json_result(&request, &observation);
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker.logs.v1")
        );
        assert_eq!(result.get("format"), Some(&serde_json::json!("vat_json")));
        assert_eq!(
            result.get("apple_container_stdio"),
            Some(&serde_json::json!("untrusted\u{1} logs"))
        );
        assert_eq!(
            result.get("apple_container_stdio_truncated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("apple_container_stdio_utf8_lossy"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("diagnostic_stderr"),
            Some(&serde_json::json!("backend diagnostic"))
        );
        assert_eq!(result.get("outcome"), Some(&serde_json::json!("failed")));
        assert_eq!(result.get("child_exit_code"), Some(&serde_json::json!(43)));
        assert_eq!(
            result.get("runtime_invoked"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("secret_redaction_guaranteed"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            result.get("untrusted_log_content"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("next"),
            Some(&serde_json::json!("docker inspect --format json agent-web"))
        );
        assert!(
            result.get("stdout").is_none() && result.get("stderr").is_none(),
            "direct Apple logs wrapper must not claim Docker stdout/stderr demultiplexing"
        );
    }

    #[test]
    fn docker_logs_json_capture_keeps_a_lossy_json_safe_suffix() {
        let lossy = capture_docker_bounded_text_stream_with_limits(
            Cursor::new(vec![b'a', 0xff, b'\n']),
            64,
            64,
        )
        .expect("capture invalid UTF-8 direct logs");
        assert_eq!(lossy.text, "a\u{FFFD}\n");
        assert!(lossy.utf8_lossy);
        assert!(!lossy.truncated);

        let control_bytes = vec![1_u8; 64];
        let capped =
            capture_docker_bounded_text_stream_with_limits(Cursor::new(control_bytes), 64, 32)
                .expect("capture control-byte suffix under serialized cap");
        assert!(capped.truncated);
        assert!(
            serde_json::to_vec(&capped.text)
                .expect("serialize capped direct logs string")
                .len()
                <= 32,
            "control-byte escaping must stay inside the advertised JSON string cap"
        );
    }

    #[test]
    fn docker_stats_capture_is_bounded_before_json_validation() {
        let valid = capture_docker_native_json_stream_with_limit(
            Cursor::new(b"{\"id\":\"agent-web\"}\n"),
            64,
        )
        .expect("capture small stats JSON");
        assert!(!valid.capped);
        serde_json::from_slice::<serde_json::Value>(&valid.bytes)
            .expect("captured small stats output stays valid JSON");

        let oversized =
            capture_docker_native_json_stream_with_limit(Cursor::new(vec![b'x'; 17]), 16)
                .expect("drain oversized stats output");
        assert!(oversized.capped);
        assert_eq!(oversized.bytes.len(), 16);
        assert!(
            serde_json::from_slice::<serde_json::Value>(&oversized.bytes).is_err(),
            "a capped stream must be rejected rather than replayed as JSON"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_stats_deadline_kills_and_reaps_the_direct_child() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_STATS_NATIVE_JSON,
        )
        .expect_err("hanging stats observation must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container stats observation timed out"),
            "timeout must describe the Apple Container observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_ps_json_uses_the_same_bounded_group_cleanup_with_a_list_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_PS_NATIVE_JSON,
        )
        .expect_err("hanging ps JSON inventory must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container list observation timed out"),
            "ps JSON must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "ps JSON timeout must not mislabel its native list observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_images_json_uses_the_same_bounded_group_cleanup_with_an_image_list_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_IMAGES_NATIVE_JSON,
        )
        .expect_err("hanging image JSON inventory must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container image list observation timed out"),
            "image JSON must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "image JSON timeout must not mislabel its native image-list observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_inspect_json_uses_the_same_bounded_group_cleanup_with_an_inspect_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_INSPECT_NATIVE_JSON,
        )
        .expect_err("hanging direct container inspect JSON must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container inspect observation timed out"),
            "inspect JSON must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "inspect JSON timeout must not mislabel its native inspect observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_image_inspect_json_uses_the_same_bounded_group_cleanup_with_an_image_inspect_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_IMAGE_INSPECT_NATIVE_JSON,
        )
        .expect_err("hanging direct image inspect JSON must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container image inspect observation timed out"),
            "image inspect JSON must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "image inspect JSON timeout must not mislabel its native observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_logs_json_uses_the_same_bounded_group_cleanup_with_a_logs_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_bounded_text_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_LOGS_JSON,
        )
        .expect_err("hanging direct logs JSON snapshot must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container logs observation timed out"),
            "logs JSON must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "logs JSON timeout must not mislabel its direct logs observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_build_json_uses_the_same_bounded_group_cleanup_with_a_build_label() {
        let mut command = Command::new("/bin/sleep");
        command.arg("5").stdout(Stdio::null()).stderr(Stdio::null());
        let error = capture_docker_bounded_text_command(
            &mut command,
            Duration::from_millis(20),
            DOCKER_BUILD_JSON,
        )
        .expect_err("hanging direct build receipt must time out");
        assert!(
            error
                .to_string()
                .contains("Apple Container build client observation timed out"),
            "build receipt must use the shared bounded cleanup with its own operation label: {error:#}"
        );
        assert!(
            !error.to_string().contains("stats observation"),
            "build receipt timeout must not mislabel its observation: {error:#}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn docker_stats_deadline_kills_a_pipe_owning_descendant_after_root_exit() {
        let temp = tempfile::tempdir().expect("stats timeout tempdir");
        let descendant_pid_path = temp.path().join("pipe-owner.pid");
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "sleep 5 & printf '%s' \"$!\" > \"$1\"",
            "vat-docker-stats-test",
            descendant_pid_path
                .to_str()
                .expect("UTF-8 descendant pid path"),
        ]);
        let started = Instant::now();
        let error = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(30),
            DOCKER_STATS_NATIVE_JSON,
        )
        .expect_err("root-exited pipe owner must still hit the shared deadline");
        assert!(
            error
                .to_string()
                .contains("Apple Container stats observation timed out"),
            "shared deadline must describe the Apple Container observation: {error:#}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "the reader join must not outlive its deadline: {:?}",
            started.elapsed()
        );
        let descendant_pid = std::fs::read_to_string(&descendant_pid_path)
            .expect("read pipe-owning descendant pid")
            .trim()
            .parse::<u32>()
            .expect("parse pipe-owning descendant pid");
        let cleanup_deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let result = unsafe { libc::kill(descendant_pid as libc::pid_t, 0) };
            if result != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                break;
            }
            if Instant::now() >= cleanup_deadline {
                panic!(
                    "pipe-owning descendant {descendant_pid} remained after the stats group timeout"
                );
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Re-exec helper for the escaped-pipe regression below. The direct root
    /// is a test binary in its own process group; this forked child starts a
    /// new session, keeps inherited stdout/stderr open, and outlives the root.
    /// The raw libc-only child path avoids post-fork Rust runtime work.
    #[cfg(unix)]
    #[test]
    fn docker_stats_escaped_pipe_holder_helper() {
        let Some(pid_path) = std::env::var_os("VAT_DOCKER_STATS_ESCAPED_PIPE_HOLDER_PID_PATH")
        else {
            return;
        };
        let release_path = PathBuf::from(
            std::env::var_os("VAT_DOCKER_STATS_ESCAPED_PIPE_HOLDER_RELEASE_PATH")
                .expect("escaped pipe-holder release path"),
        );
        let release_path = CString::new(release_path.as_os_str().as_bytes())
            .expect("NUL-free escaped pipe-holder release path");
        let mut ready_pipe = [-1_i32; 2];
        assert_eq!(
            unsafe { libc::pipe(ready_pipe.as_mut_ptr()) },
            0,
            "create escaped pipe-holder readiness pipe"
        );
        let child_pid = unsafe { libc::fork() };
        assert!(child_pid >= 0, "fork escaped pipe holder");
        if child_pid == 0 {
            unsafe {
                libc::close(ready_pipe[0]);
            }
            let marker = if unsafe { libc::setsid() } >= 0 {
                b"1"
            } else {
                b"0"
            };
            unsafe {
                libc::write(
                    ready_pipe[1],
                    marker.as_ptr().cast::<libc::c_void>(),
                    marker.len(),
                );
                libc::close(ready_pipe[1]);
            }
            if marker == b"1" {
                // The parent test owns this unique release path. Polling it
                // lets test cleanup stop this detached process without ever
                // signalling a numeric PID that could later be reused.
                for _ in 0..1_000 {
                    if unsafe { libc::access(release_path.as_ptr(), libc::F_OK) } == 0 {
                        unsafe {
                            libc::_exit(0);
                        }
                    }
                    unsafe {
                        libc::usleep(10_000);
                    }
                }
                unsafe {
                    libc::_exit(0);
                }
            }
            unsafe {
                libc::_exit(1);
            }
        }

        unsafe {
            libc::close(ready_pipe[1]);
        }
        let mut marker = [0_u8; 1];
        let read = unsafe {
            libc::read(
                ready_pipe[0],
                marker.as_mut_ptr().cast::<libc::c_void>(),
                marker.len(),
            )
        };
        unsafe {
            libc::close(ready_pipe[0]);
        }
        assert_eq!(read, 1, "read escaped pipe-holder readiness marker");
        assert_eq!(marker, *b"1", "escaped pipe holder must create a session");
        std::fs::write(pid_path, child_pid.to_string()).expect("record escaped pipe-holder pid");
    }

    #[cfg(unix)]
    #[test]
    fn docker_stats_timeout_detaches_an_escaped_pipe_holder_without_reusing_its_pgid() {
        let temp = tempfile::tempdir().expect("escaped stats timeout tempdir");
        let escaped_pid_path = temp.path().join("escaped-pipe-owner.pid");
        let escaped_release_path = temp.path().join("release-escaped-pipe-owner");
        let mut release = EscapedPipeHolderRelease::new(escaped_release_path.clone());
        let current_test_binary = std::env::current_exe().expect("current VAT unit-test binary");
        let mut command = Command::new(current_test_binary);
        command
            .args([
                "--exact",
                "docker_shim::tests::docker_stats_escaped_pipe_holder_helper",
                "--nocapture",
            ])
            .env(
                "VAT_DOCKER_STATS_ESCAPED_PIPE_HOLDER_PID_PATH",
                &escaped_pid_path,
            )
            .env(
                "VAT_DOCKER_STATS_ESCAPED_PIPE_HOLDER_RELEASE_PATH",
                &escaped_release_path,
            );

        let started = Instant::now();
        let result = capture_docker_native_json_command(
            &mut command,
            Duration::from_millis(30),
            DOCKER_STATS_NATIVE_JSON,
        );
        let escaped_pid = std::fs::read_to_string(&escaped_pid_path)
            .expect("read escaped pipe-holder pid")
            .trim()
            .parse::<u32>()
            .expect("parse escaped pipe-holder pid");
        let error = result.expect_err("escaped pipe holder must fail closed without a reader join");
        assert!(
            error
                .to_string()
                .contains("escaped pipe holder was not joined")
                || error.to_string().contains("capture readers were detached"),
            "escaped holder failure must disclose the detached reader: {error:#}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "escaped pipe holder must return after bounded cleanup: {:?}",
            started.elapsed()
        );
        assert_eq!(
            unsafe { libc::kill(escaped_pid as libc::pid_t, 0) },
            0,
            "the escaped holder must survive the original process-group KILL until test cleanup"
        );
        release.release();
        wait_for_escaped_pipe_holder_exit(escaped_pid);
    }

    #[cfg(unix)]
    struct EscapedPipeHolderRelease {
        path: PathBuf,
        released: bool,
    }

    #[cfg(unix)]
    impl EscapedPipeHolderRelease {
        fn new(path: PathBuf) -> Self {
            Self {
                path,
                released: false,
            }
        }

        fn release(&mut self) {
            if !self.released {
                std::fs::write(&self.path, b"release escaped pipe holder")
                    .expect("release escaped pipe holder");
                self.released = true;
            }
        }
    }

    #[cfg(unix)]
    impl Drop for EscapedPipeHolderRelease {
        fn drop(&mut self) {
            if !self.released {
                let _ = std::fs::write(&self.path, b"release escaped pipe holder");
            }
        }
    }

    #[cfg(unix)]
    fn wait_for_escaped_pipe_holder_exit(pid: u32) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
            if result != 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                return;
            }
            if Instant::now() >= deadline {
                panic!("escaped pipe holder {pid} did not exit after its test-owned release");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn rejects_dynamic_publish_and_engine_commands_before_spawn() {
        let publish = translate(&args(&["run", "-p", "80", "nginx:alpine"]))
            .expect_err("bare port must fail");
        assert!(publish.to_string().contains("explicit host port"));

        let engine = translate(&args(&["info"])).expect_err("Engine command must fail");
        assert!(engine
            .to_string()
            .contains("unsupported Docker command `info`"));
    }

    #[test]
    fn rejects_docker_only_network_modes_and_format_flags() {
        let network = translate(&args(&["run", "--network", "host", "nginx:alpine"]))
            .expect_err("host network must fail");
        assert!(network
            .to_string()
            .contains("unsupported Docker network `host`"));

        let format =
            translate(&args(&["ps", "--format", "{{.ID}}"])).expect_err("template must fail");
        assert!(format.to_string().contains("format/filter/template"));
    }

    #[test]
    fn rejects_apple_only_or_semantically_different_flags_before_spawn() {
        let cases: &[&[&str]] = &[
            &["stop", "--all"],
            &["kill", "-a"],
            &["rm", "--all"],
            &["container", "rm", "--all"],
            &["image", "prune"],
            &["network", "prune"],
            &["volume", "prune"],
            &["login", "--scheme", "http", "registry.example"],
            &["run", "-c", "1", "nginx:alpine"],
            &["run", "--no-dns", "nginx:alpine"],
            &["run", "--runtime", "runc", "nginx:alpine"],
            &["build", "--progress", "plain", "."],
        ];

        for docker in cases {
            assert!(
                translate(&args(docker)).is_err(),
                "Apple-only or mismatched Docker argv must fail closed: {docker:?}"
            );
        }
    }

    #[test]
    fn maps_docker_stop_timeout_to_the_apple_container_flag() {
        assert_eq!(
            translate(&args(&["stop", "-t", "2", "demo"])).expect("translate stop timeout"),
            ["container", "stop", "--time", "2", "demo"]
        );
        assert_eq!(
            translate(&args(&["stop", "--timeout=2", "demo"]))
                .expect("translate inline stop timeout"),
            ["container", "stop", "--time", "2", "demo"]
        );
    }

    #[test]
    fn compose_up_result_discloses_independent_host_facing_contract() {
        let result = compose_result(
            "up",
            "local-tools",
            Some("docker compose -p local-tools ps".to_string()),
            None,
            None,
            None,
            Some(crate::compose::DockerComposeProfile::HostFacingIndependentV1),
            None,
        );
        assert_eq!(
            result.get("profile").and_then(serde_json::Value::as_str),
            Some("host-facing-independent-v1")
        );
        assert_eq!(
            result
                .get("service_name_dns")
                .and_then(serde_json::Value::as_bool),
            Some(false)
        );
        assert_eq!(
            result
                .get("host_loopback_only")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn compose_ps_result_serializes_typed_topology_without_unproven_endpoints() {
        let topology = crate::commands::compose::DockerShimTopologySnapshot {
            phase: crate::commands::compose::DockerShimTopologyPhase::Degraded,
            ready: false,
            services: vec![crate::commands::compose::DockerShimTopologyService {
                name: "docs".to_string(),
                state: crate::commands::compose::DockerShimTopologyServiceState::Ready,
                endpoint: None,
            }],
        };
        let result = compose_result(
            "ps",
            "local-tools",
            None,
            Some("observed"),
            None,
            None,
            Some(crate::compose::DockerComposeProfile::HostFacingIndependentV1),
            Some(&topology),
        );

        assert_eq!(
            result.get("profile").and_then(serde_json::Value::as_str),
            Some("host-facing-independent-v1")
        );
        assert_eq!(
            result.get("topology"),
            Some(&serde_json::json!({
                "phase": "degraded",
                "ready": false,
                "services": [{
                    "name": "docs",
                    "state": "ready",
                }],
            }))
        );
        assert!(
            result["topology"]["services"][0].get("endpoint").is_none(),
            "a degraded typed snapshot must omit, not null-fill, an unproven endpoint"
        );
    }

    #[test]
    fn compose_ps_json_result_marks_its_vat_schema_and_keeps_degraded_endpoints_hidden() {
        let snapshot = crate::commands::compose::DockerShimPsSnapshot {
            profile: crate::compose::DockerComposeProfile::HostFacingIndependentV1,
            topology: crate::commands::compose::DockerShimTopologySnapshot {
                phase: crate::commands::compose::DockerShimTopologyPhase::Degraded,
                ready: false,
                services: vec![crate::commands::compose::DockerShimTopologyService {
                    name: "docs".to_string(),
                    state: crate::commands::compose::DockerShimTopologyServiceState::Ready,
                    endpoint: None,
                }],
            },
        };
        let result = compose_ps_json_result("local-tools", &snapshot);
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker-compose.ps.v1")
        );
        assert_eq!(
            result.get("format").and_then(serde_json::Value::as_str),
            Some("vat_json")
        );
        assert_eq!(
            result["topology"],
            serde_json::json!({
                "phase": "degraded",
                "ready": false,
                "services": [{
                    "name": "docs",
                    "state": "ready",
                }],
            })
        );
        assert!(
            result["topology"]["services"][0].get("endpoint").is_none(),
            "VAT-native JSON must preserve all-or-none endpoint proof"
        );
    }

    #[test]
    fn compose_logs_json_result_is_bounded_capture_only_and_not_a_topology_claim() {
        let snapshot = crate::commands::compose::DockerShimLogSnapshot {
            profile: crate::compose::DockerComposeProfile::HostFacingIndependentV1,
            stdout: crate::commands::compose::DockerShimLogStreamSnapshot {
                text: "stdout-two\nstdout-three".to_string(),
                truncated: true,
                utf8_lossy: false,
            },
            stderr: crate::commands::compose::DockerShimLogStreamSnapshot {
                text: "stderr-three".to_string(),
                truncated: false,
                utf8_lossy: true,
            },
        };
        let result = compose_logs_json_result("agent-tools", "docs", 2, &snapshot);
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker-compose.logs.v1")
        );
        assert_eq!(
            result.get("format").and_then(serde_json::Value::as_str),
            Some("vat_json")
        );
        assert_eq!(result.get("service"), Some(&serde_json::json!("docs")));
        assert_eq!(result.get("tail_lines"), Some(&serde_json::json!(2)));
        assert_eq!(
            result.get("stdout"),
            Some(&serde_json::json!("stdout-two\nstdout-three"))
        );
        assert_eq!(
            result.get("stdout_truncated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("stderr_utf8_lossy"),
            Some(&serde_json::json!(true))
        );
        for key in ["capture_only", "runtime_invoked", "compose_record_mutated"] {
            assert_eq!(
                result.get(key),
                Some(&serde_json::Value::Bool(key == "capture_only")),
                "capture contract field `{key}`"
            );
        }
        assert!(
            result.get("topology").is_none(),
            "reading captured logs cannot claim lifecycle topology or endpoints"
        );
        assert_eq!(
            result.get("next").and_then(serde_json::Value::as_str),
            Some("docker compose -p agent-tools ps --format json")
        );
    }

    #[test]
    fn compose_exec_json_result_is_one_bounded_nonzero_child_document() {
        let snapshot = crate::commands::compose::DockerShimExecSnapshot {
            profile: crate::compose::DockerComposeProfile::StrictSingleImageV1,
            status: exit_status(23),
            stdout: crate::commands::compose::DockerShimLogStreamSnapshot {
                text: "agent stdout".to_string(),
                truncated: true,
                utf8_lossy: true,
            },
            stderr: crate::commands::compose::DockerShimLogStreamSnapshot {
                text: "agent stderr".to_string(),
                truncated: false,
                utf8_lossy: false,
            },
        };
        let result = compose_exec_json_result("agent-tools", "web", &snapshot);
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker-compose.exec.v1")
        );
        assert_eq!(
            result.get("format").and_then(serde_json::Value::as_str),
            Some("vat_json")
        );
        assert_eq!(result.get("service"), Some(&serde_json::json!("web")));
        assert_eq!(result.get("outcome"), Some(&serde_json::json!("failed")));
        assert_eq!(result.get("child_exit_code"), Some(&serde_json::json!(23)));
        assert_eq!(
            result.get("stdout"),
            Some(&serde_json::json!("agent stdout"))
        );
        assert_eq!(
            result.get("stderr"),
            Some(&serde_json::json!("agent stderr"))
        );
        assert_eq!(
            result.get("stdout_truncated"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            result.get("stdout_utf8_lossy"),
            Some(&serde_json::json!(true))
        );
        for key in ["runtime_invoked", "compose_record_mutated"] {
            assert_eq!(
                result.get(key),
                Some(&serde_json::Value::Bool(key == "runtime_invoked")),
                "exec JSON contract field `{key}`"
            );
        }
        assert_eq!(
            result.get("profile").and_then(serde_json::Value::as_str),
            Some("strict-single-image-v1")
        );
        assert_eq!(
            result.get("service_name_dns"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            result.get("host_loopback_only"),
            Some(&serde_json::json!(true))
        );
        assert!(
            result.get("topology").is_none(),
            "an exec result must not reopen lifecycle topology after spawning the child"
        );
    }

    #[test]
    fn compose_dry_run_result_is_nonstarting_and_returns_a_shell_safe_revalidation_next() {
        let result = compose_dry_run_result(
            "agent-tools",
            Path::new("/tmp/agent's project/compose.yml"),
            crate::compose::DockerComposeProfile::HostFacingIndependentV1,
            true,
        );
        assert_eq!(
            result.get("schema").and_then(serde_json::Value::as_str),
            Some("vat.docker-compose.preflight.v1")
        );
        assert_eq!(result.get("dry_run"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(result.get("build"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            result.get("validated"),
            Some(&serde_json::Value::Bool(true))
        );
        for key in [
            "runtime_started",
            "registry_written",
            "image_built",
            "launch_revalidates",
        ] {
            assert_eq!(
                result.get(key),
                Some(&serde_json::Value::Bool(key == "launch_revalidates")),
                "preflight contract field `{key}`"
            );
        }
        assert_eq!(
            result.get("launch_argv"),
            Some(&serde_json::json!([
                "docker",
                "compose",
                "-f",
                "/tmp/agent's project/compose.yml",
                "-p",
                "agent-tools",
                "up",
                "-d",
                "--build",
            ]))
        );
        assert_eq!(
            result.get("next").and_then(serde_json::Value::as_str),
            Some("'docker' 'compose' '-f' '/tmp/agent'\\''s project/compose.yml' '-p' 'agent-tools' 'up' '-d' '--build'")
        );
        assert_eq!(
            result.get("service_name_dns"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            result.get("host_loopback_only"),
            Some(&serde_json::Value::Bool(true))
        );
        assert!(
            result.get("topology").is_none(),
            "a preflight must not invent lifecycle topology or endpoints"
        );
    }

    #[test]
    fn bounded_wait_fails_closed_for_degraded_and_replaced_generations() {
        let degraded = crate::commands::compose::DockerShimPsSnapshot {
            profile: crate::compose::DockerComposeProfile::HostFacingIndependentV1,
            topology: crate::commands::compose::DockerShimTopologySnapshot {
                phase: crate::commands::compose::DockerShimTopologyPhase::Degraded,
                ready: false,
                services: vec![crate::commands::compose::DockerShimTopologyService {
                    name: "web".to_string(),
                    state: crate::commands::compose::DockerShimTopologyServiceState::Ready,
                    endpoint: None,
                }],
            },
        };
        let degraded_result = wait_for_docker_shim_compose_ready_with(
            "agent-web",
            Instant::now() + Duration::from_secs(1),
            || Ok(crate::commands::compose::DockerShimWaitObservation::Degraded(degraded.clone())),
        )
        .expect("degraded observation is structured, not an internal error");
        assert_eq!(degraded_result.outcome, "degraded");
        assert_eq!(
            degraded_result
                .topology
                .as_ref()
                .and_then(|topology| topology.services[0].endpoint.as_ref()),
            None,
            "degraded wait must never expose a partial endpoint"
        );
        assert_eq!(
            degraded_result.next.as_deref(),
            Some("docker compose -p agent-web ps")
        );

        let replaced_result = wait_for_docker_shim_compose_ready_with(
            "agent-web",
            Instant::now() + Duration::from_secs(1),
            || {
                Ok(
                    crate::commands::compose::DockerShimWaitObservation::LifecycleReplaced(
                        "generation/ticket no longer matches".to_string(),
                    ),
                )
            },
        )
        .expect("replacement is a structured terminal result");
        assert_eq!(replaced_result.outcome, "lifecycle_replaced");
        assert!(replaced_result.topology.is_none());
        assert!(
            replaced_result.next.is_none(),
            "a replacement may have cleared shim provenance, so no stale next command is valid"
        );
    }

    #[test]
    fn bounded_wait_rejects_ready_observed_after_deadline_without_endpoint() {
        let ready = crate::commands::compose::DockerShimPsSnapshot {
            profile: crate::compose::DockerComposeProfile::StrictSingleImageV1,
            topology: crate::commands::compose::DockerShimTopologySnapshot {
                phase: crate::commands::compose::DockerShimTopologyPhase::Ready,
                ready: true,
                services: vec![crate::commands::compose::DockerShimTopologyService {
                    name: "web".to_string(),
                    state: crate::commands::compose::DockerShimTopologyServiceState::Ready,
                    endpoint: Some("127.0.0.1:18080".to_string()),
                }],
            },
        };
        let result = wait_for_docker_shim_compose_ready_with(
            "agent-web",
            Instant::now() + Duration::from_millis(1),
            || {
                std::thread::sleep(Duration::from_millis(2));
                Ok(crate::commands::compose::DockerShimWaitObservation::Ready(
                    ready.clone(),
                ))
            },
        )
        .expect("late ready is a structured timeout");
        assert_eq!(result.outcome, "timeout");
        assert!(result.topology.is_none());
    }

    #[test]
    fn parses_only_the_strict_compose_lifecycle_profile() {
        let up = parse_docker_compose_command(&[
            "-f".to_string(),
            "compose.yml".to_string(),
            "-p".to_string(),
            "agent-web".to_string(),
            "up".to_string(),
            "-d".to_string(),
        ])
        .expect("strict compose up");
        assert_eq!(
            up,
            DockerComposeCommand::Up {
                file: PathBuf::from("compose.yml"),
                project: "agent-web".to_string(),
                options: DockerComposeUpOptions {
                    build: false,
                    wait_timeout_seconds: None,
                },
            }
        );
        assert_eq!(
            parse_docker_compose_command(&[
                "-f".to_string(),
                "compose.yml".to_string(),
                "-p".to_string(),
                "agent-web".to_string(),
                "up".to_string(),
                "--build".to_string(),
                "-d".to_string(),
            ])
            .expect("strict compose source build up"),
            DockerComposeCommand::Up {
                file: PathBuf::from("compose.yml"),
                project: "agent-web".to_string(),
                options: DockerComposeUpOptions {
                    build: true,
                    wait_timeout_seconds: None,
                },
            }
        );
        assert_eq!(
            parse_docker_compose_command(&[
                "-p".to_string(),
                "agent-web".to_string(),
                "logs".to_string(),
                "web".to_string(),
            ])
            .expect("strict compose logs"),
            DockerComposeCommand::Logs {
                project: "agent-web".to_string(),
                service: "web".to_string(),
                format: DockerComposeLogsFormat::Text,
            }
        );
        assert_eq!(
            parse_docker_compose_command(&[
                "-p".to_string(),
                "agent-web".to_string(),
                "exec".to_string(),
                "-T".to_string(),
                "web".to_string(),
                "--".to_string(),
                "sh".to_string(),
                "-ec".to_string(),
                "printf compose-exec".to_string(),
            ])
            .expect("strict compose exec"),
            DockerComposeCommand::Exec {
                project: "agent-web".to_string(),
                service: "web".to_string(),
                command: vec![
                    "sh".to_string(),
                    "-ec".to_string(),
                    "printf compose-exec".to_string(),
                ],
                format: DockerComposeExecFormat::Text,
            }
        );

        for args in [
            vec!["up", "-d"],
            vec!["-f", "compose.yml", "-p", "Agent", "up", "-d"],
            vec!["-f", "compose.yml", "-p", "agent", "up"],
            vec!["-f", "compose.yml", "-p", "agent", "up", "-d", "--pull"],
            vec!["-f", "compose.yml", "-p", "agent", "up", "-d", "-d"],
            vec!["-p", "agent", "ps", "web"],
            vec!["-p", "agent", "logs", "--follow", "web"],
            vec!["-p", "agent", "exec", "web", "sh", "-ec", "true"],
            vec![
                "-p", "agent", "exec", "--no-tty", "web", "sh", "-ec", "true",
            ],
            vec!["-p", "agent", "exec", "-T", "web"],
            vec!["-p", "agent", "exec", "-T", "--", "sh", "-ec", "true"],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "exec",
                "-T",
                "web",
                "sh",
            ],
            vec!["-p", "agent", "down", "-v"],
            vec!["--env-file", ".env", "-p", "agent", "up", "-d"],
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(
                parse_docker_compose_command(&args).is_err(),
                "unsupported compose argv must fail closed: {args:?}"
            );
        }
    }

    #[test]
    fn shim_help_advertises_the_exact_agent_json_exec_inventory_and_logs_forms() {
        assert!(
            STRICT_COMPOSE_HELP.contains("exec -T --format json SERVICE -- COMMAND"),
            "help must advertise the strict noninteractive JSON exec argv"
        );
        assert!(
            STRICT_COMPOSE_HELP.contains("--format=json"),
            "help must disclose the accepted equals spelling without implying generic formats"
        );
        assert!(
            STRICT_COMPOSE_HELP.contains("Exact JSON ps, logs, and exec forms"),
            "help must retain the VAT-native schema boundary"
        );
        assert!(
            STRICT_STATS_HELP.contains("docker stats --no-stream --format json CONTAINER"),
            "help must advertise the only supported non-streaming stats argv"
        );
        assert!(
            STRICT_STATS_HELP.contains("Apple Container JSON document unchanged"),
            "help must avoid claiming a VAT or Docker stats output schema"
        );
        assert!(
            STRICT_PS_HELP.contains("docker ps --format json"),
            "help must advertise the exact direct Apple-native inventory form"
        );
        assert!(
            STRICT_PS_HELP.contains("docker container ls")
                && STRICT_PS_HELP.contains("docker container list"),
            "help must name only the documented container list aliases"
        );
        assert!(
            STRICT_PS_HELP.contains(
                "not Docker Engine schema, ownership, health, readiness, or liveness proof"
            ),
            "help must keep inventory claims below Docker Engine and readiness semantics"
        );
        assert!(
            STRICT_IMAGES_HELP.contains("docker images --format json"),
            "help must advertise the exact direct Apple-native image inventory form"
        );
        assert!(
            STRICT_IMAGES_HELP.contains("docker image ls")
                && STRICT_IMAGES_HELP.contains("docker image list"),
            "help must name only the documented image-list aliases"
        );
        assert!(
            STRICT_IMAGES_HELP.contains(
                "not Docker Engine image schema, ownership, provenance, security, executability, registry, build-readiness, health, readiness, or liveness proof"
            ),
            "help must keep image inventory claims below Docker Engine and image readiness semantics"
        );
        assert!(
            STRICT_INSPECT_HELP.contains("docker inspect --format json CONTAINER"),
            "help must advertise the exact direct Apple-native container inspect form"
        );
        assert!(
            STRICT_INSPECT_HELP.contains("docker container inspect"),
            "help must name only the documented direct container inspect alias"
        );
        assert!(
            STRICT_INSPECT_HELP.contains(
                "not Docker Engine inspect schema; ownership, provenance, security, image identity, registry, or build-status proof; health/readiness/liveness/port-reachability proof; or a secret-redaction guarantee"
            ),
            "help must retain all inspect safety and non-claim boundaries"
        );
        assert!(
            STRICT_LOGS_HELP.contains("docker logs --format json --tail LINES CONTAINER")
                && STRICT_LOGS_HELP.contains("docker container logs")
                && STRICT_LOGS_HELP.contains("container logs -n LINES CONTAINER"),
            "help must advertise the strict direct logs snapshot grammar and Apple argv"
        );
        assert!(
            STRICT_LOGS_HELP.contains("vat.docker.logs.v1")
                && STRICT_LOGS_HELP.contains("apple_container_stdio")
                && STRICT_LOGS_HELP.contains("untrusted content")
                && STRICT_LOGS_HELP.contains("secret-redaction proof"),
            "help must disclose the VAT wrapper, untrusted content, and non-claim boundaries"
        );
        assert!(
            STRICT_EXEC_HELP
                .contains("docker exec --format json --timeout SECONDS CONTAINER -- COMMAND")
                && STRICT_EXEC_HELP.contains("docker container exec")
                && STRICT_EXEC_HELP.contains("stripped after validation")
                && STRICT_EXEC_HELP.contains("container exec CONTAINER COMMAND"),
            "help must advertise the strict direct exec JSON grammar and Apple argv"
        );
        assert!(
            STRICT_EXEC_HELP.contains("vat.docker.exec.v1")
                && STRICT_EXEC_HELP.contains("host Apple Container client observation")
                && STRICT_EXEC_HELP.contains("does not claim to terminate a guest process")
                && STRICT_EXEC_HELP.contains("secret-redaction guarantee"),
            "help must disclose the direct exec wrapper and bounded host-client non-claims"
        );
    }

    #[test]
    fn compose_exec_json_is_one_exact_noninteractive_agent_mode() {
        let parse = |args: &[&str]| {
            let args = args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>();
            parse_docker_compose_command(&args)
        };
        for args in [
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format",
                "json",
                "web",
                "--",
                "sh",
                "-ec",
                "printf json-exec",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format=json",
                "web",
                "--",
                "sh",
                "-ec",
                "printf json-exec",
            ]
            .as_slice(),
        ] {
            assert_eq!(
                parse(args).expect("exact VAT-native compose exec JSON"),
                DockerComposeCommand::Exec {
                    project: "agent-web".to_string(),
                    service: "web".to_string(),
                    command: vec![
                        "sh".to_string(),
                        "-ec".to_string(),
                        "printf json-exec".to_string(),
                    ],
                    format: DockerComposeExecFormat::VatJson,
                },
                "compose argv: {args:?}"
            );
        }
        for args in [
            ["-p", "agent-web", "exec", "-T", "--format"].as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format",
                "table",
                "web",
                "--",
                "true",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format=json",
                "--format=json",
                "web",
                "--",
                "true",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "--format",
                "json",
                "-T",
                "web",
                "--",
                "true",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "--no-tty",
                "--format",
                "json",
                "web",
                "--",
                "true",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format",
                "json",
                "web",
                "true",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format=json",
                "web",
                "--",
            ]
            .as_slice(),
            [
                "-f",
                "compose.yml",
                "-p",
                "agent-web",
                "exec",
                "-T",
                "--format",
                "json",
                "web",
                "--",
                "true",
            ]
            .as_slice(),
        ] {
            assert!(
                parse(args).is_err(),
                "unsupported/misordered compose exec JSON argv must fail before spawn: {args:?}"
            );
        }
    }

    #[test]
    fn compose_ps_json_is_one_explicit_vat_native_mode() {
        let parse = |args: &[&str]| {
            let args = args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>();
            parse_docker_compose_command(&args)
        };
        assert_eq!(
            parse(&["-p", "agent-web", "ps"]).expect("text compose ps"),
            DockerComposeCommand::Ps {
                project: "agent-web".to_string(),
                format: DockerComposePsFormat::Text,
            }
        );
        for args in [
            ["-p", "agent-web", "ps", "--format", "json"].as_slice(),
            ["-p", "agent-web", "ps", "--format=json"].as_slice(),
        ] {
            assert_eq!(
                parse(args).expect("VAT-native compose ps JSON"),
                DockerComposeCommand::Ps {
                    project: "agent-web".to_string(),
                    format: DockerComposePsFormat::VatJson,
                },
                "compose argv: {args:?}"
            );
        }
        for args in [
            ["-p", "agent-web", "ps", "--format"].as_slice(),
            ["-p", "agent-web", "ps", "--format", "table"].as_slice(),
            ["-p", "agent-web", "ps", "--format={{.Name}}"].as_slice(),
            ["-p", "agent-web", "ps", "--format", "json", "--format=json"].as_slice(),
            ["-p", "agent-web", "ps", "web"].as_slice(),
            [
                "-f",
                "compose.yml",
                "-p",
                "agent-web",
                "ps",
                "--format",
                "json",
            ]
            .as_slice(),
        ] {
            assert!(
                parse(args).is_err(),
                "unsupported compose ps argv must fail before observation: {args:?}"
            );
        }
    }

    #[test]
    fn compose_logs_json_is_an_explicit_bounded_vat_snapshot() {
        let parse = |args: &[&str]| {
            let args = args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>();
            parse_docker_compose_command(&args)
        };
        assert_eq!(
            parse(&["-p", "agent-web", "logs", "web"]).expect("text compose logs"),
            DockerComposeCommand::Logs {
                project: "agent-web".to_string(),
                service: "web".to_string(),
                format: DockerComposeLogsFormat::Text,
            }
        );
        for args in [
            ["-p", "agent-web", "logs", "--format", "json", "web"].as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--tail",
                "3",
                "--format=json",
                "web",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--format=json",
                "--tail=3",
                "web",
            ]
            .as_slice(),
        ] {
            let expected_tail = if args.iter().any(|arg| *arg == "3" || *arg == "--tail=3") {
                3
            } else {
                crate::commands::compose::DEFAULT_DOCKER_SHIM_LOG_TAIL_LINES
            };
            assert_eq!(
                parse(args).expect("VAT-native compose log snapshot"),
                DockerComposeCommand::Logs {
                    project: "agent-web".to_string(),
                    service: "web".to_string(),
                    format: DockerComposeLogsFormat::VatJson {
                        tail_lines: expected_tail,
                    },
                },
                "compose argv: {args:?}"
            );
        }
        for args in [
            ["-p", "agent-web", "logs"].as_slice(),
            ["-p", "agent-web", "logs", "--format"].as_slice(),
            ["-p", "agent-web", "logs", "--format", "table", "web"].as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--tail",
                "0",
                "--format",
                "json",
                "web",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--tail",
                "1001",
                "--format",
                "json",
                "web",
            ]
            .as_slice(),
            ["-p", "agent-web", "logs", "--tail", "2", "web"].as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--follow",
                "--format",
                "json",
                "web",
            ]
            .as_slice(),
            [
                "-p",
                "agent-web",
                "logs",
                "--format",
                "json",
                "web",
                "extra",
            ]
            .as_slice(),
            [
                "-f",
                "compose.yml",
                "-p",
                "agent-web",
                "logs",
                "--format",
                "json",
                "web",
            ]
            .as_slice(),
        ] {
            assert!(
                parse(args).is_err(),
                "unsupported compose logs argv must fail before observation: {args:?}"
            );
        }
    }

    #[test]
    fn compose_dry_run_is_a_strict_nonstarting_up_preflight() {
        let parse = |args: &[&str]| {
            let args = args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>();
            parse_docker_compose_command(&args)
        };
        assert_eq!(
            parse(&[
                "--dry-run",
                "-f",
                "compose.yml",
                "-p",
                "agent-web",
                "up",
                "-d",
            ])
            .expect("strict image dry run"),
            DockerComposeCommand::DryRunUp {
                file: PathBuf::from("compose.yml"),
                project: "agent-web".to_string(),
                build: false,
            }
        );
        assert_eq!(
            parse(&[
                "-f",
                "compose.build.yml",
                "--dry-run",
                "-p",
                "agent-build",
                "up",
                "--build",
                "-d",
            ])
            .expect("strict source-build dry run"),
            DockerComposeCommand::DryRunUp {
                file: PathBuf::from("compose.build.yml"),
                project: "agent-build".to_string(),
                build: true,
            }
        );
        for args in [
            [
                "--dry-run",
                "--dry-run",
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
            ]
            .as_slice(),
            ["--dry-run", "-p", "agent", "up", "-d"].as_slice(),
            ["--dry-run", "-f", "compose.yml", "-p", "agent", "up"].as_slice(),
            [
                "--dry-run",
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait",
            ]
            .as_slice(),
            ["--dry-run", "-p", "agent", "ps"].as_slice(),
            ["-f", "compose.yml", "-p", "agent", "up", "--dry-run", "-d"].as_slice(),
        ] {
            assert!(
                parse(args).is_err(),
                "unsupported dry-run argv must fail before observation: {args:?}"
            );
        }
    }

    #[test]
    fn compose_wait_requires_explicit_detach_and_a_bounded_positive_timeout() {
        let wait = parse_docker_compose_command(&[
            "-f".to_string(),
            "compose.yml".to_string(),
            "-p".to_string(),
            "agent-web".to_string(),
            "up".to_string(),
            "-d".to_string(),
            "--wait".to_string(),
        ])
        .expect("default bounded wait");
        assert_eq!(
            wait,
            DockerComposeCommand::Up {
                file: PathBuf::from("compose.yml"),
                project: "agent-web".to_string(),
                options: DockerComposeUpOptions {
                    build: false,
                    wait_timeout_seconds: Some(DEFAULT_COMPOSE_WAIT_TIMEOUT_SECONDS),
                },
            }
        );

        let explicit = parse_docker_compose_command(&[
            "-f".to_string(),
            "compose.yml".to_string(),
            "-p".to_string(),
            "agent-web".to_string(),
            "up".to_string(),
            "--wait-timeout=7".to_string(),
            "--wait".to_string(),
            "-d".to_string(),
            "--build".to_string(),
        ])
        .expect("inline bounded wait timeout");
        assert_eq!(
            explicit,
            DockerComposeCommand::Up {
                file: PathBuf::from("compose.yml"),
                project: "agent-web".to_string(),
                options: DockerComposeUpOptions {
                    build: true,
                    wait_timeout_seconds: Some(7),
                },
            }
        );

        for args in [
            vec!["-f", "compose.yml", "-p", "agent", "up", "--wait"],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait-timeout",
                "7",
            ],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait",
                "--wait",
            ],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait",
                "--wait-timeout",
                "0",
            ],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait",
                "--wait-timeout=1201",
            ],
            vec!["-f", "compose.yml", "-p", "agent", "up", "-d", "--wait=1"],
            vec![
                "-f",
                "compose.yml",
                "-p",
                "agent",
                "up",
                "-d",
                "--wait",
                "--wait-timeout",
                "4",
                "--wait-timeout=5",
            ],
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(
                parse_docker_compose_command(&args).is_err(),
                "unsupported wait syntax must fail closed: {args:?}"
            );
        }
    }

    #[test]
    fn preserves_normal_child_exit_codes() {
        assert_eq!(exit_code(exit_status(0)), ExitCode::SUCCESS);
        assert_eq!(exit_code(exit_status(42)), ExitCode::from(42));
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }
}
// HANDWRITE-END
