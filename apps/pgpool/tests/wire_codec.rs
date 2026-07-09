// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#unit-test
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Offline (no live Postgres) integration coverage for the pgpool wire
//! codec, one test function per TD Unit Test requirement (R1-R13).

use bytes::{Bytes, BytesMut};
use pgpool::wire::{
    AuthenticationCleartextPassword, AuthenticationMd5Password, AuthenticationOk,
    AuthenticationSasl, AuthenticationSaslContinue, AuthenticationSaslFinal, BackendKeyData,
    BackendMessage, Bind, CommandComplete, DataRow, Describe, DescribeTarget, ErrorResponse,
    Execute, FieldDescription, Frame, FrameError, FrameReader, FrontendMessage, NoticeResponse,
    ParameterStatus, Parse, PasswordMessage, Query, ReadyForQuery, Role, RowDescription,
    SaslInitialResponse, SaslResponse, SslRequest, StartupMessage, Sync, Terminate,
    TransactionStatus, WireCodecConfig, WireMessage,
};

/// Test-local helper: parses a fully-encoded tagged frame's bytes back into
/// its raw `Frame` (tag + payload), mirroring what `FrameReader` does
/// internally. Used for the SASL 'p'-tag variants, which the generic
/// `FrontendMessage::decode` dispatcher can't distinguish from
/// `PasswordMessage` by tag byte alone (see `FrontendMessage::decode` docs).
fn parse_tagged_frame(bytes: &[u8]) -> Frame {
    assert!(
        bytes.len() >= 5,
        "tagged frame must have at least a 5-byte header"
    );
    let tag = bytes[0];
    let declared_length = i32::from_be_bytes(bytes[1..5].try_into().unwrap()) as usize;
    assert_eq!(
        bytes.len(),
        1 + declared_length,
        "frame bytes must match declared length"
    );
    Frame {
        tag: Some(tag),
        payload: Bytes::copy_from_slice(&bytes[5..]),
    }
}

fn encode(message: &FrontendMessage) -> BytesMut {
    let mut buf = BytesMut::new();
    message.encode(&mut buf);
    buf
}

fn encode_backend(message: &BackendMessage) -> BytesMut {
    let mut buf = BytesMut::new();
    message.encode(&mut buf);
    buf
}

/// Round-trips a frontend message through a fresh `FrameReader`, asserting
/// the decoded value matches and that re-encoding it byte-for-byte
/// reproduces the original wire bytes.
fn round_trip_frontend(message: FrontendMessage, config: &WireCodecConfig) -> FrontendMessage {
    let encoded = encode(&message);
    let mut reader = FrameReader::new(Role::Frontend, config);
    if !matches!(
        message,
        FrontendMessage::Startup(_) | FrontendMessage::Ssl(_)
    ) {
        // Tagged frontend frames are only legal after the untagged startup
        // packet, so advance the reader through a startup exchange first.
        let startup = encode(&FrontendMessage::Startup(StartupMessage {
            protocol_major: 3,
            protocol_minor: 0,
            parameters: vec![("user".to_string(), "app".to_string())],
        }));
        reader.feed(&startup);
        reader
            .next_frame()
            .expect("startup decode succeeds")
            .expect("startup frame is fully buffered");
    }
    reader.feed(&encoded);
    let decoded = match reader
        .next_frame()
        .expect("decode succeeds")
        .expect("frame is fully buffered")
    {
        WireMessage::Frontend(m) => m,
        WireMessage::Backend(_) => panic!("expected a frontend message"),
    };
    assert_eq!(decoded, message, "decoded value must equal the original");
    let re_encoded = encode(&decoded);
    assert_eq!(
        re_encoded, encoded,
        "re-encoding the decoded value must reproduce the original bytes"
    );
    decoded
}

fn round_trip_backend(message: BackendMessage, config: &WireCodecConfig) -> BackendMessage {
    let encoded = encode_backend(&message);
    let mut reader = FrameReader::new(Role::Backend, config);
    reader.feed(&encoded);
    let decoded = match reader
        .next_frame()
        .expect("decode succeeds")
        .expect("frame is fully buffered")
    {
        WireMessage::Backend(m) => m,
        WireMessage::Frontend(_) => panic!("expected a backend message"),
    };
    assert_eq!(decoded, message, "decoded value must equal the original");
    let re_encoded = encode_backend(&decoded);
    assert_eq!(
        re_encoded, encoded,
        "re-encoding the decoded value must reproduce the original bytes"
    );
    decoded
}

