// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-pubsub-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Built-in Google Pub/Sub emulator — a tonic gRPC server implementing the
//! google.pubsub.v1 Publisher/Subscriber subset over in-memory state. The
//! client SDKs reach it through `PUBSUB_EMULATOR_HOST`. Faithful enough for
//! local tests of the common path: topic/subscription admin, Publish, Pull,
//! StreamingPull, and Acknowledge.
//!
//! @spec projects/vat/tech-design/logic/built-in-rust-emulators-pub-sub-firebase-auth.md#logic

use std::collections::{HashMap, HashSet, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status, Streaming};

/// Generated from the vendored google.pubsub.v1 proto.
pub mod pb {
    tonic::include_proto!("google.pubsub.v1");
}

use pb::publisher_server::{Publisher, PublisherServer};
use pb::subscriber_server::{Subscriber, SubscriberServer};

#[derive(Default)]
struct Sub {
    topic: String,
    available: VecDeque<pb::PubsubMessage>,
    outstanding: HashMap<String, pb::PubsubMessage>,
}

#[derive(Default)]
struct State {
    topics: HashMap<String, HashSet<String>>,
    subs: HashMap<String, Sub>,
    seq: u64,
}

#[derive(Clone, Default)]
struct PubsubEmulator {
    state: Arc<Mutex<State>>,
}

