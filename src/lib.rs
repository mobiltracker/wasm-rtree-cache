use std::sync::Mutex;

use once_cell::sync::OnceCell;
use rtree::{BoundingBox, CoordinateCache};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod rtree;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub static R_TREE: OnceCell<Mutex<CoordinateCache>> = OnceCell::new();

#[wasm_bindgen]
pub struct Bbox {
    pub south_west: Coordinate,
    pub south_east: Coordinate,
    pub north_west: Coordinate,
    pub north_east: Coordinate,
}

// Wasm iterop coordinate
#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

#[wasm_bindgen]
impl Coordinate {
    pub fn lat(&self) -> f32 {
        self.y
    }

    pub fn lon(&self) -> f32 {
        self.x
    }
}

impl From<Coordinate> for geo::Coordinate<f64> {
    fn from(c: Coordinate) -> Self {
        Self {
            x: c.x as f64,
            y: c.y as f64,
        }
    }
}

impl From<geo::Coordinate<f64>> for Coordinate {
    fn from(c: geo::Coordinate<f64>) -> Self {
        Self {
            x: c.x as f32,
            y: c.y as f32,
        }
    }
}

impl From<Bbox> for BoundingBox {
    fn from(bbox: Bbox) -> Self {
        Self {
            north_east: bbox.north_east.into(),
            north_west: bbox.north_west.into(),
            south_east: bbox.south_east.into(),
            south_west: bbox.south_west.into(),
        }
    }
}

impl From<BoundingBox> for Bbox {
    fn from(bbox: BoundingBox) -> Self {
        Self {
            north_east: bbox.north_east.into(),
            north_west: bbox.north_west.into(),
            south_east: bbox.south_east.into(),
            south_west: bbox.south_west.into(),
        }
    }
}

#[wasm_bindgen]
pub fn set_bbox(data: String, bbox: Bbox, reference_point: Option<Coordinate>) {
    let bbox = BoundingBox::from(bbox);
    let reference_point = reference_point.map(|c| geo::Coordinate::from(c));
    let r_tree = R_TREE.get_or_init(|| Mutex::new(CoordinateCache::new()));
    let mut r_tree = r_tree.lock().unwrap();

    r_tree.set(data, bbox, reference_point);
}

#[wasm_bindgen]
pub fn get(coordinate: Coordinate) -> Option<String> {
    let r_tree = R_TREE.get_or_init(|| Mutex::new(CoordinateCache::new()));
    let r_tree = r_tree.lock().unwrap();
    r_tree.get(coordinate.into())
}

#[wasm_bindgen]
pub fn clear() {
    let r_tree = R_TREE.get_or_init(|| Mutex::new(CoordinateCache::new()));
    r_tree.lock().unwrap().clear();
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}
