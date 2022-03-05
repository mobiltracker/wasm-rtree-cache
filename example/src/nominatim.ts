export type OsmAddress = {
  attraction?: string;
  bakery?: string;
  borough?: string;
  city_block?: string;
  city_district?: string;
  city: string;
  commercial?: string;
  construction?: string;
  continent?: string;
  country_code: string;
  country: string;
  county?: string;
  district?: string;
  electronics?: string;
  farm?: string;
  farmyard?: string;
  hamlet?: string;
  house_number?: string;
  house_name?: string;
  industrial?: string;
  isolated_dwelling?: string;
  municipality?: string;
  neighbourhood?: string;
  peak?: string;
  pedestrian?: string;
  postcode?: string;
  public_building?: string;
  region?: string;
  residental?: string;
  road?: string;
  state: string;
  state_district: string;
  subdivision?: string;
  suburb?: string;
  town?: string;
  village?: string;
};

export type OsmSearchResult = {
  address: OsmAddress;
  boundingbox?: number[];
  class: string;
  display_name: string;
  importance: number;
  lat: number;
  licence: string;
  lon: number;
  osm_type: PlaceTypeLabel;
  place_id: number;
  svg?: string;
  type: string;
};

const PLACES_TYPES = {
  node: "N" as "N",
  way: "W" as "W",
  relation: "R" as "R",
};

type Places = typeof PLACES_TYPES;
type PlaceTypeLabel = keyof Places;

export function formatOsmAddress(address: OsmAddress): string {
  // rua numero bairro cidade estado

  const street =
    address.road ??
    address.residental ??
    address.city_block ??
    address.commercial ??
    address.farmyard ??
    address.farm ??
    address.electronics ??
    address.public_building ??
    address.bakery ??
    address.attraction;

  const number = address.house_number ?? address.house_name;
  const neighbourhood =
    address.suburb ??
    address.neighbourhood ??
    address.borough ??
    address.district ??
    address.subdivision ??
    address.city_district;

  const city =
    address.city ??
    address.town ??
    address.village ??
    address.municipality ??
    address.isolated_dwelling ??
    address.hamlet;

  const state =
    address.state ??
    address.state_district ??
    address.country ??
    address.municipality ??
    address.region;

  const postcode = address.postcode;

  const formatted = [street, number, neighbourhood, city, state, postcode]
    .filter((s) => s !== undefined)
    .join(", ");

  return formatted;
}
