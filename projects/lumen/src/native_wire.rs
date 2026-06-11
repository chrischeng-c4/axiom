// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-native-wire-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! Native binary search wire.
//!
//! The public HTTP/JSON API remains lumen's integration surface. This module is
//! a small length-prefixed CBOR transport for Rust/native clients that need the
//! same engine over a lower fixed-cost wire, especially for sub-100us predicate
//! lookups where HTTP framing dominates the index work.

use std::{io::Cursor, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
};

use crate::{
    storage::Engine,
    types::{
        FieldValue, QueryNode, RangeQuery, SearchHit, SearchRequest, SearchResponse, TermQuery,
    },
};

const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
const FAST_MAGIC: &[u8; 3] = b"LMN";
const FAST_VER: u8 = 1;
const FAST_TERM: u8 = 1;
const FAST_RANGE: u8 = 2;
const FAST_TERM_RANGE: u8 = 3;
const FAST_RESPONSE: u8 = 0x80;
const FAST_OK: u8 = 0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeSearchRequest {
    pub collection_id: String,
    pub request: SearchRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum NativeSearchResponse {
    Ok { response: SearchResponse },
    Err { message: String },
}

/// Serve native binary search on an already-bound listener.
pub async fn serve_search(listener: TcpListener, engine: Arc<Engine>) -> Result<()> {
    loop {
        let (stream, _) = listener.accept().await.context("native accept")?;
        let engine = engine.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_conn(stream, engine).await {
                tracing::debug!(error = %err, "native search connection closed");
            }
        });
    }
}

#[cfg(unix)]
pub async fn serve_unix_search(listener: UnixListener, engine: Arc<Engine>) -> Result<()> {
    loop {
        let (stream, _) = listener.accept().await.context("native unix accept")?;
        let engine = engine.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_conn(stream, engine).await {
                tracing::debug!(error = %err, "native unix search connection closed");
            }
        });
    }
}

/// Encode a prepared search frame that can be written repeatedly on a persistent
/// connection. This is the native analogue of pg's prepared statement path in
/// the competitive gate.
pub fn encode_search_frame(collection_id: &str, request: &SearchRequest) -> Result<Vec<u8>> {
    encode_frame(&NativeSearchRequest {
        collection_id: collection_id.to_string(),
        request: request.clone(),
    })
}

pub fn encode_term_frame(
    collection_id: &str,
    field: &str,
    value: &str,
    limit: u32,
) -> Result<Vec<u8>> {
    let mut payload = fast_header(FAST_TERM);
    put_str(&mut payload, collection_id)?;
    put_str(&mut payload, field)?;
    put_str(&mut payload, value)?;
    payload.extend_from_slice(&limit.to_be_bytes());
    frame_payload(payload)
}

pub fn encode_range_frame(
    collection_id: &str,
    field: &str,
    gte: Option<f64>,
    lt: Option<f64>,
    limit: u32,
) -> Result<Vec<u8>> {
    let mut payload = fast_header(FAST_RANGE);
    put_str(&mut payload, collection_id)?;
    put_str(&mut payload, field)?;
    put_bound(&mut payload, gte);
    put_bound(&mut payload, lt);
    payload.extend_from_slice(&limit.to_be_bytes());
    frame_payload(payload)
}

pub fn encode_term_range_frame(
    collection_id: &str,
    term_field: &str,
    term_value: &str,
    range_field: &str,
    gte: Option<f64>,
    lt: Option<f64>,
    limit: u32,
) -> Result<Vec<u8>> {
    let mut payload = fast_header(FAST_TERM_RANGE);
    put_str(&mut payload, collection_id)?;
    put_str(&mut payload, term_field)?;
    put_str(&mut payload, term_value)?;
    put_str(&mut payload, range_field)?;
    put_bound(&mut payload, gte);
    put_bound(&mut payload, lt);
    payload.extend_from_slice(&limit.to_be_bytes());
    frame_payload(payload)
}

