use std::any::Any;
use osmpbf::{ElementReader, Element};

fn main() {
    graph_parsing();
}

fn graph_parsing() -> std::io::Result<()> {
    let reader = ElementReader::from_path("C:/Users/Acer/Documents/EnProFMI2022/backend/graph_creator_osm/osm_graphs/bremen-latest.osm.pbf")?;
    let mut ways = 0_u64;
    let lat = 53.5449736 as f64;
    let lon = 8.5683268 as f64;

    // Increment the counter by one for each way.
    reader.for_each(|element| {
        if let Element::Way(w) = element {
            if (w.id() == 171748328) {
                println!("way found");
            }
            ways += 1;
        }
    })?;

    println!("Number of ways: {}", ways);
    Ok(())
}