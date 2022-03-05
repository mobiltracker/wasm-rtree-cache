use std::convert::TryFrom;

use geo::{
    prelude::{ClosestPoint, Contains, HaversineDestination, HaversineDistance},
    Coordinate, Line, Point, Rect,
};
use rstar::primitives::{GeomWithData, Rectangle};
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Debug)]
pub struct Place(pub PlaceWithAddress);
type PlaceWithAddress = GeomWithData<Rectangle<(f64, f64)>, String>;

#[derive(Debug)]
pub struct CoordinateCache {
    inner: rstar::RTree<PlaceWithAddress>,
    float_precision: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    // Bouding box corner points
    pub south_west: Coordinate<f64>,
    pub south_east: Coordinate<f64>,
    pub north_west: Coordinate<f64>,
    pub north_east: Coordinate<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct PointBoundingBox {
    // Bouding box corner points
    pub south_west: Point<f64>,
    pub south_east: Point<f64>,
    pub north_west: Point<f64>,
    pub north_east: Point<f64>,
}

#[derive(Debug)]
pub struct SetNotChanged {
    pub area_meters: f64,
    pub bbox: PointBoundingBox,
    pub width: f64,
    pub height: f64,
    pub is_missing_reference_point: bool,
}

#[derive(Debug)]
pub struct SetTruncated {
    pub old_area_meters: f64,
    pub new_area_meters: f64,
    pub old_bbox: PointBoundingBox,
    pub new_bbox: PointBoundingBox,
    pub old_height: f64,
    pub new_height: f64,
    pub old_width: f64,
    pub new_width: f64,
    pub is_missing_reference_point: bool,
}

#[derive(Debug)]
pub enum BoundingBoxSetResult {
    SetNotChanged(SetNotChanged),
    SetTruncated(SetTruncated),
}

#[derive(Debug)]
pub struct BoundingBoxConversionError {
    _bounding_box: Vec<f64>,
}

impl std::fmt::Display for BoundingBoxConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bounding box conversion error {:?}", self)
    }
}
impl std::error::Error for BoundingBoxConversionError {}

impl From<BoundingBox> for PointBoundingBox {
    fn from(b: BoundingBox) -> Self {
        Self {
            north_east: b.north_east.into(),
            north_west: b.north_west.into(),
            south_east: b.south_east.into(),
            south_west: b.south_west.into(),
        }
    }
}

impl CoordinateCache {
    pub fn new() -> Self {
        Self {
            inner: rstar::RTree::default(),
            float_precision: 5,
        }
    }

    pub fn clear(&mut self) {
        *self = Self {
            inner: rstar::RTree::default(),
            float_precision: self.float_precision,
        };
    }

    pub fn new_with_precision(float_precision: u8) -> Self {
        Self {
            inner: rstar::RTree::default(),
            float_precision,
        }
    }

    pub fn set(
        &mut self,
        data: String,
        bbox: BoundingBox,
        reference_point: Option<Coordinate<f64>>,
    ) -> SetNotChanged {
        let bbox = truncate_bounding_box(bbox, self.float_precision);
        let reference_point = reference_point.map(|c| truncate_coordinate(c, self.float_precision));
        let bbox = PointBoundingBox::from(bbox);
        let place = Place::new(bbox.north_west, bbox.south_east, data);
        let width = bbox.north_east.haversine_distance(&bbox.north_west);
        let height = bbox.north_east.haversine_distance(&bbox.south_east);

        let rect = Rect::new(bbox.north_west, bbox.south_east);

        let is_missing_reference_point = reference_point
            .map(|c| !rect.contains(&Point::from(c)))
            .unwrap_or(false);

        self.inner.insert(place.0);

        SetNotChanged {
            area_meters: width * height,
            bbox,
            width,
            height,
            is_missing_reference_point,
        }
    }

