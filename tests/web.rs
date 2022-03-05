//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::convert::TryInto;
use wasm_bindgen_test::*;
use wasm_rtree_cache::rtree::BoundingBox;
use wasm_rtree_cache::{Bbox, Coordinate};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn contains_point() {
    let bbox: BoundingBox = vec![-30.0146987, -30.0115462, -51.1833537, -51.1832816]
        .try_into()
        .unwrap();

    let bbox: Bbox = bbox.into();
    let data = "Porto Alegre".to_string();

    let reference_point = Coordinate {
        x: -51.1833500,
        y: -30.0126987,
    };

    wasm_rtree_cache::clear();
    wasm_rtree_cache::set_bbox(data.clone(), bbox, Some(reference_point));
    let address = wasm_rtree_cache::get(reference_point).unwrap();

    assert_eq!(address, data);
}

#[wasm_bindgen_test]
pub fn do_not_contains_point() {
    let bbox: BoundingBox = vec![-30.0146987, -30.0115462, -51.1833537, -51.1832816]
        .try_into()
        .unwrap();

    let bbox: Bbox = bbox.into();
    let data = "Porto Alegre".to_string();

    let reference_point = Coordinate {
        x: -51.1833500,
        y: -29.0126987,
    };

    wasm_rtree_cache::clear();
    wasm_rtree_cache::set_bbox(data.clone(), bbox, Some(reference_point));
    let address = wasm_rtree_cache::get(reference_point);

    assert!(address.is_none());
}

#[wasm_bindgen_test]
pub fn overlapping_bbox_huge() {
    let small_bbox: BoundingBox = vec![-30.0146987, -30.0115462, -51.1833537, -51.1832816]
        .try_into()
        .unwrap();

    let huge_bbox: BoundingBox = vec![-40.0, -20.0, -60.0, -40.0].try_into().unwrap();

    let data = "Small".to_string();

    let reference_point = Coordinate {
        x: -51.1833500,
        y: -30.0126987,
    };

    wasm_rtree_cache::clear();
    wasm_rtree_cache::set_bbox(data.clone(), small_bbox.into(), Some(reference_point));
    wasm_rtree_cache::set_bbox("HUGE".to_string(), huge_bbox.into(), Some(reference_point));
    let address = wasm_rtree_cache::get(reference_point).unwrap();

    assert_eq!(address, data);
}

#[wasm_bindgen_test]
pub fn set_bbox_huge() {
    let bbox: BoundingBox = vec![-31.0, -20.0, -50.0, -40.0].try_into().unwrap();

    let data = "Porto Alegre".to_string();

    let reference_point = Coordinate {
        x: -45.222222,
        y: -25.111111,
    };

    let max_len = 10000.0;

    wasm_rtree_cache::clear();
    wasm_rtree_cache::set_bbox(data.clone(), bbox.into(), Some(reference_point));
    let result = wasm_rtree_cache::get(reference_point).unwrap();

    assert_eq!(result, data);
}
