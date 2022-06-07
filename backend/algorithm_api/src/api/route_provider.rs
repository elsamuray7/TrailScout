use serde::{Serialize, Deserialize};
use crate::api::Coordinate;

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqCategory {
    name: String,
    pref: usize,
}

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqSight {
    id: usize,
    category: String,
    pref: usize,
}

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqUserPrefs {
    categories: Vec<RouteProviderReqCategory>,
    sights: Vec<RouteProviderReqSight>,
}

#[derive(Deserialize)]
pub struct RouteProviderReq {
    user_prefs: RouteProviderReqUserPrefs,
}

#[derive(Serialize)]
pub struct RouteProviderRes {
    route: Vec<Coordinate>,
}