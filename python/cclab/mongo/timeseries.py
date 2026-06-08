"""
Time-series collection support for MongoDB 5.0+.

This module provides Beanie-compatible configuration for MongoDB time-series collections.

Example:
    >>> from cclab.mongo import Document
    >>> from cclab.timeseries import TimeSeriesConfig, Granularity
    >>>
    >>> class SensorReading(Document):
    ...     sensor_id: str
    ...     timestamp: datetime
    ...     temperature: float
    ...     humidity: float
    ...
    ...     class Settings:
    ...         name = "sensor_readings"
    ...         timeseries = TimeSeriesConfig(
    ...             time_field="timestamp",
    ...             meta_field="sensor_id",
    ...             granularity=Granularity.minutes,
    ...             expire_after_seconds=86400 * 30,  # 30 days
    ...         )
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class Granularity(str, Enum):
    """
    Time-series collection granularity.

    Controls how MongoDB optimizes storage and indexing for time-series data.
    Choose based on the expected frequency of your measurements:
    - seconds: Data points every few seconds (high-frequency sensors)
    - minutes: Data points every few minutes (metrics, monitoring)
    - hours: Data points every few hours or less (daily summaries, batch data)
    """
    seconds = "seconds"
    minutes = "minutes"
    hours = "hours"


@dataclass
class TimeSeriesConfig:
    """
    Configuration for MongoDB time-series collections.

    Time-series collections are optimized for storing sequences of measurements
    over time, providing better storage efficiency and query performance.

    Attributes:
        time_field: Name of the field containing the measurement timestamp.
            This field must be present in every document and contain a datetime.
        meta_field: Optional field containing metadata that identifies the
            data source (e.g., sensor ID, device name). Documents with the
            same meta_field value are grouped together.
        granularity: Hint for how frequently data is expected to be inserted.
            Helps MongoDB optimize internal bucketing.
        bucket_max_span_seconds: Maximum time span for a bucket in seconds.
            Requires MongoDB 6.3+. Range: 1-31536000 (1 second to 1 year).
        bucket_rounding_seconds: Bucket boundary rounding in seconds.
            Requires MongoDB 6.3+.
        expire_after_seconds: Automatic document expiration. Documents older
            than this many seconds will be automatically deleted (TTL).

    Example:
        >>> config = TimeSeriesConfig(
        ...     time_field="timestamp",
        ...     meta_field="device_id",
        ...     granularity=Granularity.seconds,
        ...     expire_after_seconds=3600 * 24 * 7,  # 7 days
        ... )
    """

    time_field: str
    meta_field: Optional[str] = None
    granularity: Optional[Granularity] = None
    bucket_max_span_seconds: Optional[int] = None  # MongoDB 6.3+
    bucket_rounding_seconds: Optional[int] = None  # MongoDB 6.3+
    expire_after_seconds: Optional[int] = None

    def to_create_options(self) -> dict:
        """
        Convert to MongoDB createCollection options.

        Returns:
            Dict suitable for MongoDB createCollection command.
        """
        timeseries = {"timeField": self.time_field}

        if self.meta_field:
            timeseries["metaField"] = self.meta_field

        if self.granularity:
            timeseries["granularity"] = self.granularity.value

        if self.bucket_max_span_seconds is not None:
            timeseries["bucketMaxSpanSeconds"] = self.bucket_max_span_seconds

        if self.bucket_rounding_seconds is not None:
            timeseries["bucketRoundingSeconds"] = self.bucket_rounding_seconds

        options = {"timeseries": timeseries}

        if self.expire_after_seconds is not None:
            options["expireAfterSeconds"] = self.expire_after_seconds

        return options

    def __repr__(self) -> str:
        parts = [f"time_field={self.time_field!r}"]
        if self.meta_field:
            parts.append(f"meta_field={self.meta_field!r}")
        if self.granularity:
            parts.append(f"granularity={self.granularity.value!r}")
        if self.expire_after_seconds:
            parts.append(f"expire_after_seconds={self.expire_after_seconds}")
        return f"TimeSeriesConfig({', '.join(parts)})"


__all__ = ["TimeSeriesConfig", "Granularity"]
