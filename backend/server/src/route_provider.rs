use algorithm_api::api::{Area, Route, UserPreferences};
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct RouteProviderReq {
    pub(crate) start: String,
    pub(crate) end: String,
    /// Walking speed in kilometers per hour
    pub(crate) walking_speed_kmh: usize,
    pub(crate) area: Area,
    pub(crate) user_prefs: UserPreferences,
}

#[derive(Serialize)]
pub struct RouteProviderRes {
    pub(crate) route: Route,
}