/// Send one already-encoded native search request and decode its response.
pub async fn search_prepared<S>(stream: &mut S, frame: &[u8]) -> Result<SearchResponse>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    stream
        .write_all(frame)
        .await
        .context("native write request")?;
    let Some(resp_frame) = read_frame(stream).await.context("native read response")? else {
        bail!("native connection closed before response");
    };
    if is_fast_response(&resp_frame) {
        return decode_fast_response(&resp_frame);
    }
    match decode_response(&resp_frame)? {
        NativeSearchResponse::Ok { response } => Ok(response),
        NativeSearchResponse::Err { message } => Err(anyhow!(message)),
    }
}

async fn handle_conn<S>(mut stream: S, engine: Arc<Engine>) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    while let Some(frame) = read_frame(&mut stream).await? {
        let out = if is_fast_request(&frame) {
            match handle_fast_frame(&engine, &frame) {
                Ok(out) => out,
                Err(err) => encode_frame(&NativeSearchResponse::Err {
                    message: err.to_string(),
                })?,
            }
        } else {
            let req: NativeSearchRequest = ciborium::de::from_reader(Cursor::new(&frame))
                .context("decode native search request")?;
            let resp = match engine.search(&req.collection_id, req.request) {
                Ok(response) => NativeSearchResponse::Ok { response },
                Err(err) => NativeSearchResponse::Err {
                    message: err.to_string(),
                },
            };
            encode_frame(&resp)?
        };
        stream
            .write_all(&out)
            .await
            .context("native write response")?;
    }
    Ok(())
}

fn encode_frame<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    ciborium::ser::into_writer(value, &mut payload).context("encode native frame")?;
    frame_payload(payload)
}

