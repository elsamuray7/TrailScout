use actix_web::{http::{header::ContentType, StatusCode}, HttpResponse, ResponseError};
use derive_more::{Display, Error};
use trailscout_lib::algorithm::AlgorithmError;

///Custom Error for TrailScout
#[derive(Debug, Display, Error)]
pub enum TrailScoutError {
    #[display(fmt = "Unbekannter Algorithmus Fehler")]
    UnknownErrorServer,

    #[display(fmt = "Unbekannter Routing Algorithmus")]
    BadAlgoServer,

    #[display(fmt = "Keine erreichbaren Sehenswürdigkeiten im Radius gefunden")]
    NoSightsFoundServer,

    #[display(fmt = "Unbekannte Kategorie von Sehenswürdigkeiten gefunden")]
    UnknownCategoryServer,

    #[display(fmt = "Keine Route konnte gefunden werden")]
    NoRouteFoundServer,

    #[display(fmt = "Keine Präferenzen gefunden")]
    NoPreferencesProvidedServer,

    #[display(fmt = "Nächster Straßenknoten konnte nicht gefungen werden")]
    NoNearestNodeFoundServer,

    #[display(fmt = "Zeitfenster ist negativ")]
    NegativeTimeIntervalServer
}


impl ResponseError for TrailScoutError {
    fn status_code(&self) -> StatusCode {
        match *self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(self.to_string())
    }
}

///Match AlgorithmError (intern) to appropriate TrailScoutError (intended to show german user)
pub fn match_error(x: AlgorithmError) -> TrailScoutError {
    return match x {
        AlgorithmError::NegativeTimeInterval {..} => { TrailScoutError::NegativeTimeIntervalServer },
        AlgorithmError::NoSightsFound {..} => { TrailScoutError::NoSightsFoundServer },
        AlgorithmError::UnknownCategory {..} => { TrailScoutError::UnknownCategoryServer },
        AlgorithmError::NoRouteFound {..} => { TrailScoutError::NoRouteFoundServer },
        AlgorithmError::NoPreferencesProvided {..} => { TrailScoutError::NoPreferencesProvidedServer },
        AlgorithmError::NoNearestNodeFound {..} => { TrailScoutError::NoNearestNodeFoundServer },
        AlgorithmError::UnknownAlgorithm {..} => {TrailScoutError::BadAlgoServer},
        _ => {TrailScoutError::UnknownErrorServer }
    }
}