// R1: StartupMessage + SSLRequest untagged round trip.
#[test]
fn frontend_startup_and_ssl_round_trip() {
    let config = WireCodecConfig::default();

    let startup = FrontendMessage::Startup(StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![
            ("user".to_string(), "alice".to_string()),
            ("database".to_string(), "app_db".to_string()),
            ("application_name".to_string(), "pgpool-test".to_string()),
        ],
    });
    round_trip_frontend(startup, &config);

    // SSLRequest is a fixed 8-byte packet: 4-byte length (8) + request code.
    let ssl = FrontendMessage::Ssl(SslRequest);
    let encoded = encode(&ssl);
    assert_eq!(encoded.len(), 8);
    assert_eq!(&encoded[0..4], &8i32.to_be_bytes());
    assert_eq!(&encoded[4..8], &80_877_103i32.to_be_bytes());
    round_trip_frontend(ssl, &config);

    // A fresh connection can see SSLRequest *then* StartupMessage, both
    // untagged, before switching to tagged frames.
    let mut reader = FrameReader::new(Role::Frontend, &config);
    let ssl_bytes = encode(&FrontendMessage::Ssl(SslRequest));
    reader.feed(&ssl_bytes);
    assert!(matches!(
        reader.next_frame().unwrap(),
        Some(WireMessage::Frontend(FrontendMessage::Ssl(_)))
    ));

    let startup2 = FrontendMessage::Startup(StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![("user".to_string(), "bob".to_string())],
    });
    let startup_bytes = encode(&startup2);
    reader.feed(&startup_bytes);
    match reader.next_frame().unwrap() {
        Some(WireMessage::Frontend(FrontendMessage::Startup(m))) => {
            assert_eq!(m.parameters, vec![("user".to_string(), "bob".to_string())])
        }
        other => panic!("expected StartupMessage after SSLRequest, got {other:?}"),
    }

    // After the real StartupMessage, subsequent frames are tagged.
    let query_bytes = encode(&FrontendMessage::Query(Query {
        sql: "SELECT 1".to_string(),
    }));
    reader.feed(&query_bytes);
    assert!(matches!(
        reader.next_frame().unwrap(),
        Some(WireMessage::Frontend(FrontendMessage::Query(_)))
    ));
}

// R2: PasswordMessage + SASL initial-response/response round trip.
#[test]
fn frontend_password_and_sasl_round_trip() {
    let config = WireCodecConfig::default();

    let password = FrontendMessage::Password(PasswordMessage {
        payload: Bytes::from_static(b"md5abcdef0123456789"),
    });
    round_trip_frontend(password, &config);

    // SaslInitialResponse and SaslResponse both share tag 'p' with
    // PasswordMessage, so they're decoded directly (not via the generic
    // FrontendMessage::decode dispatcher — see its doc comment).
    let sir_with_response = SaslInitialResponse {
        mechanism: "SCRAM-SHA-256".to_string(),
        response: Some(Bytes::from_static(b"n,,n=user,r=clientnonce")),
    };
    let msg = FrontendMessage::SaslInitialResponse(sir_with_response.clone());
    let encoded = encode(&msg);
    let frame = parse_tagged_frame(&encoded);
    assert_eq!(frame.tag, Some(b'p'));
    let decoded = SaslInitialResponse::decode(&frame.payload).expect("decode ok");
    assert_eq!(decoded, sir_with_response);
    let mut re_encoded_buf = BytesMut::new();
    decoded.encode(&mut re_encoded_buf);
    assert_eq!(re_encoded_buf, encoded);

    let sir_no_response = SaslInitialResponse {
        mechanism: "SCRAM-SHA-256".to_string(),
        response: None,
    };
    let encoded2 = encode(&FrontendMessage::SaslInitialResponse(
        sir_no_response.clone(),
    ));
    let frame2 = parse_tagged_frame(&encoded2);
    let decoded2 = SaslInitialResponse::decode(&frame2.payload).expect("decode ok");
    assert_eq!(decoded2, sir_no_response);

    let sasl_response = SaslResponse {
        payload: Bytes::from_static(b"c=biws,r=clientservernonce,p=proof"),
    };
    let encoded3 = encode(&FrontendMessage::SaslResponse(sasl_response.clone()));
    let frame3 = parse_tagged_frame(&encoded3);
    assert_eq!(frame3.tag, Some(b'p'));
    let decoded3 = SaslResponse::decode(&frame3.payload);
    assert_eq!(decoded3, sasl_response);
}

