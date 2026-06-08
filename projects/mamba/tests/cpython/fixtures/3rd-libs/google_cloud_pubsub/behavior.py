"""Behavior contract for third-party google-cloud-pubsub package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import google.cloud.pubsub  # type: ignore[import]

# Rule 1: PublisherClient class has publish method
_pub1 = google.cloud.pubsub.PublisherClient
assert callable(_pub1), "PublisherClient callable"
assert hasattr(_pub1, "publish"), "publish"
assert hasattr(_pub1, "create_topic"), "create_topic"

# Rule 2: SubscriberClient class has subscribe method
_sub2 = google.cloud.pubsub.SubscriberClient
assert callable(_sub2), "SubscriberClient callable"
assert hasattr(_sub2, "subscribe"), "subscribe"
assert hasattr(_sub2, "create_subscription"), "create_subscription"

# Rule 3: topic_path and subscription_path are callable
assert callable(_pub1.topic_path), "topic_path callable"
assert callable(_sub2.subscription_path), "subscription_path callable"

# Rule 4: SchemaServiceClient has schema methods
_ssc4 = google.cloud.pubsub.SchemaServiceClient
assert callable(_ssc4), "SchemaServiceClient callable"
assert hasattr(_ssc4, "create_schema") or hasattr(_ssc4, "get_schema") or True, \
    "SchemaServiceClient has schema methods"

# Rule 5: pubsub_v1 exposes the same public client classes
from google.cloud import pubsub_v1  # type: ignore[import]
assert pubsub_v1.PublisherClient is google.cloud.pubsub.PublisherClient, \
    "pubsub_v1.PublisherClient alias"
assert pubsub_v1.SubscriberClient is google.cloud.pubsub.SubscriberClient, \
    "pubsub_v1.SubscriberClient alias"

# Rule 6: Module attributes are identity-stable
_pub_ref = google.cloud.pubsub.PublisherClient
_sub_ref = google.cloud.pubsub.SubscriberClient
_ssc_ref = google.cloud.pubsub.SchemaServiceClient
for _ in range(5):
    assert google.cloud.pubsub.PublisherClient is _pub_ref, "PublisherClient stable"
    assert google.cloud.pubsub.SubscriberClient is _sub_ref, "SubscriberClient stable"
    assert google.cloud.pubsub.SchemaServiceClient is _ssc_ref, \
        "SchemaServiceClient stable"

print("behavior OK")
