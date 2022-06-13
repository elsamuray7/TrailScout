use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqCategory {
    pub (in crate::api) name: String,
    pub (in crate::api) pref: usize,
}

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqSight {
    pub (in crate::api) id: usize,
    pub (in crate::api) category: String,
    pub (in crate::api) pref: usize,
}

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqUserPrefs {
    pub (in crate::api) categories: Vec<RouteProviderReqCategory>,
    pub (in crate::api) sights: Vec<RouteProviderReqSight>,
}

#[derive(Deserialize)]
pub(in crate::api) struct RouteProviderReqRoot {
    pub (in crate::api) lat: f64,
    pub (in crate::api) lon: f64,
}

#[derive(Deserialize)]
pub struct RouteProviderReq {
    pub (in crate::api) start: String,
    pub (in crate::api) end: String,
    pub (in crate::api) root: RouteProviderReqRoot,
    pub (in crate::api) user_prefs: RouteProviderReqUserPrefs,
}

#[derive(Serialize)]
pub(in crate::api) struct RouteProviderResCoordinate {
    pub (in crate::api) lat: f64,
    pub (in crate::api) lon: f64,
}

#[derive(Serialize)]
pub struct RouteProviderRes {
    route: Vec<RouteProviderResCoordinate>,
}