"""
Tests for MongoDB time-series collection support.

Time-series collections require MongoDB 5.0+ and are optimized for storing
sequences of measurements over time.

Migrated from pytest to cclab.probe framework.
"""
from datetime import datetime, timezone, timedelta
from typing import Optional

from cclab import Document
from cclab.nebula.timeseries import TimeSeriesConfig, Granularity
from cclab.probe import test, expect
from tests.base import MongoTestSuite, CommonTestSuite


class TestTimeSeriesConfig(CommonTestSuite):
    """Tests for TimeSeriesConfig class."""

    @test(tags=["unit", "timeseries"])
    async def test_basic_config(self):
        """Test creating a basic time-series config."""
        config = TimeSeriesConfig(time_field="timestamp")
        expect(config.time_field).to_equal("timestamp")
        expect(config.meta_field).to_be_none()
        expect(config.granularity).to_be_none()

    @test(tags=["unit", "timeseries"])
    async def test_full_config(self):
        """Test creating a fully configured time-series config."""
        config = TimeSeriesConfig(
            time_field="timestamp",
            meta_field="sensor_id",
            granularity=Granularity.seconds,
            expire_after_seconds=86400,
        )
        expect(config.time_field).to_equal("timestamp")
        expect(config.meta_field).to_equal("sensor_id")
        expect(config.granularity).to_equal(Granularity.seconds)
        expect(config.expire_after_seconds).to_equal(86400)

    @test(tags=["unit", "timeseries"])
    async def test_granularity_enum(self):
        """Test Granularity enum values."""
        expect(Granularity.seconds.value).to_equal("seconds")
        expect(Granularity.minutes.value).to_equal("minutes")
        expect(Granularity.hours.value).to_equal("hours")

    @test(tags=["unit", "timeseries"])
    async def test_to_create_options_basic(self):
        """Test converting basic config to MongoDB options."""
        config = TimeSeriesConfig(time_field="ts")
        options = config.to_create_options()
        expect(options).to_equal({"timeseries": {"timeField": "ts"}})

    @test(tags=["unit", "timeseries"])
    async def test_to_create_options_full(self):
        """Test converting full config to MongoDB options."""
        config = TimeSeriesConfig(
            time_field="timestamp",
            meta_field="device_id",
            granularity=Granularity.minutes,
            expire_after_seconds=3600,
        )
        options = config.to_create_options()
        expected = {
            "timeseries": {
                "timeField": "timestamp",
                "metaField": "device_id",
                "granularity": "minutes",
            },
            "expireAfterSeconds": 3600,
        }
        expect(options).to_equal(expected)

    @test(tags=["unit", "timeseries"])
    async def test_repr(self):
        """Test string representation."""
        config = TimeSeriesConfig(
            time_field="ts",
            meta_field="sensor",
            granularity=Granularity.seconds,
        )
        repr_str = repr(config)
        expect("time_field='ts'" in repr_str).to_be_true()
        expect("meta_field='sensor'" in repr_str).to_be_true()
        expect("granularity='seconds'" in repr_str).to_be_true()