    pub fn set_with_max_len(
        &mut self,
        data: String,
        bbox: BoundingBox,
        reference_point: Coordinate<f64>,
        max_side_len_meters: Option<f64>,
    ) -> BoundingBoxSetResult {
        let reference_point: Point<f64> = reference_point.into();
        let bbox = truncate_bounding_box(bbox, self.float_precision);
        let reference_point =
            Point::from(truncate_coordinate(reference_point.0, self.float_precision));

        let bbox = PointBoundingBox::from(bbox);

        let width = bbox.north_east.haversine_distance(&bbox.north_west);
        let height = bbox.north_east.haversine_distance(&bbox.south_east);

        let rect = Rect::new(bbox.north_west, bbox.south_east);
        let is_missing_reference_point = !rect.contains(&reference_point);

        match max_side_len_meters {
            Some(max_len) if width > max_len || height > max_len => {
                let new_bbox = Self::fix_rect(bbox, max_len, reference_point, self.float_precision);
                let place = Place::new(new_bbox.north_west, new_bbox.south_east, data);
                self.inner.insert(place.0);

                let new_width = new_bbox.north_east.haversine_distance(&new_bbox.north_west);
                let new_height = new_bbox.north_east.haversine_distance(&new_bbox.south_east);

                BoundingBoxSetResult::SetTruncated(SetTruncated {
                    new_area_meters: new_height * new_width,
                    new_bbox,
                    old_bbox: bbox,
                    old_area_meters: width * height,
                    new_height,
                    new_width,
                    old_height: height,
                    old_width: width,
                    is_missing_reference_point,
                })
            }
            _ => {
                let place = Place::new(bbox.north_west, bbox.south_east, data);
                self.inner.insert(place.0);
                BoundingBoxSetResult::SetNotChanged(SetNotChanged {
                    area_meters: width * height,
                    bbox,
                    height,
                    width,
                    is_missing_reference_point,
                })
            }
        }
    }

    fn fix_rect(
        bbox: PointBoundingBox,
        max_len_side: f64,
        reference: Point<f64>,
        float_precision: u8,
    ) -> PointBoundingBox {
        let rect = Rect::new(bbox.north_west, bbox.south_east);

        let mut reference = reference;
        // If reference point is not inside the container, set reference point as the rectangle centroid so if doesn't do anything too weird
        if !rect.contains(&reference) {
            // Set reference point as rectangle center
            reference = rect.center().into();
        }

        let top = Line::new(bbox.north_west, bbox.north_east);
        let bottom = Line::new(bbox.south_west, bbox.south_east);
        let left = Line::new(bbox.north_west, bbox.south_west);
        let right = Line::new(bbox.north_east, bbox.south_east);
        let max_len = max_len_side / 2.0;

        // Calculate distance to the closest point of the rectangle perimeter
        // P is the reference point, arrow length represents distance_to_{left/top/right/bottom}
        //  -----------------------
        //  |    ^                |
        //  |    |                |
        //  | <--p -------------> |
        //  |    |                |
        //  |    v                |
        //  -----------------------

        let distance_to_top = Self::closest_point(&top, &reference).haversine_distance(&reference);
        let distance_to_bottom =
            Self::closest_point(&bottom, &reference).haversine_distance(&reference);

        let distance_to_left =
            Self::closest_point(&left, &reference).haversine_distance(&reference);
        let distance_to_right =
            Self::closest_point(&right, &reference).haversine_distance(&reference);

        // New points from max of distance_to_x and max_len
        //  -----N-------
        //  |    ^      |
        //  |    |      |
        //  L <--p ---> R   <= in this example, right side is truncated to max_len, everything else is the same
        //  |    |      |
        //  |    v      |
        //  -----S-------

        // bearing to another Point in degrees, where North is 0° and East is 90°.
        let fixed_north_point =
            reference.haversine_destination(0.0f64, distance_to_top.min(max_len));

        let fixed_east_point =
            reference.haversine_destination(90.0f64, distance_to_right.min(max_len));

        let fixed_south_point =
            reference.haversine_destination(180.0f64, distance_to_bottom.min(max_len));

        let fixed_west_point =
            reference.haversine_destination(270.0f64, distance_to_left.min(max_len));

        // Calculate Rectangle corners using N,S,W,E x and y positions
        // NW----N------NE
        //  |    ^      |
        //  |    |      |
        //  W <--p ---> E
        //  |    |      |
        //  |    v      |
        // SW----S------SE

        let fixed_northwest_point = Point::new(fixed_west_point.x(), fixed_north_point.y());
        let fixed_southwest_point = Point::new(fixed_west_point.x(), fixed_south_point.y());

        let fixed_northeast_point = Point::new(fixed_east_point.x(), fixed_north_point.y());
        let fixed_southeast_point = Point::new(fixed_east_point.x(), fixed_south_point.y());

        truncate_point_bounding_box(
            PointBoundingBox {
                south_west: fixed_southwest_point,
                south_east: fixed_southeast_point,
                north_west: fixed_northwest_point,
                north_east: fixed_northeast_point,
            },
            float_precision,
        )
    }

    fn closest_point(line: &Line<f64>, point: &Point<f64>) -> Point<f64> {
        match line.closest_point(point) {
            geo::Closest::Intersection(i) => i,
            geo::Closest::SinglePoint(s) => s,
            geo::Closest::Indeterminate => point.to_owned(),
        }
    }

