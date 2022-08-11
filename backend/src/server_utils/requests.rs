use serde::{Deserialize, Serialize};
use trailscout_lib::algorithm::{Area, Route, UserPreferences};


///struct to contain parameters from route request
#[derive(Deserialize)]
pub struct RouteProviderReq {
    pub start: String,
    pub end: String,
    /// Walking speed in kilometers per hour
    pub walking_speed_kmh: usize,
    pub area: Area,
    pub user_prefs: UserPreferences,
}


///Response for Route request
#[derive(Serialize)]
pub struct RouteProviderRes<'a> {
    pub route: Route<'a>,
}

///struct to contain parameters from sights request
#[derive(Deserialize)]
pub struct SightsRequest {
   pub lat: f64,
   pub lon: f64,
   pub radius: f64
}