class TestTimeSeriesDocument(MongoTestSuite):
    """Tests for time-series Document functionality."""

    @test(tags=["unit", "timeseries"])
    async def test_document_with_timeseries_settings(self):
        """Test defining a document with time-series settings."""
        class SensorReading(Document):
            sensor_id: str
            timestamp: datetime
            temperature: float

            class Settings:
                name = "sensor_readings_test1"
                timeseries = TimeSeriesConfig(
                    time_field="timestamp",
                    meta_field="sensor_id",
                    granularity=Granularity.seconds,
                )

        expect(SensorReading.is_timeseries()).to_be_true()
        config = SensorReading.get_timeseries_config()
        expect(config).not_.to_be_none()
        expect(config.time_field).to_equal("timestamp")
        expect(config.meta_field).to_equal("sensor_id")
        expect(config.granularity).to_equal(Granularity.seconds)

    @test(tags=["unit", "timeseries"])
    async def test_non_timeseries_document(self):
        """Test that regular documents don't have time-series config."""
        class RegularDoc(Document):
            name: str

            class Settings:
                name = "regular_docs"

        expect(RegularDoc.is_timeseries()).to_be_false()
        expect(RegularDoc.get_timeseries_config()).to_be_none()

    @test(tags=["mongo", "timeseries"])
    async def test_ensure_timeseries_collection(self):
        """Test creating a time-series collection via ensure_timeseries_collection."""
        class MetricData(Document):
            metric_name: str
            timestamp: datetime
            value: float

            class Settings:
                name = "metrics_test_ensure"
                timeseries = TimeSeriesConfig(
                    time_field="timestamp",
                    meta_field="metric_name",
                    granularity=Granularity.minutes,
                )

        result = await MetricData.ensure_timeseries_collection()
        expect(isinstance(result, bool)).to_be_true()

        result2 = await MetricData.ensure_timeseries_collection()
        expect(result2).to_be_false()

    @test(tags=["mongo", "timeseries"])
    async def test_insert_timeseries_document(self):
        """Test inserting a document into a time-series collection."""
        class TempReading(Document):
            location: str
            timestamp: datetime
            temperature: float
            humidity: Optional[float] = None

            class Settings:
                name = "temp_readings_test"
                timeseries = TimeSeriesConfig(
                    time_field="timestamp",
                    meta_field="location",
                    granularity=Granularity.seconds,
                )

        await TempReading.ensure_timeseries_collection()

        now = datetime.now(timezone.utc)
        reading = TempReading(
            location="office",
            timestamp=now,
            temperature=22.5,
            humidity=45.0,
        )
        await reading.save()

        expect(reading._id).not_.to_be_none()

        found = await TempReading.find_one(TempReading.location == "office")
        expect(found).not_.to_be_none()
        expect(found.temperature).to_equal(22.5)
        expect(found.humidity).to_equal(45.0)

    @test(tags=["mongo", "timeseries"])
    async def test_query_timeseries_with_time_range(self):
        """Test querying time-series data with time range filters."""
        class EventLog(Document):
            event_type: str
            timestamp: datetime
            message: str

            class Settings:
                name = "event_logs_test"
                timeseries = TimeSeriesConfig(
                    time_field="timestamp",
                    meta_field="event_type",
                    granularity=Granularity.seconds,
                )

        await EventLog.ensure_timeseries_collection()

        from cclab.nebula import _engine
        await _engine.delete_many("event_logs_test", {})

        now = datetime.now(timezone.utc)
        events = [
            EventLog(
                event_type="info",
                timestamp=now - timedelta(hours=2),
                message="Event 1",
            ),
            EventLog(
                event_type="warning",
                timestamp=now - timedelta(hours=1),
                message="Event 2",
            ),
            EventLog(
                event_type="error",
                timestamp=now,
                message="Event 3",
            ),
        ]

        for event in events:
            await event.save()

        cutoff = now - timedelta(hours=1, minutes=30)
        recent_events = await EventLog.find(
            EventLog.timestamp >= cutoff
        ).to_list()

        expect(len(recent_events)).to_equal(2)

    @test(tags=["mongo", "timeseries"])
    async def test_timeseries_with_ttl(self):
        """Test time-series collection with TTL expiration."""
        class ShortLivedMetric(Document):
            name: str
            timestamp: datetime
            value: float

            class Settings:
                name = "short_lived_metrics_test"
                timeseries = TimeSeriesConfig(
                    time_field="timestamp",
                    meta_field="name",
                    granularity=Granularity.seconds,
                    expire_after_seconds=3600,
                )

        config = ShortLivedMetric.get_timeseries_config()
        expect(config.expire_after_seconds).to_equal(3600)

        await ShortLivedMetric.ensure_timeseries_collection()

        metric = ShortLivedMetric(
            name="cpu_usage",
            timestamp=datetime.now(timezone.utc),
            value=75.5,
        )
        await metric.save()
        expect(metric._id).not_.to_be_none()


class TestGranularity(CommonTestSuite):
    """Tests for Granularity enum."""

    @test(tags=["unit", "timeseries"])
    async def test_seconds_granularity(self):
        """Test seconds granularity value."""
        expect(Granularity.seconds.value).to_equal("seconds")

    @test(tags=["unit", "timeseries"])
    async def test_minutes_granularity(self):
        """Test minutes granularity value."""
        expect(Granularity.minutes.value).to_equal("minutes")

    @test(tags=["unit", "timeseries"])
    async def test_hours_granularity(self):
        """Test hours granularity value."""
        expect(Granularity.hours.value).to_equal("hours")

    @test(tags=["unit", "timeseries"])
    async def test_granularity_is_str_enum(self):
        """Test that Granularity can be used as string."""
        config = TimeSeriesConfig(
            time_field="ts",
            granularity=Granularity.minutes,
        )
        options = config.to_create_options()
        expect(options["timeseries"]["granularity"]).to_equal("minutes")


# Run tests when executed directly
if __name__ == "__main__":
    from cclab.probe import run_suites

    run_suites([
        TestTimeSeriesConfig,
        TestTimeSeriesDocument,
        TestGranularity,
    ], verbose=True)
