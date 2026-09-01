//! Shared, rebuildable text-index contract.
//!
//! Products own their records and query language. This crate owns schema
//! validation, text analysis, version ordering, snapshots, and rebuilds.

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    sync::RwLock,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const SNAPSHOT_FORMAT_VERSION: u32 = 1;
pub const DEFAULT_NGRAM_MIN: usize = 2;
pub const DEFAULT_NGRAM_MAX: usize = 3;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Analyzer {
    WhitespaceLower,
    Jieba,
    Ngram,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldKind {
    Text { analyzer: Analyzer },
    Keyword,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FieldSpec {
    pub kind: FieldKind,
}

impl FieldSpec {
    pub fn text(analyzer: Analyzer) -> Self {
        Self {
            kind: FieldKind::Text { analyzer },
        }
    }

    pub fn keyword() -> Self {
        Self {
            kind: FieldKind::Keyword,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TextSchema {
    fields: BTreeMap<String, FieldSpec>,
}

impl TextSchema {
    pub fn new(fields: BTreeMap<String, FieldSpec>) -> Result<Self> {
        if fields.is_empty() {
            return Err(IndexError::InvalidSchema {
                message: "text index needs at least one field".to_string(),
            });
        }
        if let Some(name) = fields
            .keys()
            .find(|name| name.trim().is_empty() || name.contains('\0'))
        {
            return Err(IndexError::InvalidSchema {
                message: format!("invalid field name {name:?}"),
            });
        }
        Ok(Self { fields })
    }

    pub fn fields(&self) -> &BTreeMap<String, FieldSpec> {
        &self.fields
    }

    fn field(&self, name: &str) -> Result<&FieldSpec> {
        self.fields
            .get(name)
            .ok_or_else(|| IndexError::UnknownField {
                field: name.to_string(),
            })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TextDocument {
    pub external_id: String,
    pub version: u64,
    pub fields: BTreeMap<String, String>,
}

impl TextDocument {
    pub fn new(external_id: impl Into<String>, version: u64) -> Self {
        Self {
            external_id: external_id.into(),
            version,
            fields: BTreeMap::new(),
        }
    }

    pub fn with_field(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), value.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOperator {
    All,
    Any,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum TextQuery {
    All,
    Match {
        field: String,
        text: String,
        operator: MatchOperator,
    },
    Exact {
        field: String,
        value: String,
    },
    And {
        queries: Vec<TextQuery>,
    },
    Or {
        queries: Vec<TextQuery>,
    },
    Not {
        query: Box<TextQuery>,
    },
}

impl TextQuery {
    pub fn match_text(
        field: impl Into<String>,
        text: impl Into<String>,
        operator: MatchOperator,
    ) -> Self {
        Self::Match {
            field: field.into(),
            text: text.into(),
            operator,
        }
    }

    pub fn exact(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Exact {
            field: field.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TextHit {
    pub external_id: String,
    pub version: u64,
    pub score: f32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TextIndexSnapshot {
    pub format_version: u32,
    pub schema: TextSchema,
    pub documents: Vec<TextDocument>,
    /// Highest observed delete version for each absent document. Older version
    /// 1 snapshots omit this field and decode with an empty tombstone table.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tombstones: BTreeMap<String, u64>,
}

impl TextIndexSnapshot {
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|error| IndexError::CorruptSnapshot {
            message: error.to_string(),
        })
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|error| IndexError::CorruptSnapshot {
            message: error.to_string(),
        })
    }
}

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("invalid text-index schema: {message}")]
    InvalidSchema { message: String },
    #[error("unknown text-index field: {field}")]
    UnknownField { field: String },
    #[error("field {field} does not support {operation}")]
    UnsupportedFieldOperation {
        field: String,
        operation: &'static str,
    },
    #[error("invalid text-index document: {message}")]
    InvalidDocument { message: String },
    #[error("text-index snapshot is corrupt: {message}")]
    CorruptSnapshot { message: String },
    #[error("text-index lock is poisoned")]
    LockPoisoned,
}

pub type Result<T> = std::result::Result<T, IndexError>;

/// Product-neutral index boundary. The caller remains the source of truth.
pub trait TextIndex: Send + Sync {
    fn schema(&self) -> TextSchema;
    fn upsert(&self, document: TextDocument) -> Result<()>;
    fn delete(&self, external_id: &str, version: Option<u64>) -> Result<bool>;
    fn search(&self, query: &TextQuery, limit: usize) -> Result<Vec<TextHit>>;
    fn snapshot(&self) -> Result<TextIndexSnapshot>;
    fn restore(&self, snapshot: &TextIndexSnapshot) -> Result<()>;
    fn rebuild(&self, documents: Vec<TextDocument>) -> Result<()>;
}

/// Deterministic in-process index. Durability comes from its typed snapshot or
/// from rebuilding it from the product's committed segments.
pub struct MemoryTextIndex {
    schema: TextSchema,
    state: RwLock<MemoryTextState>,
}

#[derive(Default)]
struct MemoryTextState {
    documents: BTreeMap<String, TextDocument>,
    tombstones: BTreeMap<String, u64>,
}

impl MemoryTextIndex {
    pub fn new(schema: TextSchema) -> Result<Self> {
        // Re-run validation for deserialized schemas.
        let schema = TextSchema::new(schema.fields)?;
        Ok(Self {
            schema,
            state: RwLock::new(MemoryTextState::default()),
        })
    }

    fn validate_external_id(&self, external_id: &str) -> Result<()> {
        if external_id.trim().is_empty() || external_id.contains('\0') {
            return Err(IndexError::InvalidDocument {
                message: "external_id must not be empty or contain NUL".to_string(),
            });
        }
        Ok(())
    }

    fn validate_document(&self, document: &TextDocument) -> Result<()> {
        self.validate_external_id(&document.external_id)?;
        for field in document.fields.keys() {
            self.schema.field(field)?;
        }
        Ok(())
    }

    fn build_state(
        &self,
        documents: Vec<TextDocument>,
        tombstones: BTreeMap<String, u64>,
    ) -> Result<MemoryTextState> {
        let mut rebuilt = BTreeMap::<String, TextDocument>::new();
        for document in documents {
            self.validate_document(&document)?;
            if rebuilt
                .get(&document.external_id)
                .is_none_or(|current| current.version < document.version)
            {
                rebuilt.insert(document.external_id.clone(), document);
            }
        }

        let mut retained_tombstones = BTreeMap::new();
        for (external_id, delete_version) in tombstones {
            self.validate_external_id(&external_id)?;
            if rebuilt
                .get(&external_id)
                .is_some_and(|document| document.version > delete_version)
            {
                continue;
            }
            rebuilt.remove(&external_id);
            retained_tombstones.insert(external_id, delete_version);
        }
        Ok(MemoryTextState {
            documents: rebuilt,
            tombstones: retained_tombstones,
        })
    }

    fn validate_query(&self, query: &TextQuery) -> Result<()> {
        match query {
            TextQuery::All => Ok(()),
            TextQuery::Match { field, .. } => match self.schema.field(field)?.kind {
                FieldKind::Text { .. } => Ok(()),
                FieldKind::Keyword => Err(IndexError::UnsupportedFieldOperation {
                    field: field.clone(),
                    operation: "text match",
                }),
            },
            TextQuery::Exact { field, .. } => match self.schema.field(field)?.kind {
                FieldKind::Keyword => Ok(()),
                FieldKind::Text { .. } => Err(IndexError::UnsupportedFieldOperation {
                    field: field.clone(),
                    operation: "exact match",
                }),
            },
            TextQuery::And { queries } | TextQuery::Or { queries } => {
                for query in queries {
                    self.validate_query(query)?;
                }
                Ok(())
            }
            TextQuery::Not { query } => self.validate_query(query),
        }
    }

    fn evaluate(&self, document: &TextDocument, query: &TextQuery) -> Option<f32> {
        match query {
            TextQuery::All => Some(1.0),
            TextQuery::Match {
                field,
                text,
                operator,
            } => {
                let FieldKind::Text { analyzer } = self.schema.fields.get(field)?.kind else {
                    return None;
                };
                let query_terms = tokenize(text, analyzer);
                if query_terms.is_empty() {
                    return None;
                }
                let document_terms = document
                    .fields
                    .get(field)
                    .map(|value| tokenize(value, analyzer))
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<BTreeSet<_>>();
                let matched = query_terms
                    .iter()
                    .filter(|term| document_terms.contains(*term))
                    .count();
                let accepts = match operator {
                    MatchOperator::All => matched == query_terms.len(),
                    MatchOperator::Any => matched > 0,
                };
                accepts.then_some(matched as f32 / query_terms.len() as f32)
            }
            TextQuery::Exact { field, value } => document
                .fields
                .get(field)
                .is_some_and(|actual| actual == value)
                .then_some(1.0),
            TextQuery::And { queries } => {
                let mut score = 0.0;
                for query in queries {
                    score += self.evaluate(document, query)?;
                }
                Some(score.max(1.0))
            }
            TextQuery::Or { queries } => queries
                .iter()
                .filter_map(|query| self.evaluate(document, query))
                .reduce(f32::max),
            TextQuery::Not { query } => self.evaluate(document, query).is_none().then_some(1.0),
        }
    }
}

impl TextIndex for MemoryTextIndex {
    fn schema(&self) -> TextSchema {
        self.schema.clone()
    }

    fn upsert(&self, document: TextDocument) -> Result<()> {
        self.validate_document(&document)?;
        let mut state = self.state.write().map_err(|_| IndexError::LockPoisoned)?;
        if state
            .tombstones
            .get(&document.external_id)
            .is_some_and(|delete_version| *delete_version >= document.version)
            || state
                .documents
                .get(&document.external_id)
                .is_some_and(|current| current.version >= document.version)
        {
            return Ok(());
        }
        state.tombstones.remove(&document.external_id);
        state
            .documents
            .insert(document.external_id.clone(), document);
        Ok(())
    }

    fn delete(&self, external_id: &str, version: Option<u64>) -> Result<bool> {
        self.validate_external_id(external_id)?;
        let mut state = self.state.write().map_err(|_| IndexError::LockPoisoned)?;
        let current_version = state
            .documents
            .get(external_id)
            .map(|document| document.version);
        let delete_version = match (version, current_version) {
            (Some(delete_version), Some(current_version)) if current_version > delete_version => {
                return Ok(false);
            }
            (Some(delete_version), _) => delete_version,
            (None, Some(current_version)) => current_version,
            (None, None) => return Ok(false),
        };
        let removed = state.documents.remove(external_id).is_some();
        state
            .tombstones
            .entry(external_id.to_string())
            .and_modify(|current| *current = (*current).max(delete_version))
            .or_insert(delete_version);
        Ok(removed)
    }

    fn search(&self, query: &TextQuery, limit: usize) -> Result<Vec<TextHit>> {
        self.validate_query(query)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let state = self.state.read().map_err(|_| IndexError::LockPoisoned)?;
        let mut hits = state
            .documents
            .values()
            .filter_map(|document| {
                self.evaluate(document, query).map(|score| TextHit {
                    external_id: document.external_id.clone(),
                    version: document.version,
                    score,
                })
            })
            .collect::<Vec<_>>();
        hits.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.external_id.cmp(&right.external_id))
        });
        hits.truncate(limit);
        Ok(hits)
    }

    fn snapshot(&self) -> Result<TextIndexSnapshot> {
        let state = self.state.read().map_err(|_| IndexError::LockPoisoned)?;
        Ok(TextIndexSnapshot {
            format_version: SNAPSHOT_FORMAT_VERSION,
            schema: self.schema.clone(),
            documents: state.documents.values().cloned().collect(),
            tombstones: state.tombstones.clone(),
        })
    }

    fn restore(&self, snapshot: &TextIndexSnapshot) -> Result<()> {
        if snapshot.format_version != SNAPSHOT_FORMAT_VERSION {
            return Err(IndexError::CorruptSnapshot {
                message: format!(
                    "unsupported format version {}; expected {SNAPSHOT_FORMAT_VERSION}",
                    snapshot.format_version
                ),
            });
        }
        if snapshot.schema != self.schema {
            return Err(IndexError::CorruptSnapshot {
                message: "snapshot schema does not match the opened index".to_string(),
            });
        }
        let restored = self.build_state(snapshot.documents.clone(), snapshot.tombstones.clone())?;
        *self.state.write().map_err(|_| IndexError::LockPoisoned)? = restored;
        Ok(())
    }

    fn rebuild(&self, documents: Vec<TextDocument>) -> Result<()> {
        let rebuilt = self.build_state(documents, BTreeMap::new())?;
        *self.state.write().map_err(|_| IndexError::LockPoisoned)? = rebuilt;
        Ok(())
    }
}

pub fn tokenize(text: &str, analyzer: Analyzer) -> Vec<String> {
    match analyzer {
        Analyzer::WhitespaceLower => {
            let mut out = Vec::new();
            for_whitespace_lower(text, |token| out.push(token));
            out
        }
        Analyzer::Jieba => jieba(text),
        Analyzer::Ngram => ngram(text, DEFAULT_NGRAM_MIN, DEFAULT_NGRAM_MAX),
    }
}

pub fn for_whitespace_lower(text: &str, mut emit: impl FnMut(String)) -> u32 {
    for_whitespace_lower_cow(text, |token| emit(token.into_owned()))
}

pub fn for_whitespace_lower_cow<'a>(mut text: &'a str, mut emit: impl FnMut(Cow<'a, str>)) -> u32 {
    let mut emitted = 0u32;
    while !text.is_empty() {
        let trimmed_start = text.trim_start();
        if trimmed_start.is_empty() {
            break;
        }
        text = trimmed_start;
        let end = text.find(char::is_whitespace).unwrap_or(text.len());
        let raw = &text[..end];
        text = &text[end..];
        let token = raw.trim_matches(|character: char| !character.is_alphanumeric());
        if token.is_empty() {
            continue;
        }
        emitted += 1;
        if token
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            emit(Cow::Borrowed(token));
        } else {
            emit(Cow::Owned(token.to_lowercase()));
        }
    }
    emitted
}

#[cfg(feature = "jieba")]
fn jieba(text: &str) -> Vec<String> {
    use std::sync::OnceLock;

    static JIEBA: OnceLock<jieba_rs::Jieba> = OnceLock::new();
    JIEBA
        .get_or_init(jieba_rs::Jieba::new)
        .cut(text, false)
        .into_iter()
        .map(str::to_lowercase)
        .filter(|token| !token.trim().is_empty())
        .collect()
}

#[cfg(not(feature = "jieba"))]
fn jieba(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut tokens = Vec::new();
    let characters = trimmed.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        let cjk = is_cjk_char(characters[index]);
        let start = index;
        while index < characters.len() && is_cjk_char(characters[index]) == cjk {
            index += 1;
        }
        if cjk {
            let run = &characters[start..index];
            if run.len() == 1 {
                tokens.push(run[0].to_string());
            } else {
                for offset in 0..run.len() - 1 {
                    tokens.push(run[offset..offset + 2].iter().collect());
                }
            }
        } else {
            let run = characters[start..index].iter().collect::<String>();
            for_whitespace_lower(&run, |token| tokens.push(token));
        }
    }
    tokens
}

fn is_cjk_char(character: char) -> bool {
    let code = character as u32;
    (0x4E00..=0x9FFF).contains(&code)
        || (0x3400..=0x4DBF).contains(&code)
        || (0x3040..=0x309F).contains(&code)
        || (0x30A0..=0x30FF).contains(&code)
        || (0xAC00..=0xD7A3).contains(&code)
}

fn ngram(text: &str, min: usize, max: usize) -> Vec<String> {
    let characters = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<Vec<_>>();
    let mut tokens = Vec::new();
    for window in min..=max {
        if characters.len() < window {
            continue;
        }
        for start in 0..=characters.len() - window {
            tokens.push(characters[start..start + window].iter().collect());
        }
    }
    tokens
}