fn frame_payload(payload: Vec<u8>) -> Result<Vec<u8>> {
    if payload.len() > MAX_FRAME_BYTES {
        bail!(
            "native frame too large: {} > {}",
            payload.len(),
            MAX_FRAME_BYTES
        );
    }
    let len = payload.len() as u32;
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

fn fast_header(op: u8) -> Vec<u8> {
    let mut payload = Vec::with_capacity(128);
    payload.extend_from_slice(FAST_MAGIC);
    payload.push(FAST_VER);
    payload.push(op);
    payload
}

fn put_str(out: &mut Vec<u8>, s: &str) -> Result<()> {
    let len: u32 = s
        .len()
        .try_into()
        .map_err(|_| anyhow!("native string too long"))?;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn put_bound(out: &mut Vec<u8>, value: Option<f64>) {
    match value {
        Some(v) => {
            out.push(1);
            out.extend_from_slice(&v.to_bits().to_be_bytes());
        }
        None => out.push(0),
    }
}

fn is_fast_request(frame: &[u8]) -> bool {
    frame.len() >= 5
        && &frame[0..3] == FAST_MAGIC
        && frame[3] == FAST_VER
        && frame[4] < FAST_RESPONSE
}

fn is_fast_response(frame: &[u8]) -> bool {
    frame.len() >= 6
        && &frame[0..3] == FAST_MAGIC
        && frame[3] == FAST_VER
        && frame[4] == FAST_RESPONSE
}

fn handle_fast_frame(engine: &Engine, frame: &[u8]) -> Result<Vec<u8>> {
    let mut pos = 5usize;
    let op = *frame.get(4).ok_or_else(|| anyhow!("missing native op"))?;
    if op == FAST_TERM {
        let collection_id = take_str_ref(frame, &mut pos)?;
        let field = take_str_ref(frame, &mut pos)?;
        let value = take_str_ref(frame, &mut pos)?;
        let limit = take_u32(frame, &mut pos)?;
        if pos != frame.len() {
            bail!("native frame has {} trailing bytes", frame.len() - pos);
        }
        let response = engine.search_fast_string_term(collection_id, field, value, limit)?;
        return encode_fast_response(&response);
    }

    let (collection_id, request) = match op {
        FAST_RANGE => {
            let collection_id = take_str(frame, &mut pos)?;
            let field = take_str(frame, &mut pos)?;
            let gte = take_bound(frame, &mut pos)?;
            let lt = take_bound(frame, &mut pos)?;
            let limit = take_u32(frame, &mut pos)?;
            (
                collection_id,
                SearchRequest {
                    query: QueryNode::Range(RangeQuery {
                        field,
                        gt: None,
                        gte,
                        lt,
                        lte: None,
                    }),
                    limit,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
        }
        FAST_TERM_RANGE => {
            let collection_id = take_str(frame, &mut pos)?;
            let term_field = take_str(frame, &mut pos)?;
            let term_value = take_str(frame, &mut pos)?;
            let range_field = take_str(frame, &mut pos)?;
            let gte = take_bound(frame, &mut pos)?;
            let lt = take_bound(frame, &mut pos)?;
            let limit = take_u32(frame, &mut pos)?;
            (
                collection_id,
                SearchRequest {
                    query: QueryNode::And(vec![
                        QueryNode::Term(TermQuery {
                            field: term_field,
                            value: FieldValue::String(term_value),
                        }),
                        QueryNode::Range(RangeQuery {
                            field: range_field,
                            gt: None,
                            gte,
                            lt,
                            lte: None,
                        }),
                    ]),
                    limit,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
        }
        _ => bail!("unknown native op {op}"),
    };
    if pos != frame.len() {
        bail!("native frame has {} trailing bytes", frame.len() - pos);
    }
    let response = engine.search(&collection_id, request)?;
    encode_fast_response(&response)
}

fn encode_fast_response(response: &SearchResponse) -> Result<Vec<u8>> {
    let mut payload = fast_header(FAST_RESPONSE);
    payload.push(FAST_OK);
    payload.extend_from_slice(&response.took_us.to_be_bytes());
    payload.extend_from_slice(&response.total.to_be_bytes());
    payload.extend_from_slice(&(response.hits.len() as u32).to_be_bytes());
    for hit in &response.hits {
        put_str(&mut payload, &hit.external_id)?;
        payload.extend_from_slice(&hit.score.to_bits().to_be_bytes());
    }
    frame_payload(payload)
}

fn decode_fast_response(frame: &[u8]) -> Result<SearchResponse> {
    let mut pos = 5usize;
    let status = *frame
        .get(pos)
        .ok_or_else(|| anyhow!("missing native response status"))?;
    pos += 1;
    if status != FAST_OK {
        bail!("native fast response status {status}");
    }
    let took_us = take_u64(frame, &mut pos)?;
    let total = take_u64(frame, &mut pos)?;
    let count = take_u32(frame, &mut pos)? as usize;
    let mut hits = Vec::with_capacity(count);
    for _ in 0..count {
        let external_id = take_str(frame, &mut pos)?;
        let score = f32::from_bits(take_u32(frame, &mut pos)?);
        hits.push(SearchHit { external_id, score });
    }
    if pos != frame.len() {
        bail!(
            "native fast response has {} trailing bytes",
            frame.len() - pos
        );
    }
    Ok(SearchResponse {
        hits,
        total,
        cursor: None,
        took_ms: took_us / 1000,
        took_us,
    })
}

fn take<'a>(frame: &'a [u8], pos: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = pos
        .checked_add(n)
        .ok_or_else(|| anyhow!("native frame offset overflow"))?;
    let bytes = frame
        .get(*pos..end)
        .ok_or_else(|| anyhow!("native frame truncated"))?;
    *pos = end;
    Ok(bytes)
}

fn take_u32(frame: &[u8], pos: &mut usize) -> Result<u32> {
    let bytes: [u8; 4] = take(frame, pos, 4)?.try_into().unwrap();
    Ok(u32::from_be_bytes(bytes))
}

fn take_u64(frame: &[u8], pos: &mut usize) -> Result<u64> {
    let bytes: [u8; 8] = take(frame, pos, 8)?.try_into().unwrap();
    Ok(u64::from_be_bytes(bytes))
}

fn take_bound(frame: &[u8], pos: &mut usize) -> Result<Option<f64>> {
    match *take(frame, pos, 1)?
        .first()
        .ok_or_else(|| anyhow!("missing native bound flag"))?
    {
        0 => Ok(None),
        1 => Ok(Some(f64::from_bits(take_u64(frame, pos)?))),
        tag => bail!("unknown native bound flag {tag}"),
    }
}

fn take_str(frame: &[u8], pos: &mut usize) -> Result<String> {
    let len = take_u32(frame, pos)? as usize;
    let bytes = take(frame, pos, len)?;
    String::from_utf8(bytes.to_vec()).context("native string is not utf-8")
}

fn take_str_ref<'a>(frame: &'a [u8], pos: &mut usize) -> Result<&'a str> {
    let len = take_u32(frame, pos)? as usize;
    let bytes = take(frame, pos, len)?;
    std::str::from_utf8(bytes).context("native string is not utf-8")
}

fn decode_response(bytes: &[u8]) -> Result<NativeSearchResponse> {
    ciborium::de::from_reader(Cursor::new(bytes)).context("decode native search response")
}

async fn read_frame<S>(stream: &mut S) -> Result<Option<Vec<u8>>>
where
    S: AsyncRead + Unpin,
{
    let mut hdr = [0u8; 4];
    match stream.read_exact(&mut hdr).await {
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err).context("read native frame header"),
    }
    let len = u32::from_be_bytes(hdr) as usize;
    if len > MAX_FRAME_BYTES {
        bail!("native frame too large: {len} > {MAX_FRAME_BYTES}");
    }
    let mut payload = vec![0u8; len];
    stream
        .read_exact(&mut payload)
        .await
        .context("read native frame payload")?;
    Ok(Some(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{QueryNode, TermQuery};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn native_search_round_trips_on_persistent_conn() {
        let engine = Arc::new(Engine::new());
        engine
            .create_collection(
                "docs",
                crate::types::CreateCollectionRequest {
                    fields: [(
                        "city".to_string(),
                        crate::types::FieldSpec {
                            field_type: crate::types::FieldType::Keyword,
                            analyzer: None,
                            multi: None,
                            dim: None,
                            metric: None,
                            backend: None,
                            quantize: None,
                        },
                    )]
                    .into_iter()
                    .collect(),
                },
            )
            .unwrap();
        engine
            .index(
                "docs",
                crate::types::IndexRequest {
                    items: vec![crate::types::IndexItem {
                        external_id: "a".to_string(),
                        field: "city".to_string(),
                        value: crate::types::FieldValue::String("taipei".to_string()),
                    }],
                    request_id: None,
                },
            )
            .unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let serve_engine = engine.clone();
        tokio::spawn(async move {
            let _ = serve_search(listener, serve_engine).await;
        });

        let mut stream = TcpStream::connect(addr).await.unwrap();
        let frame = encode_search_frame(
            "docs",
            &SearchRequest {
                query: QueryNode::Term(TermQuery {
                    field: "city".to_string(),
                    value: crate::types::FieldValue::String("taipei".to_string()),
                }),
                limit: 10,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap();
        let first = search_prepared(&mut stream, &frame).await.unwrap();
        let second = search_prepared(&mut stream, &frame).await.unwrap();
        let fast = search_prepared(
            &mut stream,
            &encode_term_frame("docs", "city", "taipei", 10).unwrap(),
        )
        .await
        .unwrap();

        assert_eq!(first.total, 1);
        assert_eq!(second.hits[0].external_id, "a");
        assert_eq!(fast.hits[0].external_id, "a");
    }
}

// </HANDWRITE>