fn now_ts() -> prost_types::Timestamp {
    let now = chrono::Utc::now();
    prost_types::Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

fn pull_into(state: &Arc<Mutex<State>>, sub: &str, max: usize) -> Vec<pb::ReceivedMessage> {
    let mut s = state.lock().unwrap();
    let taken: Vec<pb::PubsubMessage> = match s.subs.get_mut(sub) {
        Some(entry) => {
            let n = max.min(entry.available.len());
            entry.available.drain(..n).collect()
        }
        None => return Vec::new(),
    };
    let mut out = Vec::with_capacity(taken.len());
    for message in taken {
        s.seq += 1;
        let ack_id = format!("ack-{}", s.seq);
        if let Some(entry) = s.subs.get_mut(sub) {
            entry.outstanding.insert(ack_id.clone(), message.clone());
        }
        out.push(pb::ReceivedMessage {
            ack_id,
            message: Some(message),
            delivery_attempt: 1,
        });
    }
    out
}

fn ack(state: &Arc<Mutex<State>>, sub: &str, ack_ids: &[String]) {
    let mut s = state.lock().unwrap();
    if let Some(entry) = s.subs.get_mut(sub) {
        for id in ack_ids {
            entry.outstanding.remove(id);
        }
    }
}

#[tonic::async_trait]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-pubsub-rs.md#source
impl Publisher for PubsubEmulator {
    async fn create_topic(
        &self,
        request: Request<pb::Topic>,
    ) -> Result<Response<pb::Topic>, Status> {
        let topic = request.into_inner();
        let mut s = self.state.lock().unwrap();
        s.topics.entry(topic.name.clone()).or_default();
        Ok(Response::new(topic))
    }

    async fn get_topic(
        &self,
        request: Request<pb::GetTopicRequest>,
    ) -> Result<Response<pb::Topic>, Status> {
        let name = request.into_inner().topic;
        let s = self.state.lock().unwrap();
        if s.topics.contains_key(&name) {
            Ok(Response::new(pb::Topic {
                name,
                ..Default::default()
            }))
        } else {
            Err(Status::not_found(format!("topic {name} not found")))
        }
    }

    async fn list_topics(
        &self,
        request: Request<pb::ListTopicsRequest>,
    ) -> Result<Response<pb::ListTopicsResponse>, Status> {
        let project = request.into_inner().project;
        let prefix = format!("{project}/topics/");
        let s = self.state.lock().unwrap();
        let topics = s
            .topics
            .keys()
            .filter(|name| name.starts_with(&prefix))
            .map(|name| pb::Topic {
                name: name.clone(),
                ..Default::default()
            })
            .collect();
        Ok(Response::new(pb::ListTopicsResponse {
            topics,
            next_page_token: String::new(),
        }))
    }

    async fn list_topic_subscriptions(
        &self,
        request: Request<pb::ListTopicSubscriptionsRequest>,
    ) -> Result<Response<pb::ListTopicSubscriptionsResponse>, Status> {
        let topic = request.into_inner().topic;
        let s = self.state.lock().unwrap();
        let subscriptions = s
            .topics
            .get(&topic)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();
        Ok(Response::new(pb::ListTopicSubscriptionsResponse {
            subscriptions,
            next_page_token: String::new(),
        }))
    }

    async fn delete_topic(
        &self,
        request: Request<pb::DeleteTopicRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().topic;
        let mut s = self.state.lock().unwrap();
        s.topics.remove(&name);
        Ok(Response::new(()))
    }

    async fn publish(
        &self,
        request: Request<pb::PublishRequest>,
    ) -> Result<Response<pb::PublishResponse>, Status> {
        let req = request.into_inner();
        let mut s = self.state.lock().unwrap();
        let subs: Vec<String> = s
            .topics
            .entry(req.topic.clone())
            .or_default()
            .iter()
            .cloned()
            .collect();
        let mut message_ids = Vec::with_capacity(req.messages.len());
        for mut message in req.messages {
            s.seq += 1;
            let id = format!("msg-{}", s.seq);
            message.message_id = id.clone();
            message.publish_time = Some(now_ts());
            for sub in &subs {
                if let Some(entry) = s.subs.get_mut(sub) {
                    entry.available.push_back(message.clone());
                }
            }
            message_ids.push(id);
        }
        Ok(Response::new(pb::PublishResponse { message_ids }))
    }
}

#[tonic::async_trait]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-pubsub-rs.md#source
impl Subscriber for PubsubEmulator {
    type StreamingPullStream =
        Pin<Box<dyn Stream<Item = Result<pb::StreamingPullResponse, Status>> + Send + 'static>>;

    async fn create_subscription(
        &self,
        request: Request<pb::Subscription>,
    ) -> Result<Response<pb::Subscription>, Status> {
        let sub = request.into_inner();
        let mut s = self.state.lock().unwrap();
        s.topics
            .entry(sub.topic.clone())
            .or_default()
            .insert(sub.name.clone());
        s.subs.entry(sub.name.clone()).or_insert_with(|| Sub {
            topic: sub.topic.clone(),
            ..Default::default()
        });
        Ok(Response::new(sub))
    }

    async fn get_subscription(
        &self,
        request: Request<pb::GetSubscriptionRequest>,
    ) -> Result<Response<pb::Subscription>, Status> {
        let name = request.into_inner().subscription;
        let s = self.state.lock().unwrap();
        match s.subs.get(&name) {
            Some(entry) => Ok(Response::new(pb::Subscription {
                name,
                topic: entry.topic.clone(),
                ack_deadline_seconds: 10,
            })),
            None => Err(Status::not_found(format!("subscription {name} not found"))),
        }
    }

    async fn list_subscriptions(
        &self,
        request: Request<pb::ListSubscriptionsRequest>,
    ) -> Result<Response<pb::ListSubscriptionsResponse>, Status> {
        let project = request.into_inner().project;
        let prefix = format!("{project}/subscriptions/");
        let s = self.state.lock().unwrap();
        let subscriptions = s
            .subs
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(name, entry)| pb::Subscription {
                name: name.clone(),
                topic: entry.topic.clone(),
                ack_deadline_seconds: 10,
            })
            .collect();
        Ok(Response::new(pb::ListSubscriptionsResponse {
            subscriptions,
            next_page_token: String::new(),
        }))
    }

    async fn delete_subscription(
        &self,
        request: Request<pb::DeleteSubscriptionRequest>,
    ) -> Result<Response<()>, Status> {
        let name = request.into_inner().subscription;
        let mut s = self.state.lock().unwrap();
        if let Some(entry) = s.subs.remove(&name) {
            if let Some(set) = s.topics.get_mut(&entry.topic) {
                set.remove(&name);
            }
        }
        Ok(Response::new(()))
    }

    async fn modify_ack_deadline(
        &self,
        request: Request<pb::ModifyAckDeadlineRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        // Deadline 0 is a nack: make the message available for redelivery.
        if req.ack_deadline_seconds == 0 {
            let mut s = self.state.lock().unwrap();
            if let Some(entry) = s.subs.get_mut(&req.subscription) {
                for id in &req.ack_ids {
                    if let Some(message) = entry.outstanding.remove(id) {
                        entry.available.push_back(message);
                    }
                }
            }
        }
        Ok(Response::new(()))
    }

    async fn acknowledge(
        &self,
        request: Request<pb::AcknowledgeRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        ack(&self.state, &req.subscription, &req.ack_ids);
        Ok(Response::new(()))
    }

    async fn pull(
        &self,
        request: Request<pb::PullRequest>,
    ) -> Result<Response<pb::PullResponse>, Status> {
        let req = request.into_inner();
        let max = if req.max_messages > 0 {
            req.max_messages as usize
        } else {
            100
        };
        let received_messages = pull_into(&self.state, &req.subscription, max);
        Ok(Response::new(pb::PullResponse { received_messages }))
    }

    async fn streaming_pull(
        &self,
        request: Request<Streaming<pb::StreamingPullRequest>>,
    ) -> Result<Response<Self::StreamingPullStream>, Status> {
        let mut inbound = request.into_inner();
        let state = self.state.clone();
        let stream = async_stream::stream! {
            let mut subscription = String::new();
            loop {
                tokio::select! {
                    incoming = inbound.message() => {
                        match incoming {
                            Ok(Some(req)) => {
                                if !req.subscription.is_empty() {
                                    subscription = req.subscription.clone();
                                }
                                if !req.ack_ids.is_empty() {
                                    ack(&state, &subscription, &req.ack_ids);
                                }
                            }
                            Ok(None) => break,
                            Err(status) => {
                                yield Err(status);
                                break;
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {}
                }
                if !subscription.is_empty() {
                    let received_messages = pull_into(&state, &subscription, 100);
                    if !received_messages.is_empty() {
                        yield Ok(pb::StreamingPullResponse { received_messages });
                    }
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}

/// Serve the Pub/Sub emulator until the process is killed.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-pubsub-rs.md#source
pub async fn serve(host_port: &str) -> Result<()> {
    let addr = host_port
        .parse()
        .with_context(|| format!("parse pubsub emulator address {host_port}"))?;
    let emulator = PubsubEmulator::default();
    Server::builder()
        .add_service(PublisherServer::new(emulator.clone()))
        .add_service(SubscriberServer::new(emulator))
        .serve(addr)
        .await
        .context("serve pubsub emulator")?;
    Ok(())
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