// R3: simple query round trip.
#[test]
fn frontend_query_round_trip() {
    let config = WireCodecConfig::default();
    let query = FrontendMessage::Query(Query {
        sql: "SELECT * FROM widgets WHERE id = 1".to_string(),
    });
    round_trip_frontend(query, &config);
}

// R4: Parse/Bind/Describe/Execute/Sync extended-query round trip.
#[test]
fn frontend_extended_query_round_trip() {
    let config = WireCodecConfig::default();

    let parse = FrontendMessage::Parse(Parse {
        statement_name: "stmt1".to_string(),
        sql: "SELECT $1::int, $2::text".to_string(),
        param_type_oids: vec![23, 25],
    });
    round_trip_frontend(parse, &config);

    let bind = FrontendMessage::Bind(Bind {
        portal_name: "portal1".to_string(),
        statement_name: "stmt1".to_string(),
        param_formats: vec![0, 1],
        param_values: vec![
            Some(Bytes::from_static(b"42")),
            None,
            Some(Bytes::from_static(b"hello")),
        ],
        result_formats: vec![0],
    });
    round_trip_frontend(bind, &config);

    let describe_statement = FrontendMessage::Describe(Describe {
        target_kind: DescribeTarget::Statement,
        name: "stmt1".to_string(),
    });
    round_trip_frontend(describe_statement, &config);

    let describe_portal = FrontendMessage::Describe(Describe {
        target_kind: DescribeTarget::Portal,
        name: "portal1".to_string(),
    });
    round_trip_frontend(describe_portal, &config);

    let execute = FrontendMessage::Execute(Execute {
        portal_name: "portal1".to_string(),
        max_rows: 0,
    });
    round_trip_frontend(execute, &config);

    let execute_limited = FrontendMessage::Execute(Execute {
        portal_name: "portal1".to_string(),
        max_rows: 100,
    });
    round_trip_frontend(execute_limited, &config);

    let sync = FrontendMessage::Sync(Sync);
    round_trip_frontend(sync, &config);
}

// R5: Terminate round trip, fixed-shape fieldless message.
#[test]
fn frontend_terminate_round_trip() {
    let config = WireCodecConfig::default();
    let terminate = FrontendMessage::Terminate(Terminate);
    let encoded = encode(&terminate);
    assert_eq!(encoded.as_ref(), &[b'X', 0, 0, 0, 4]);
    round_trip_frontend(terminate, &config);
}

// R6: Authentication backend message family round trip.
#[test]
fn backend_authentication_family_round_trip() {
    let config = WireCodecConfig::default();

    round_trip_backend(BackendMessage::AuthenticationOk(AuthenticationOk), &config);
    round_trip_backend(
        BackendMessage::AuthenticationCleartextPassword(AuthenticationCleartextPassword),
        &config,
    );
    round_trip_backend(
        BackendMessage::AuthenticationMd5Password(AuthenticationMd5Password {
            salt: [0xDE, 0xAD, 0xBE, 0xEF],
        }),
        &config,
    );
    round_trip_backend(
        BackendMessage::AuthenticationSasl(AuthenticationSasl {
            mechanisms: vec![
                "SCRAM-SHA-256".to_string(),
                "SCRAM-SHA-256-PLUS".to_string(),
            ],
        }),
        &config,
    );
    round_trip_backend(
        BackendMessage::AuthenticationSaslContinue(AuthenticationSaslContinue {
            payload: Bytes::from_static(b"r=servernonce,s=salt,i=4096"),
        }),
        &config,
    );
    round_trip_backend(
        BackendMessage::AuthenticationSaslFinal(AuthenticationSaslFinal {
            payload: Bytes::from_static(b"v=serverproof"),
        }),
        &config,
    );
}

// R7: ParameterStatus, BackendKeyData, ReadyForQuery round trip.
#[test]
fn backend_parameter_status_keydata_ready_round_trip() {
    let config = WireCodecConfig::default();

    round_trip_backend(
        BackendMessage::ParameterStatus(ParameterStatus {
            name: "server_version".to_string(),
            value: "16.2".to_string(),
        }),
        &config,
    );
    round_trip_backend(
        BackendMessage::BackendKeyData(BackendKeyData {
            process_id: 4242,
            secret_key: -123_456,
        }),
        &config,
    );

    for status in [
        TransactionStatus::Idle,
        TransactionStatus::InTransaction,
        TransactionStatus::Failed,
    ] {
        let decoded = round_trip_backend(
            BackendMessage::ReadyForQuery(ReadyForQuery { status }),
            &config,
        );
        match decoded {
            BackendMessage::ReadyForQuery(r) => assert_eq!(r.status, status),
            _ => panic!("expected ReadyForQuery"),
        }
    }
}

