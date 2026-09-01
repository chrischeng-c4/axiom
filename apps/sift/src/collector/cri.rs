// HANDWRITE-BEGIN gap="missing-generator:logic:25c76bad" tracker="1675" reason="Own safe CRI discovery, envelope/partial framing, device-inode rotation, multi-source checkpoint, metadata, and loss accounting."
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use service_collector::CollectorSource as _;

use crate::AttributeValue;

use super::checkpoint::QuarantineEntry;
use super::source::{
    read_bounded_line, CommitStats, RawRecord, ReadOutcome, RecordEnrichment, SourceCursor,
    SourceRejection,
};
use super::CriSourceConfig;

const CRI_CHECKPOINT_SCHEMA: &str = "collector.cri.checkpoint.v1";
const MAX_WORKLOAD_ID_BYTES: usize = 253;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CriCheckpoint {
    schema: String,
    root: String,
    files: BTreeMap<String, CriFileCheckpoint>,
    accepted: u64,
    duplicates: u64,
    rejected: u64,
    lost_bytes: u64,
    lost_sources: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CriFileCheckpoint {
    offset: u64,
    line: u64,
    observed_len: u64,
    relative_path: String,
    workload: WorkloadIdentity,
    retired: bool,
    loss_reported: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct WorkloadIdentity {
    namespace: String,
    pod: String,
    pod_uid: String,
    container: String,
    restart: u32,
}

#[derive(Clone, Debug)]
struct DiscoveredFile {
    identity: String,
    path: PathBuf,
    relative_path: String,
    len: u64,
    workload: WorkloadIdentity,
    known_before: bool,
}

pub(crate) struct CriSource {
    config: CriSourceConfig,
    root: PathBuf,
    checkpoint_path: PathBuf,
    checkpoint: CriCheckpoint,
    discovered: Vec<DiscoveredFile>,
    read_positions: HashMap<String, (u64, u64)>,
    pending_losses: VecDeque<SourceRejection>,
    staged_losses: HashSet<String>,
    start_offset: u64,
}

impl CriSource {
    pub(crate) fn open(config: CriSourceConfig, checkpoint_path: PathBuf) -> Result<Self> {
        let root = std::fs::canonicalize(&config.root)
            .with_context(|| format!("resolve CRI root {}", config.root.display()))?;
        if !std::fs::metadata(&root)?.is_dir() {
            bail!("collector CRI root must be a directory");
        }
        let root_text = root.to_string_lossy().to_string();
        let checkpoint = CriCheckpoint::load(&checkpoint_path, &root_text)?;
        let start_offset = checkpoint.files.values().map(|file| file.offset).sum();
        let read_positions = checkpoint
            .files
            .iter()
            .map(|(identity, file)| (identity.clone(), (file.offset, file.line)))
            .collect();
        let mut source = Self {
            config,
            root,
            checkpoint_path,
            checkpoint,
            discovered: Vec::new(),
            read_positions,
            pending_losses: VecDeque::new(),
            staged_losses: HashSet::new(),
            start_offset,
        };
        source.refresh()?;
        Ok(source)
    }

    fn read_file(&mut self, discovered: &DiscoveredFile, max_bytes: usize) -> Result<ReadOutcome> {
        let (start_offset, start_line) = self
            .read_positions
            .get(&discovered.identity)
            .copied()
            .unwrap_or((0, 0));
        if start_offset >= discovered.len {
            return Ok(ReadOutcome::Exhausted);
        }

        let mut file = File::open(&discovered.path)
            .with_context(|| format!("open CRI source {}", discovered.path.display()))?;
        file.seek(SeekFrom::Start(start_offset))
            .with_context(|| format!("seek CRI source {}", discovered.path.display()))?;
        let mut reader = BufReader::new(file);
        let first = read_bounded_line(&mut reader, max_bytes)?;
        if first.bytes_read == 0 {
            return Ok(ReadOutcome::Exhausted);
        }

        let mut next_offset = start_offset
            .checked_add(first.bytes_read)
            .context("CRI byte offset overflow")?;
        let mut next_line = start_line.checked_add(1).context("CRI line overflow")?;
        if first.oversized {
            return Ok(self.rejection(
                discovered,
                start_offset,
                next_offset,
                next_line,
                "cri_line_too_large",
                format!("CRI physical record exceeds {max_bytes} bytes"),
                &first.preview,
            ));
        }
        let first_frame = match parse_cri_frame(&first.preview) {
            Ok(frame) => frame,
            Err(error) => {
                return Ok(self.rejection(
                    discovered,
                    start_offset,
                    next_offset,
                    next_line,
                    "invalid_cri_envelope",
                    error.to_string(),
                    &first.preview,
                ));
            }
        };

        let stream = first_frame.stream;
        let mut content = first_frame.content;
        let mut preview = first.preview;
        let mut tag = first_frame.tag;
        let mut oversized = content.len() > max_bytes;

        while tag == CriTag::Partial {
            let fragment = read_bounded_line(&mut reader, max_bytes)?;
            if fragment.bytes_read == 0 {
                return Ok(ReadOutcome::Pending);
            }
            next_offset = next_offset
                .checked_add(fragment.bytes_read)
                .context("CRI byte offset overflow")?;
            next_line = next_line.checked_add(1).context("CRI line overflow")?;
            if preview.len() < 1024 {
                preview.extend_from_slice(
                    &fragment.preview[..fragment.preview.len().min(1024 - preview.len())],
                );
            }
            if fragment.oversized {
                oversized = true;
            }
            let frame = match parse_cri_frame(&fragment.preview) {
                Ok(frame) => frame,
                Err(error) => {
                    return Ok(self.rejection(
                        discovered,
                        start_offset,
                        next_offset,
                        next_line,
                        "invalid_cri_partial",
                        error.to_string(),
                        &preview,
                    ));
                }
            };
            if frame.stream != stream {
                return Ok(self.rejection(
                    discovered,
                    start_offset,
                    next_offset,
                    next_line,
                    "interleaved_cri_partial",
                    "CRI partial fragments must remain on one stream",
                    &preview,
                ));
            }
            if content.len().saturating_add(frame.content.len()) > max_bytes {
                oversized = true;
            } else if !oversized {
                content.extend_from_slice(&frame.content);
            }
            tag = frame.tag;
        }

        if oversized {
            return Ok(self.rejection(
                discovered,
                start_offset,
                next_offset,
                next_line,
                "cri_record_too_large",
                format!("assembled CRI record exceeds {max_bytes} bytes"),
                &preview,
            ));
        }

        self.read_positions
            .insert(discovered.identity.clone(), (next_offset, next_line));
        Ok(ReadOutcome::Record(RawRecord {
            source_id: format!("cri:{}", discovered.identity),
            line: start_line + 1,
            offset: start_offset,
            bytes: content,
            cursor: SourceCursor::Cri {
                identity: discovered.identity.clone(),
                next_offset,
                next_line,
                observed_len: discovered.len,
            },
            enrichment: enrichment(&self.config, discovered, stream),
        }))
    }

    fn rejection(
        &mut self,
        discovered: &DiscoveredFile,
        start_offset: u64,
        next_offset: u64,
        next_line: u64,
        code: &str,
        message: impl AsRef<str>,
        preview: &[u8],
    ) -> ReadOutcome {
        self.read_positions
            .insert(discovered.identity.clone(), (next_offset, next_line));
        ReadOutcome::Rejection(SourceRejection {
            entry: QuarantineEntry::invalid_line(
                &format!("cri:{}", discovered.identity),
                next_line,
                start_offset,
                code,
                message,
                preview,
            ),
            cursor: SourceCursor::Cri {
                identity: discovered.identity.clone(),
                next_offset,
                next_line,
                observed_len: discovered.len,
            },
        })
    }
}

impl service_collector::CollectorSource for CriSource {
    type Cursor = SourceCursor;
    type Error = anyhow::Error;
    type Record = RawRecord;
    type Rejection = SourceRejection;

    fn next_record(&mut self, max_bytes: usize) -> Result<ReadOutcome> {
        if let Some(loss) = self.pending_losses.pop_front() {
            if let SourceCursor::CriLoss { identity, .. } = &loss.cursor {
                self.staged_losses.insert(identity.clone());
            }
            return Ok(ReadOutcome::Rejection(loss));
        }
        let mut saw_pending = false;
        for discovered in self.discovered.clone() {
            match self.read_file(&discovered, max_bytes)? {
                ReadOutcome::Exhausted => {}
                ReadOutcome::Pending => saw_pending = true,
                outcome => return Ok(outcome),
            }
        }
        Ok(if saw_pending {
            ReadOutcome::Pending
        } else {
            ReadOutcome::Exhausted
        })
    }

    fn commit(&mut self, cursors: &[SourceCursor], stats: CommitStats) -> Result<()> {
        for cursor in cursors {
            match cursor {
                SourceCursor::Cri {
                    identity,
                    next_offset,
                    next_line,
                    observed_len,
                } => {
                    let file =
                        self.checkpoint.files.get_mut(identity).with_context(|| {
                            format!("missing CRI checkpoint identity {identity}")
                        })?;
                    file.offset = *next_offset;
                    file.line = *next_line;
                    file.observed_len = file.observed_len.max(*observed_len);
                }
                SourceCursor::CriLoss {
                    identity,
                    lost_bytes,
                } => {
                    let file = self
                        .checkpoint
                        .files
                        .get_mut(identity)
                        .with_context(|| format!("missing lost CRI identity {identity}"))?;
                    if !file.loss_reported {
                        file.retired = true;
                        file.loss_reported = true;
                        self.checkpoint.lost_bytes = self
                            .checkpoint
                            .lost_bytes
                            .checked_add(*lost_bytes)
                            .context("CRI lost byte counter overflow")?;
                        self.checkpoint.lost_sources = self
                            .checkpoint
                            .lost_sources
                            .checked_add(1)
                            .context("CRI lost source counter overflow")?;
                    }
                    self.staged_losses.remove(identity);
                }
                SourceCursor::Linear { .. } => bail!("linear cursor committed to CRI source"),
            }
        }
        self.checkpoint.accepted = self.checkpoint.accepted.saturating_add(stats.accepted);
        self.checkpoint.duplicates = self.checkpoint.duplicates.saturating_add(stats.duplicates);
        self.checkpoint.rejected = self.checkpoint.rejected.saturating_add(stats.rejected);
        self.checkpoint.save(&self.checkpoint_path)
    }

    fn refresh(&mut self) -> Result<()> {
        let mut discovered = discover(&self.root, &self.checkpoint.files)?;
        let present: HashSet<_> = discovered
            .iter()
            .map(|file| file.identity.clone())
            .collect();

        for file in &discovered {
            let entry = self
                .checkpoint
                .files
                .entry(file.identity.clone())
                .or_insert_with(|| CriFileCheckpoint {
                    offset: 0,
                    line: 0,
                    observed_len: file.len,
                    relative_path: file.relative_path.clone(),
                    workload: file.workload.clone(),
                    retired: false,
                    loss_reported: false,
                });
            entry.observed_len = entry.observed_len.max(file.len);
            entry.relative_path = file.relative_path.clone();
            entry.workload = file.workload.clone();
            entry.retired = false;
            self.read_positions
                .entry(file.identity.clone())
                .or_insert((entry.offset, entry.line));
        }

        for (identity, file) in &mut self.checkpoint.files {
            if present.contains(identity) || file.retired || self.staged_losses.contains(identity) {
                continue;
            }
            let lost_bytes = file.observed_len.saturating_sub(file.offset);
            if lost_bytes == 0 {
                file.retired = true;
                continue;
            }
            self.pending_losses.push_back(SourceRejection {
                entry: QuarantineEntry::invalid_line(
                    &format!("cri:{identity}"),
                    file.line,
                    file.offset,
                    "source_lost",
                    format!(
                        "CRI source disappeared with {lost_bytes} observed uncommitted bytes at {}",
                        file.relative_path
                    ),
                    &[],
                ),
                cursor: SourceCursor::CriLoss {
                    identity: identity.clone(),
                    lost_bytes,
                },
            });
        }

        discovered.sort_by(|left, right| {
            (!left.known_before, &left.relative_path, &left.identity).cmp(&(
                !right.known_before,
                &right.relative_path,
                &right.identity,
            ))
        });
        self.discovered = discovered;
        self.checkpoint.save(&self.checkpoint_path)
    }

    fn progress(&self) -> service_collector::SourceProgress {
        service_collector::SourceProgress {
            start_offset: self.start_offset,
            final_offset: self.checkpoint.files.values().map(|file| file.offset).sum(),
            lost_bytes: self.checkpoint.lost_bytes,
            lost_sources: self.checkpoint.lost_sources,
        }
    }
}

impl CriCheckpoint {
    fn new(root: &str) -> Self {
        Self {
            schema: CRI_CHECKPOINT_SCHEMA.to_string(),
            root: root.to_string(),
            files: BTreeMap::new(),
            accepted: 0,
            duplicates: 0,
            rejected: 0,
            lost_bytes: 0,
            lost_sources: 0,
        }
    }

    fn load(path: &Path, root: &str) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new(root));
        }
        let checkpoint: Self = service_collector::load_json_checkpoint(path)?
            .context("CRI checkpoint disappeared while loading")?;
        if checkpoint.schema != CRI_CHECKPOINT_SCHEMA {
            bail!(
                "unsupported CRI checkpoint schema {}; expected {CRI_CHECKPOINT_SCHEMA}",
                checkpoint.schema
            );
        }
        if checkpoint.root != root {
            bail!(
                "CRI checkpoint root mismatch: stored {}, configured {root}",
                checkpoint.root
            );
        }
        Ok(checkpoint)
    }

    fn save(&self, path: &Path) -> Result<()> {
        service_collector::save_json_checkpoint(path, self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CriStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CriTag {
    Full,
    Partial,
}

struct CriFrame {
    stream: CriStream,
    tag: CriTag,
    content: Vec<u8>,
}

fn parse_cri_frame(line: &[u8]) -> Result<CriFrame> {
    let line = trim_line_ending(line);
    let mut spaces = line
        .iter()
        .enumerate()
        .filter_map(|(index, byte)| (*byte == b' ').then_some(index));
    let first = spaces
        .next()
        .context("CRI record is missing timestamp delimiter")?;
    let second = spaces
        .next()
        .context("CRI record is missing stream delimiter")?;
    let third = spaces
        .next()
        .context("CRI record is missing tag delimiter")?;
    let timestamp = std::str::from_utf8(&line[..first]).context("CRI timestamp must be UTF-8")?;
    DateTime::parse_from_rfc3339(timestamp).context("CRI timestamp must be RFC3339")?;
    let stream = match &line[first + 1..second] {
        b"stdout" => CriStream::Stdout,
        b"stderr" => CriStream::Stderr,
        _ => bail!("CRI stream must be stdout or stderr"),
    };
    let tag = match &line[second + 1..third] {
        b"F" => CriTag::Full,
        b"P" => CriTag::Partial,
        _ => bail!("CRI tag must be F or P"),
    };
    Ok(CriFrame {
        stream,
        tag,
        content: line[third + 1..].to_vec(),
    })
}

fn trim_line_ending(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\n")
        .unwrap_or(line)
        .strip_suffix(b"\r")
        .unwrap_or_else(|| line.strip_suffix(b"\n").unwrap_or(line))
}

fn enrichment(
    config: &CriSourceConfig,
    file: &DiscoveredFile,
    stream: CriStream,
) -> RecordEnrichment {
    let mut resource = BTreeMap::from([
        ("gcp.resource.type".to_string(), "k8s_container".to_string()),
        (
            "gcp.project_id".to_string(),
            config.metadata.gcp_project.clone(),
        ),
        (
            "gcp.resource.label.project_id".to_string(),
            config.metadata.gcp_project.clone(),
        ),
        (
            "gcp.resource.label.namespace_name".to_string(),
            file.workload.namespace.clone(),
        ),
        (
            "gcp.resource.label.pod_name".to_string(),
            file.workload.pod.clone(),
        ),
        (
            "gcp.resource.label.container_name".to_string(),
            file.workload.container.clone(),
        ),
        (
            "k8s.namespace.name".to_string(),
            file.workload.namespace.clone(),
        ),
        ("k8s.pod.name".to_string(), file.workload.pod.clone()),
        ("k8s.pod.uid".to_string(), file.workload.pod_uid.clone()),
        (
            "k8s.container.name".to_string(),
            file.workload.container.clone(),
        ),
    ]);
    if let Some(cluster) = &config.metadata.cluster {
        resource.insert("k8s.cluster.name".to_string(), cluster.clone());
        resource.insert(
            "gcp.resource.label.cluster_name".to_string(),
            cluster.clone(),
        );
    }
    if let Some(location) = &config.metadata.location {
        resource.insert("cloud.region".to_string(), location.clone());
        resource.insert("gcp.resource.label.location".to_string(), location.clone());
    }
    if let Some(node) = &config.metadata.node {
        resource.insert("k8s.node.name".to_string(), node.clone());
    }
    RecordEnrichment {
        resource,
        attributes: BTreeMap::from([
            (
                "collector.stream".to_string(),
                AttributeValue::String(
                    match stream {
                        CriStream::Stdout => "stdout",
                        CriStream::Stderr => "stderr",
                    }
                    .to_string(),
                ),
            ),
            (
                "k8s.container.restart_count".to_string(),
                AttributeValue::Int(i64::from(file.workload.restart)),
            ),
        ]),
        cloud_logging_coexistence: true,
    }
}

fn discover(
    root: &Path,
    known: &BTreeMap<String, CriFileCheckpoint>,
) -> Result<Vec<DiscoveredFile>> {
    let mut paths = Vec::new();
    visit_regular_files(root, root, 0, &mut paths)?;
    let mut files = Vec::new();
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .context("CRI discovery escaped canonical root")?;
        let Some(workload) = parse_workload_path(relative)? else {
            continue;
        };
        let metadata = std::fs::symlink_metadata(&path)?;
        #[cfg(unix)]
        let identity = {
            use std::os::unix::fs::MetadataExt;
            format!("{}:{}", metadata.dev(), metadata.ino())
        };
        #[cfg(not(unix))]
        bail!("CRI device/inode collection requires Unix");
        let relative_path = relative.to_string_lossy().to_string();
        files.push(DiscoveredFile {
            known_before: known.contains_key(&identity),
            identity,
            path,
            relative_path,
            len: metadata.len(),
            workload,
        });
    }
    Ok(files)
}

fn visit_regular_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    output: &mut Vec<PathBuf>,
) -> Result<()> {
    if depth > 3 {
        return Ok(());
    }
    for entry in std::fs::read_dir(directory)
        .with_context(|| format!("read CRI directory {}", directory.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            visit_regular_files(root, &path, depth + 1, output)?;
        } else if file_type.is_file() && path.starts_with(root) {
            output.push(path);
        }
    }
    Ok(())
}

