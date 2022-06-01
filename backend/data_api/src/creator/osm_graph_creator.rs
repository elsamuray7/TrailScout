/*

    fn parse_graph (&mut self, graph_file_path: &str) -> Result<(), io::Error> {
        let reader = ElementReader::from_path(graph_file_path)?;

        let mut node_count = 0;
        let mut way_count = 0;
        let mut dense_count = 0;
        let mut relation_count = 0;

        reader.for_each(|element| {
            if let Element::Node(_) = element {

                node_count += 1;
                /*
                let mut node = Node {
                    id: n.id() as usize,
                    tags: vec![],
                    lat: n.lat(),
                    lon: n.lon(),
                    info: "".to_string()
                };
                for (key, value) in n.tags() {
                    node.tags.push((key.parse().unwrap(), value.parse().unwrap()));
                }
                println!("Node with id {} lat {} lon {}",
                            n.id(), n.lat(), n.lon());
                self.nodes.push(node);
                self.num_nodes += 1;
                 */
            } else if let Element::Way(_) = element {
                way_count += 1;
            } else if let Element::DenseNode(n) = element {
                dense_count += 1;
                let mut node = Node {
                    id: n.id() as usize,
                    tags: vec![],
                    lat: n.lat(),
                    lon: n.lon(),
                    info: "".to_string()
                };
                for (key, value) in n.tags() {
                    node.tags.push((key.parse().unwrap(), value.parse().unwrap()));
                }
                self.nodes.push(node);
            } else if let Element::Relation(_) = element {
                relation_count += 1;
            }
        })?;

        println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
        Ok(())
    }

    fn write_graph(&mut self, graph_file_path_out: &str) -> std::io::Result<()> {
        let file = File::create(graph_file_path_out)?;
        let mut file = LineWriter::new(file);

        //file.write((format!("{}", self.meta)).as_bytes())?;
        //file.write((format!("{}\n", self.num_nodes)).as_bytes())?;
        //file.write((format!("{}\n", self.new_num_edges)).as_bytes())?;

        for node in &self.nodes {
            file.write((format!("node lat lon info \n{} {} {} {}\ntags\n", node.id, node.lat, node.lon, node.info)).as_bytes())?;
            for (key, value) in &node.tags {
                file.write((format!("key:{} value:{}\n\n", key, value)).as_bytes())?;
            }
        }
/*
        for (a, b, dist, edge_type, maxspeed) in &self.new_edges {
            file.write((format!("{} {} {} {} {}\n", a, b, dist, edge_type, maxspeed)).as_bytes())?;
        }
*/
        Ok(())
    }
}

fn main() -> Result<(), io::Error> {
    let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "./osm_graphs/bremen-latest.fmi";

    let mut graph = Graph::new();
    Graph::parse_graph(&mut graph, &in_graph);
    Graph::write_graph(&mut graph, &out_graph);

    Ok(())
}


 */