// R8: RowDescription, DataRow, CommandComplete result-set round trip.
#[test]
fn backend_result_set_round_trip() {
    let config = WireCodecConfig::default();

    let row_description = BackendMessage::RowDescription(RowDescription {
        fields: vec![
            FieldDescription {
                name: "id".to_string(),
                table_oid: 16384,
                column_attr: 1,
                type_oid: 23,
                type_size: 4,
                type_modifier: -1,
                format: 0,
            },
            FieldDescription {
                name: "name".to_string(),
                table_oid: 16384,
                column_attr: 2,
                type_oid: 25,
                type_size: -1,
                type_modifier: -1,
                format: 0,
            },
        ],
    });
    round_trip_backend(row_description, &config);

    let data_row = BackendMessage::DataRow(DataRow {
        columns: vec![
            Some(Bytes::from_static(b"1")),
            Some(Bytes::from_static(b"widget")),
            None,
        ],
    });
    round_trip_backend(data_row, &config);

    let command_complete = BackendMessage::CommandComplete(CommandComplete {
        tag: "SELECT 3".to_string(),
    });
    round_trip_backend(command_complete, &config);
}

// R9: ErrorResponse and NoticeResponse round trip.
#[test]
fn backend_error_notice_round_trip() {
    let config = WireCodecConfig::default();

    let error_response = BackendMessage::ErrorResponse(ErrorResponse {
        fields: vec![
            (b'S', "ERROR".to_string()),
            (b'C', "42601".to_string()),
            (b'M', "syntax error at or near \"SELCT\"".to_string()),
        ],
    });
    round_trip_backend(error_response, &config);

    let notice_response = BackendMessage::NoticeResponse(NoticeResponse {
        fields: vec![
            (b'S', "NOTICE".to_string()),
            (b'M', "identifier truncated".to_string()),
        ],
    });
    round_trip_backend(notice_response, &config);
}

// R10: FrameReader rejects oversized frames with FrameError::Oversized,
// without panicking and without buffering the oversized payload.
#[test]
fn reader_rejects_oversized_frame() {
    let config = WireCodecConfig::default();

    // Tagged frame: 1 tag byte + i32 declared length that exceeds
    // max_frame_bytes. Only the 5-byte header is fed - the (huge) body is
    // never sent, proving the oversized payload is never buffered.
    let mut reader = FrameReader::new(Role::Backend, &config);
    let oversized_declared = (config.max_frame_bytes as i32) + 1;
    let mut header = Vec::new();
    header.push(b'Q');
    header.extend_from_slice(&oversized_declared.to_be_bytes());
    reader.feed(&header);
    match reader.next_frame() {
        Err(FrameError::Oversized { declared, max }) => {
            assert_eq!(declared, oversized_declared as usize);
            assert_eq!(max, config.max_frame_bytes);
        }
        other => panic!("expected FrameError::Oversized, got {other:?}"),
    }

    // Untagged startup packet exceeding max_startup_bytes.
    let mut reader2 = FrameReader::new(Role::Frontend, &config);
    let oversized_startup = (config.max_startup_bytes as i32) + 1;
    reader2.feed(&oversized_startup.to_be_bytes());
    match reader2.next_frame() {
        Err(FrameError::Oversized { declared, max }) => {
            assert_eq!(declared, oversized_startup as usize);
            assert_eq!(max, config.max_startup_bytes);
        }
        other => panic!("expected FrameError::Oversized for startup packet, got {other:?}"),
    }
}