fn parse_workload_path(relative: &Path) -> Result<Option<WorkloadIdentity>> {
    let components = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if components.len() != 3 {
        return Ok(None);
    }
    let mut pod = components[0].splitn(3, '_');
    let Some(namespace) = pod.next() else {
        return Ok(None);
    };
    let Some(pod_name) = pod.next() else {
        return Ok(None);
    };
    let Some(pod_uid) = pod.next() else {
        return Ok(None);
    };
    let container = &components[1];
    let Some(restart_text) = components[2].split(".log").next() else {
        return Ok(None);
    };
    if !components[2].contains(".log") || restart_text.is_empty() {
        return Ok(None);
    }
    for (name, value) in [
        ("namespace", namespace),
        ("pod", pod_name),
        ("pod uid", pod_uid),
        ("container", container.as_str()),
    ] {
        validate_workload_id(name, value)?;
    }
    let restart = restart_text
        .parse::<u32>()
        .context("CRI restart index must be u32")?;
    Ok(Some(WorkloadIdentity {
        namespace: namespace.to_string(),
        pod: pod_name.to_string(),
        pod_uid: pod_uid.to_string(),
        container: container.clone(),
        restart,
    }))
}

fn validate_workload_id(name: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > MAX_WORKLOAD_ID_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        bail!("invalid CRI {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_and_partial_cri_frames_without_touching_json() {
        let full = parse_cri_frame(b"2026-07-17T10:00:00.123456789Z stdout F {\"x\":1}\n").unwrap();
        assert_eq!(full.stream, CriStream::Stdout);
        assert_eq!(full.tag, CriTag::Full);
        assert_eq!(full.content, br#"{"x":1}"#);

        let partial = parse_cri_frame(b"2026-07-17T10:00:00Z stderr P {\"x\"").unwrap();
        assert_eq!(partial.stream, CriStream::Stderr);
        assert_eq!(partial.tag, CriTag::Partial);
        assert_eq!(partial.content, br#"{"x""#);
        assert!(parse_cri_frame(b"bad stdout F {}\n").is_err());
    }

    #[test]
    fn parses_standard_and_rotated_pod_log_paths() {
        let identity = parse_workload_path(Path::new(
            "prod_checkout-7_1234-abcd/lumen/0.log.20260717-100000",
        ))
        .unwrap()
        .unwrap();
        assert_eq!(identity.namespace, "prod");
        assert_eq!(identity.pod, "checkout-7");
        assert_eq!(identity.pod_uid, "1234-abcd");
        assert_eq!(identity.container, "lumen");
        assert_eq!(identity.restart, 0);
        assert!(parse_workload_path(Path::new("escape.log"))
            .unwrap()
            .is_none());
    }
}

// HANDWRITE-END
