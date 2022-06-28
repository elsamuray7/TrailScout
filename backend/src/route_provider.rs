use trailscout_lib::algorithm::{Area, Route, UserPreferences};
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
pub struct RouteProviderRes<'a> {
    pub(crate) route: Route<'a>,
}