// R11: decode returns typed errors (never panics) for truncated fields, bad
// UTF-8, unknown enum discriminants, and unrecognized tag bytes.
#[test]
fn decode_rejects_malformed_input_without_panic() {
    let config = WireCodecConfig::default();

    // Truncated field: Query payload with no null terminator at all.
    let truncated = Frame {
        tag: Some(b'Q'),
        payload: Bytes::from_static(b"SELECT 1"),
    };
    match FrontendMessage::decode(&truncated, &config) {
        Err(FrameError::Malformed { tag, .. }) => assert_eq!(tag, Some(b'Q')),
        other => panic!("expected Malformed for truncated query, got {other:?}"),
    }

    // Bad UTF-8 inside a null-terminated string field.
    let mut bad_utf8 = BytesMut::new();
    bad_utf8.extend_from_slice(&[0xFF, 0xFE, 0x00]);
    let bad_utf8_frame = Frame {
        tag: Some(b'Q'),
        payload: bad_utf8.freeze(),
    };
    match FrontendMessage::decode(&bad_utf8_frame, &config) {
        Err(FrameError::Malformed { tag, .. }) => assert_eq!(tag, Some(b'Q')),
        other => panic!("expected Malformed for invalid UTF-8, got {other:?}"),
    }

    // Unknown enum discriminant: ReadyForQuery with an invalid status byte.
    let bad_status = Frame {
        tag: Some(b'Z'),
        payload: Bytes::from_static(b"X"),
    };
    match BackendMessage::decode(&bad_status, &config) {
        Err(FrameError::Malformed { tag, .. }) => assert_eq!(tag, Some(b'Z')),
        other => panic!("expected Malformed for unknown ReadyForQuery status, got {other:?}"),
    }

    // Unknown enum discriminant: unrecognized Authentication sub-code.
    let mut bad_auth = BytesMut::new();
    use bytes::BufMut;
    bad_auth.put_i32(999);
    let bad_auth_frame = Frame {
        tag: Some(b'R'),
        payload: bad_auth.freeze(),
    };
    match BackendMessage::decode(&bad_auth_frame, &config) {
        Err(FrameError::Malformed { tag, .. }) => assert_eq!(tag, Some(b'R')),
        other => panic!("expected Malformed for unknown auth sub-code, got {other:?}"),
    }

    // Unrecognized tag byte -> UnknownTag.
    let unknown_tag_frame = Frame {
        tag: Some(0xFF),
        payload: Bytes::new(),
    };
    match FrontendMessage::decode(&unknown_tag_frame, &config) {
        Err(FrameError::UnknownTag { tag }) => assert_eq!(tag, 0xFF),
        other => panic!("expected UnknownTag, got {other:?}"),
    }
    let unknown_tag_backend = Frame {
        tag: Some(0xFE),
        payload: Bytes::new(),
    };
    match BackendMessage::decode(&unknown_tag_backend, &config) {
        Err(FrameError::UnknownTag { tag }) => assert_eq!(tag, 0xFE),
        other => panic!("expected UnknownTag, got {other:?}"),
    }

    // Also drive the same malformed cases through FrameReader end-to-end to
    // prove the incremental reader path never panics either.
    let mut reader = FrameReader::new(Role::Backend, &config);
    let mut buf = BytesMut::new();
    buf.extend_from_slice(&[b'Z', 0, 0, 0, 5, b'?']); // length=5 -> 1 payload byte, invalid status
    reader.feed(&buf);
    match reader.next_frame() {
        Err(FrameError::Malformed { .. }) => {}
        other => panic!("expected Malformed via FrameReader, got {other:?}"),
    }
}

