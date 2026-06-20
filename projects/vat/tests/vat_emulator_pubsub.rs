// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-tests.md#schema
// CODEGEN-BEGIN
//! Self-contained integration test for the built-in Pub/Sub gRPC emulator.
//! Spawns `vat emulator pubsub` and drives CreateTopic -> CreateSubscription ->
//! Publish -> Pull -> Acknowledge (and a StreamingPull round-trip) with a tonic
//! client generated from the same proto — no gcloud/Java required.

use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use vat::emulator::pubsub::pb;
use vat::emulator::pubsub::pb::publisher_client::PublisherClient;
use vat::emulator::pubsub::pb::subscriber_client::SubscriberClient;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_port(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("emulator did not open {addr}");
}

struct Killed(Child);
impl Drop for Killed {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

const TOPIC: &str = "projects/demo-vat/topics/t";
const SUB: &str = "projects/demo-vat/subscriptions/s";

#[tokio::test]
async fn pubsub_emulator_publish_pull_ack_and_stream() {
    let port = free_port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(vat_bin())
        .args(["emulator", "pubsub", "--host-port", &addr])
        .spawn()
        .expect("spawn pubsub emulator");
    let _guard = Killed(child);
    wait_for_port(&addr);

    let endpoint = format!("http://{addr}");
    let mut publisher = PublisherClient::connect(endpoint.clone()).await.unwrap();
    let mut subscriber = SubscriberClient::connect(endpoint).await.unwrap();

    publisher
        .create_topic(pb::Topic {
            name: TOPIC.into(),
            ..Default::default()
        })
        .await
        .unwrap();
    subscriber
        .create_subscription(pb::Subscription {
            name: SUB.into(),
            topic: TOPIC.into(),
            ack_deadline_seconds: 10,
        })
        .await
        .unwrap();

    // Publish -> Pull -> Acknowledge.
    publisher
        .publish(pb::PublishRequest {
            topic: TOPIC.into(),
            messages: vec![pb::PubsubMessage {
                data: b"hello".to_vec(),
                ..Default::default()
            }],
        })
        .await
        .unwrap();

    let pulled = subscriber
        .pull(pb::PullRequest {
            subscription: SUB.into(),
            max_messages: 10,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(pulled.received_messages.len(), 1);
    let received = &pulled.received_messages[0];
    assert_eq!(received.message.as_ref().unwrap().data, b"hello");
    assert!(!received.message.as_ref().unwrap().message_id.is_empty());

    subscriber
        .acknowledge(pb::AcknowledgeRequest {
            subscription: SUB.into(),
            ack_ids: vec![received.ack_id.clone()],
        })
        .await
        .unwrap();

    let empty = subscriber
        .pull(pb::PullRequest {
            subscription: SUB.into(),
            max_messages: 10,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(empty.received_messages.is_empty(), "acked message is gone");

    // StreamingPull round-trip: keep the request stream open while a publish
    // lands, then read one streamed message.
    let (tx, rx) = tokio::sync::mpsc::channel(4);
    tx.send(pb::StreamingPullRequest {
        subscription: SUB.into(),
        stream_ack_deadline_seconds: 10,
        ..Default::default()
    })
    .await
    .unwrap();
    let mut stream = subscriber
        .streaming_pull(tokio_stream::wrappers::ReceiverStream::new(rx))
        .await
        .unwrap()
        .into_inner();

    publisher
        .publish(pb::PublishRequest {
            topic: TOPIC.into(),
            messages: vec![pb::PubsubMessage {
                data: b"stream-hello".to_vec(),
                ..Default::default()
            }],
        })
        .await
        .unwrap();

    let response = tokio::time::timeout(Duration::from_secs(5), stream.message())
        .await
        .expect("streaming pull timed out")
        .unwrap()
        .unwrap();
    assert_eq!(
        response.received_messages[0].message.as_ref().unwrap().data,
        b"stream-hello"
    );
    drop(tx);
}
// CODEGEN-END
