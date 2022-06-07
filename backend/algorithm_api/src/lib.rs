pub mod api;

#[cfg(test)]
mod tests {
    use crate::api;
    #[test]
    fn it_works() {
        let _ = api::default();
    }
}
