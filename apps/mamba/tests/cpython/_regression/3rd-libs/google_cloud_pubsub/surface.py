"""Surface contract for third-party google-cloud-pubsub package.

# type-regime: monomorphic

Probes: google.cloud.pubsub.PublisherClient,
google.cloud.pubsub.SubscriberClient, google.cloud.pubsub.SchemaServiceClient.
CPython 3.12 is the oracle.
"""

import google.cloud.pubsub  # type: ignore[import]
from google.cloud import pubsub_v1  # type: ignore[import]

# Core API
assert hasattr(google.cloud.pubsub, "PublisherClient"), "PublisherClient"
assert hasattr(google.cloud.pubsub, "SubscriberClient"), "SubscriberClient"
assert hasattr(google.cloud.pubsub, "SchemaServiceClient"), "SchemaServiceClient"

# Classes are callable
assert callable(google.cloud.pubsub.PublisherClient), "PublisherClient callable"
assert callable(google.cloud.pubsub.SubscriberClient), "SubscriberClient callable"

# pubsub_v1 alias
assert hasattr(pubsub_v1, "PublisherClient"), "v1.PublisherClient"
assert hasattr(pubsub_v1, "SubscriberClient"), "v1.SubscriberClient"

# PublisherClient has expected methods
assert hasattr(google.cloud.pubsub.PublisherClient, "publish"), "publish"
assert hasattr(google.cloud.pubsub.PublisherClient, "create_topic"), "create_topic"
assert hasattr(google.cloud.pubsub.PublisherClient, "topic_path"), "topic_path"

# SubscriberClient has expected methods
assert hasattr(google.cloud.pubsub.SubscriberClient, "subscribe"), "subscribe"
assert hasattr(google.cloud.pubsub.SubscriberClient, "create_subscription"), \
    "create_subscription"
assert hasattr(google.cloud.pubsub.SubscriberClient, "subscription_path"), \
    "subscription_path"

# Module attributes stable
_pub_ref = google.cloud.pubsub.PublisherClient
assert google.cloud.pubsub.PublisherClient is _pub_ref, "PublisherClient stable"
_sub_ref = google.cloud.pubsub.SubscriberClient
assert google.cloud.pubsub.SubscriberClient is _sub_ref, "SubscriberClient stable"
_ssc_ref = google.cloud.pubsub.SchemaServiceClient
assert google.cloud.pubsub.SchemaServiceClient is _ssc_ref, \
    "SchemaServiceClient stable"

print("surface OK")