    pub fn get(&self, coordinate: Coordinate<f64>) -> Option<String> {
        let coordinate = truncate_coordinate(coordinate, self.float_precision);
        let mut places_containing_point = self.inner.locate_all_at_point(&coordinate.x_y());
        let first = places_containing_point.next();
        let second = places_containing_point.next();

        // If we have only a single point, return data w/o any extra allocations
        if second.is_none() {
            first.map(|f| f.data.clone())
        } else {
            // We have more than a single point
            let mut places = places_containing_point.collect::<Vec<_>>();

            if let Some(first) = first {
                places.push(first);
            };

            if let Some(second) = second {
                places.push(second);
            };

            // Sort by distance from point to rectangle center
            places.sort_by(|a, b| {
                let rect_a_center: Point<f64> = geo::Rect::new(a.geom().upper(), a.geom().lower())
                    .center()
                    .into();
                let rect_b_center: Point<f64> = geo::Rect::new(b.geom().upper(), b.geom().lower())
                    .center()
                    .into();

                rect_a_center
                    .haversine_distance(&Point::from(coordinate))
                    .partial_cmp(&rect_b_center.haversine_distance(&Point::from(coordinate)))
                    .unwrap()
            });

            places.first().map(|f| f.data.clone())
        }
    }
}

impl Default for CoordinateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Place {
    pub fn new(north_west: Point<f64>, south_east: Point<f64>, name: String) -> Self {
        let rect = Rectangle::from_corners(north_west.x_y(), south_east.x_y());
        let geom = GeomWithData::new(rect, name);

        Place(geom)
    }
}

#[inline(always)]
pub fn truncate_float(value: f64, decimal_places: u8) -> f64 {
    let power_of_10 = 10.0f64.powi(decimal_places.into());
    (value * power_of_10).round() / power_of_10
}

pub fn truncate_coordinate(coordinate: Coordinate<f64>, decimal_places: u8) -> Coordinate<f64> {
    Coordinate {
        x: truncate_float(coordinate.x, decimal_places),
        y: truncate_float(coordinate.y, decimal_places),
    }
}

pub fn truncate_bounding_box(bbox: BoundingBox, decimal_places: u8) -> BoundingBox {
    BoundingBox {
        north_west: truncate_coordinate(bbox.north_west, decimal_places),
        north_east: truncate_coordinate(bbox.north_east, decimal_places),
        south_east: truncate_coordinate(bbox.south_east, decimal_places),
        south_west: truncate_coordinate(bbox.south_west, decimal_places),
    }
}

pub fn truncate_point_bounding_box(bbox: PointBoundingBox, decimal_places: u8) -> PointBoundingBox {
    PointBoundingBox {
        north_west: truncate_coordinate(bbox.north_west.0, decimal_places).into(),
        north_east: truncate_coordinate(bbox.north_east.0, decimal_places).into(),
        south_west: truncate_coordinate(bbox.south_west.0, decimal_places).into(),
        south_east: truncate_coordinate(bbox.south_east.0, decimal_places).into(),
    }
}

impl TryFrom<Vec<f64>> for BoundingBox {
    type Error = BoundingBoxConversionError;

    fn try_from(bounding_box: Vec<f64>) -> Result<Self, Self::Error> {
        let south = bounding_box
            .get(0)
            .ok_or_else(|| BoundingBoxConversionError {
                _bounding_box: bounding_box.clone(),
            })?;

        let north = bounding_box
            .get(1)
            .ok_or_else(|| BoundingBoxConversionError {
                _bounding_box: bounding_box.clone(),
            })?;

        let west = bounding_box
            .get(2)
            .ok_or_else(|| BoundingBoxConversionError {
                _bounding_box: bounding_box.clone(),
            })?;

        let east = bounding_box
            .get(3)
            .ok_or_else(|| BoundingBoxConversionError {
                _bounding_box: bounding_box.clone(),
            })?;

        Ok(Self {
            south_west: Point::new(*west, *south).into(),
            south_east: Point::new(*east, *south).into(),
            north_west: Point::new(*west, *north).into(),
            north_east: Point::new(*east, *north).into(),
        })
    }
}

impl From<BoundingBox> for Vec<Coordinate<f64>> {
    fn from(bbox: BoundingBox) -> Self {
        vec![
            bbox.south_west,
            bbox.south_east,
            bbox.north_east,
            bbox.north_west,
            bbox.south_west,
        ]
    }
}
