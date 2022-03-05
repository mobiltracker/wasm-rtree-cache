import {
  get as addressCacheGet,
  set_bbox as addressCacheSet,
  clear as addressCacheClear,
  Coordinate,
  Bbox,
} from "wasm-rtree-cache";
import { formatOsmAddress, OsmAddress, OsmSearchResult } from "./nominatim";

export async function nominatimCachedReverseGeocode(lat: number, lon: number) {
  const coordinate: Coordinate = Coordinate.new(lat, lon);
  let address = addressCacheGet(coordinate);

  if (!address) {
    console.log("cache miss, fetching..");

    const data = await fetch(
      `https://nominatim.openstreetmap.org/reverse?lat=${lat}&lon=${lon}&format=jsonv2&addressdetails=1>`
    );

    const json = await data.json();

    const parsed = parse(json);
    address = formatOsmAddress(parsed.address);

    const bboxVec = parsed.boundingbox;
    if (bboxVec) {
      const bbox = Bbox.from_osm_bbox(Float64Array.from(bboxVec));
      addressCacheSet(address, bbox, coordinate);
    }
  } else {
    console.log("cache hit");
  }

  console.log(address);
  return address;
}

function parse(d: any): OsmSearchResult {
  return {
    address: {
      attraction: d.address.attraction,
      bakery: d.address.bakery,
      borough: d.address.borough,
      city_block: d.address.city_block,
      city_district: d.address.city_district,
      city: d.address.city,
      commercial: d.address.commercial,
      construction: d.address.construction,
      continent: d.address.continent,
      country_code: d.address.country_code,
      country: d.address.country,
      county: d.address.county,
      district: d.address.district,
      electronics: d.address.electronics,
      farm: d.address.farm,
      farmyard: d.address.farmyard,
      hamlet: d.address.hamlet,
      house_name: d.address.house_name,
      house_number: d.address.house_number,
      industrial: d.address.industrial,
      isolated_dwelling: d.address.isolated_dwelling,
      municipality: d.address.municipality,
      neighbourhood: d.address.neighbourhood,
      peak: d.address.peak,
      pedestrian: d.address.pedestrian,
      postcode: d.address.postcode,
      public_building: d.address.public_building,
      region: d.address.region,
      residental: d.address.residental,
      road: d.address.road,
      state: d.address.state,
      state_district: d.address.state_district,
      subdivision: d.address.subdivision,
      suburb: d.address.suburb,
      town: d.address.town,
      village: d.address.village,
    },
    boundingbox: (d.boundingbox as string[]).map((b) => +b),
    class: d.class,
    display_name: d.display_name,
    importance: d.importance,
    lat: +d.lat,
    licence: d.licence,
    lon: +d.lon,
    osm_type: d.osm_type,
    place_id: +d.place_id,
    svg: d.svg,
    type: d.type,
  };
}