// R12: FrameReader reassembles a frame delivered across multiple partial
// reads, split at the header boundary and at arbitrary body offsets.
#[test]
fn reader_handles_split_and_partial_reads() {
    let config = WireCodecConfig::default();

    let row_description = BackendMessage::RowDescription(RowDescription {
        fields: vec![
            FieldDescription {
                name: "id".to_string(),
                table_oid: 16384,
                column_attr: 1,
                type_oid: 23,
                type_size: 4,
                type_modifier: -1,
                format: 0,
            },
            FieldDescription {
                name: "email".to_string(),
                table_oid: 16384,
                column_attr: 2,
                type_oid: 25,
                type_size: -1,
                type_modifier: -1,
                format: 0,
            },
        ],
    });
    let encoded = encode_backend(&row_description);
    assert!(
        encoded.len() > 10,
        "test fixture should be large enough to exercise multiple splits"
    );

    // Split exactly at the header boundary: feed 3 of the 5 header bytes
    // first (tag + partial length), then the rest of the header, then the
    // body in small, arbitrarily-sized chunks.
    let mut reader = FrameReader::new(Role::Backend, &config);
    reader.feed(&encoded[0..3]);
    assert_eq!(
        reader
            .next_frame()
            .expect("no error while header incomplete"),
        None
    );
    reader.feed(&encoded[3..5]);
    assert_eq!(
        reader
            .next_frame()
            .expect("no error once header complete but body pending"),
        None
    );

    let mut offset = 5;
    while offset < encoded.len() - 1 {
        reader.feed(&encoded[offset..offset + 1]);
        assert_eq!(
            reader.next_frame().expect("no error mid-body"),
            None,
            "must stay Pending until the full body is buffered"
        );
        offset += 1;
    }
    reader.feed(&encoded[encoded.len() - 1..]);
    match reader
        .next_frame()
        .expect("decode succeeds once full frame buffered")
    {
        Some(WireMessage::Backend(decoded)) => assert_eq!(decoded, row_description),
        other => panic!("expected the fully reassembled RowDescription, got {other:?}"),
    }

    // Also drive two frames back-to-back split at a single arbitrary
    // midpoint each, proving the reader correctly resumes after emitting.
    let query = FrontendMessage::Query(Query {
        sql: "SELECT 1".to_string(),
    });
    let terminate = FrontendMessage::Terminate(Terminate);
    let mut combined = BytesMut::new();
    combined.extend_from_slice(&encode(&query));
    combined.extend_from_slice(&encode(&terminate));

    let mut reader2 = FrameReader::new(Role::Frontend, &config);
    // Mark this reader as past the untagged-startup phase by driving one
    // real StartupMessage through it first.
    let startup = encode(&FrontendMessage::Startup(StartupMessage {
        protocol_major: 3,
        protocol_minor: 0,
        parameters: vec![],
    }));
    reader2.feed(&startup);
    reader2.next_frame().unwrap().unwrap();

    let midpoint = combined.len() / 2;
    reader2.feed(&combined[0..midpoint]);
    reader2.feed(&combined[midpoint..]);
    let first = reader2.next_frame().unwrap().unwrap();
    assert_eq!(
        first,
        WireMessage::Frontend(FrontendMessage::Query(Query {
            sql: "SELECT 1".to_string()
        }))
    );
    let second = reader2.next_frame().unwrap().unwrap();
    assert_eq!(
        second,
        WireMessage::Frontend(FrontendMessage::Terminate(Terminate))
    );
    assert_eq!(reader2.next_frame().unwrap(), None);
}

// R13: TransactionStatus tracks idle -> in_transaction -> idle and
// idle -> in_transaction -> failed -> idle transitions driven purely by
// observed ReadyForQuery status bytes.
#[test]
fn transaction_status_tracks_simple_and_extended_session() {
    let config = WireCodecConfig::default();
    let mut reader = FrameReader::new(Role::Backend, &config);
    assert_eq!(
        reader.transaction_status(),
        TransactionStatus::Idle,
        "fresh connection starts idle"
    );

    let feed_ready = |reader: &mut FrameReader, status: TransactionStatus| {
        let encoded = encode_backend(&BackendMessage::ReadyForQuery(ReadyForQuery { status }));
        reader.feed(&encoded);
        reader
            .next_frame()
            .expect("ReadyForQuery decodes")
            .expect("frame is complete");
    };

    // Simple-query session: idle -> in_transaction (BEGIN) -> idle (COMMIT).
    feed_ready(&mut reader, TransactionStatus::InTransaction);
    assert_eq!(
        reader.transaction_status(),
        TransactionStatus::InTransaction
    );
    feed_ready(&mut reader, TransactionStatus::Idle);
    assert_eq!(reader.transaction_status(), TransactionStatus::Idle);

    // Extended-query session: idle -> in_transaction -> failed (a statement
    // errors) -> failed (another statement rejected while still failed) ->
    // idle (ROLLBACK completes).
    feed_ready(&mut reader, TransactionStatus::InTransaction);
    assert_eq!(
        reader.transaction_status(),
        TransactionStatus::InTransaction
    );
    feed_ready(&mut reader, TransactionStatus::Failed);
    assert_eq!(reader.transaction_status(), TransactionStatus::Failed);
    feed_ready(&mut reader, TransactionStatus::Failed);
    assert_eq!(reader.transaction_status(), TransactionStatus::Failed);
    feed_ready(&mut reader, TransactionStatus::Idle);
    assert_eq!(reader.transaction_status(), TransactionStatus::Idle);

    // A Query frame between ReadyForQuery observations does not disturb
    // tracked status (only ReadyForQuery drives it).
    let mut reader3 = FrameReader::new(Role::Backend, &config);
    let row = encode_backend(&BackendMessage::CommandComplete(CommandComplete {
        tag: "BEGIN".to_string(),
    }));
    reader3.feed(&row);
    reader3.next_frame().unwrap().unwrap();
    assert_eq!(
        reader3.transaction_status(),
        TransactionStatus::Idle,
        "non-ReadyForQuery frames don't move status"
    );
}
// </HANDWRITE>
// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#unit-test
// CODEGEN-BEGIN

// CODEGEN-END
