use serde::{Serialize, Deserialize};
use crate::api::{Area, Route, UserPreferences};

#[derive(Deserialize)]
pub struct RouteProviderReq {
    pub(crate) start: String,
    pub(crate) end: String,
    pub(crate) area: Area,
    pub(crate) user_prefs: UserPreferences,
}

#[derive(Serialize)]
pub struct RouteProviderRes {
    route: Route,
}