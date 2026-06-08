"""PostGIS spatial database support."""
from typing import Optional

__all__ = [
    "Point",
    "GeoQuery",
]


class Point:
    """PostGIS Point geometry."""

    def __init__(self, lng: float, lat: float, srid: int = 4326):
        """Create a Point.

        Args:
            lng: Longitude
            lat: Latitude
            srid: Spatial Reference System ID (default: 4326 for WGS84)
        """
        self.lng = lng
        self.lat = lat
        self.srid = srid

    def to_sql(self) -> str:
        """Convert to SQL expression.

        Returns:
            SQL for creating PostGIS point

        Example:
            >>> Point(121.5, 25.0).to_sql()
            "ST_SetSRID(ST_MakePoint(121.5, 25.0), 4326)"
        """
        return f"ST_SetSRID(ST_MakePoint({self.lng}, {self.lat}), {self.srid})"

    @staticmethod
    def from_wkt(wkt: str, srid: int = 4326) -> str:
        """Create point from Well-Known Text.

        Args:
            wkt: WKT string (e.g., "POINT(121.5 25.0)")
            srid: Spatial Reference System ID

        Returns:
            SQL expression
        """
        return f"ST_GeomFromText('{wkt}', {srid})"

    def __repr__(self) -> str:
        return f"Point(lng={self.lng}, lat={self.lat}, srid={self.srid})"


class GeoQuery:
    """PostGIS query helpers."""

    @staticmethod
    def distance(geom1: str, geom2: str) -> str:
        """Calculate distance between geometries.

        Args:
            geom1: First geometry column/expression
            geom2: Second geometry column/expression

        Returns:
            SQL expression for ST_Distance
        """
        return f"ST_Distance({geom1}, {geom2})"

    @staticmethod
    def dwithin(geom1: str, geom2: str, distance: float) -> str:
        """Check if geometries are within distance.

        Args:
            geom1: First geometry column/expression
            geom2: Second geometry column/expression
            distance: Maximum distance in meters

        Returns:
            SQL WHERE clause
        """
        return f"ST_DWithin({geom1}, {geom2}, {distance})"

    @staticmethod
    def contains(geom1: str, geom2: str) -> str:
        """Check if geom1 contains geom2.

        Returns:
            SQL WHERE clause
        """
        return f"ST_Contains({geom1}, {geom2})"

    @staticmethod
    def within(geom1: str, geom2: str) -> str:
        """Check if geom1 is within geom2.

        Returns:
            SQL WHERE clause
        """
        return f"ST_Within({geom1}, {geom2})"

    @staticmethod
    def intersects(geom1: str, geom2: str) -> str:
        """Check if geometries intersect.

        Returns:
            SQL WHERE clause
        """
        return f"ST_Intersects({geom1}, {geom2})"
