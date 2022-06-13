pub mod api;

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use data_api::api::graph::Graph;
    #[test]
    fn it_works() {
        let _ = Arc::new(RwLock::new(Graph::new()));
    }
}
