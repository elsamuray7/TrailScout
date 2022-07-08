use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Formatter;
use std::fs::File;
use std::thread::{Thread, self, JoinHandle};
use std::{io, vec};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::num::{ParseFloatError, ParseIntError};
use std::ptr::null;
use actix_web::web::trace;
use futures::future::Then;
use itertools::Tuples;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::{Duration, Instant};
use log::{info,trace,error};
use osmpbf::{ElementReader, Element, Node, BlobReader, BlobType, Way};
use crate::data::graph::{calc_dist, Category, Edge, Node as GraphNode, Sight};

pub fn parse_osm_data (osmpbf_file_path: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> Result<(), io::Error> {
    let mut num_nodes: usize = 0;
    let mut num_edges: usize = 0;
    //let mut num_sights: usize = 0;

    let reader = BlobReader::from_path(osmpbf_file_path)?;
    let mut node_count = 0;
    let mut way_count = 0;
    let mut dense_count = 0;
    let mut relation_count = 0;

    let mut progress_counter = 0;

    let mut osm_id_to_node_id: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
    let mut thread_id_to_offset: BTreeMap<usize, usize> = BTreeMap::new();

    let mut temp_edges = Vec::<Vec<Edge>>::new();

    
    info!("Start reading the PBF file!");
    let time_start = Instant::now();
    //read the file into memory with multi threading
    let mut threads = Vec::new();
    let mut thread_id = 0;
    let entries = reader.for_each(|result|{
        let blob = result.unwrap();
        let blob_type = blob.get_type();
        if blob_type == BlobType::OsmHeader {
            info!("This is a Header");
            let header = blob.to_headerblock().unwrap();
            info!("required Features: {:?}", header.required_features());
            info!("optional Features: {:?}", header.optional_features());
        } else if blob_type == BlobType::OsmData {
            info!("This is a Block");
            let thread_result = thread::spawn(move || {
                let data = blob.to_primitiveblock().unwrap();
                let mut x = 0;
                let mut result = (Vec::<GraphNode>::new(), Vec::<Edge>::new(), BTreeMap::<usize,(usize,usize)>::new(), thread_id);
                //start iterating through the blob elements
                data.for_each_element(|element| {
                    match element {
                        Element::Node(n) => {
                            // TODO if no tags corrects tags for category + category enum
                            let node = GraphNode {
                                osm_id: n.id() as usize,
                                //TODO assign a proper id or remove the field if it turns out to be unnecessary
                                id: x, 
                                lat: n.lat(),
                                lon: n.lon(),
                            };
                            result.2.insert(node.osm_id, (x, thread_id));
                            result.0.push(node);
                        },
                        Element::DenseNode(n) => {
                            // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
                            let node = GraphNode {
                                osm_id: n.id() as usize,
                                //TODO assign a proper id or remove the field if it turns out to be unnecessary
                                id: x,
                                lat: n.lat(),
                                lon: n.lon(),
                            };
                            result.2.insert(node.osm_id, (x, thread_id));
                            result.0.push(node);
                        },
                        Element::Way(w) => {
                            // TODO way id; check way tags
                            let mut way_ref_iter = w.refs();
                            let mut osm_src = way_ref_iter.next().unwrap() as usize;
                            for node_id  in way_ref_iter {
                                let osm_tgt = node_id as usize;
                                let mut edge = Edge {
                                    osm_id: w.id() as usize,
                                    osm_src: osm_src,
                                    osm_tgt: osm_tgt,
                                    src: 0,
                                    tgt: 0,
                                    dist: 0
                                };
                                result.1.push(edge);
                                osm_src = osm_tgt;
                            }
                        },
                        Element::Relation(r) => trace!("Relation"),
                        _ => error!("Unrecognized Element")
                    }
                    x += 1;
                });
                info!("x is {}", x);
                result
            });
            threads.push(thread_result);
            thread_id += 1;
        }
    });
    //join all threads and accumulate results
    let mut offset_counter = 0;
    for t in threads  {
        let mut result = t.join().unwrap();
        nodes.append(&mut result.0);
        temp_edges.push(result.1);
        result.0.len();
        osm_id_to_node_id.extend(result.2);
        thread_id_to_offset.insert(result.3, offset_counter);
        offset_counter += result.0.len();
    }
    let time_duration = time_start.elapsed();
    info!("Finished reading PBF file in {} seconds!", time_duration.as_secs());

    for edge in edges {
        let srcTuple = osm_id_to_node_id.get(&edge.osm_src).unwrap();
        let src = srcTuple.0 + thread_id_to_offset.get(&srcTuple.1).unwrap();
        let src_node = nodes.get(src).unwrap();

        let tgtTuple = osm_id_to_node_id.get(&edge.osm_tgt).unwrap();
        let tgt = tgtTuple.0 + thread_id_to_offset.get(&tgtTuple.1).unwrap();
        let tgt_node = nodes.get(tgt).unwrap();
        let mut edge = edge;
        edge.src = src;
        edge.tgt = tgt;
        edge.dist = calc_dist(src_node.lat, src_node.lon, tgt_node.lat, tgt_node.lon);
    }
    
    let time_duration = time_start.elapsed();
    info!("Finished processing the PBF data in {} seconds!", time_duration.as_secs());


    /*info!("Start reading the PBF file!");
    let time_start = Instant::now();
    //this is single threaded
    let entries = reader.for_each(|result|{
        let blob = result.unwrap();
        let blob_type = blob.get_type();
        if blob_type == BlobType::OsmHeader {
            info!("This is a Header");
            let header = blob.to_headerblock().unwrap();
            info!("required Features: {:?}", header.required_features());
            info!("optional Features: {:?}", header.optional_features())
        } else if blob_type == BlobType::OsmData {
            println!("This is a Block");
            let data = blob.to_primitiveblock().unwrap();
            let mut x = 0;
            data.for_each_element(|element| {
                x += 1;
            });
            info!("x is {}", x);
        }
        println!("This is a blob");
    });
    let time_duration = time_start.elapsed();
    info!("Finished reading PBF file in {} seconds!", time_duration.as_secs());*/

    /*let entries:(Vec<GraphNode>, Vec<Edge>) = reader.par_map_reduce(
        |element| {

            let mut result = (Vec::<GraphNode>::new(), Vec::<Edge>::new());


            /*match element {
                Element::Way(_) => trace!("Way"),
                Element::DenseNode(_) => trace!("DenseNode"),
                Element::Node(_) => trace!("Node"),
                Element::Relation(_) => trace!("Relation"),
                _ => error!("Unrecognized Element")
            }*/
            if let Element::Node(n) = element {
                // TODO if no tags corrects tags for category + category enum
                let node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                };
                result.0.push(node);
            } else if let Element::DenseNode(n) = element {
                // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
                let node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                    //info: "".to_string()
                };
                result.0.push(node);
            } else if let Element::Way(w) = element {
                // TODO way id; check way tags
                /*let mut way_ref_iter = w.refs();
                let mut osm_src = way_ref_iter.next().unwrap() as usize;
                for node_id  in way_ref_iter {
                    let osm_tgt = node_id as usize;
                    let mut edge = Edge {
                        osm_id: w.id() as usize,
                        osm_src: osm_src,
                        osm_tgt: osm_tgt,
                        src: *osm_id_to_node_id.get(&osm_src).unwrap(),
                        tgt: *osm_id_to_node_id.get(&osm_tgt).unwrap(),
                        dist: 0
                    };
                    let src_node = &nodes[edge.src];
                    let tgt_node = &nodes[edge.tgt];
                    edge.dist = calc_dist(src_node.lat, src_node.lon, tgt_node.lat, tgt_node.lon);
                    result.1.push(edge);
                    osm_src = osm_tgt;
                }*/
            }
            return result
        },
        ||(Vec::<GraphNode>::new(), Vec::<Edge>::new()),      // Zero is the identity value for addition
        |mut m1, m2| {
            for k in m2.0 {
                m1.0.push(k);
            }
            for k in m2.1 {
                m1.1.push(k);
            }
            m1
        }
    )?;

    /*reader.for_each(|element| {
        if let Element::Node(n) = element {
            // TODO if no tags corrects tags for category + category enum
            /*
            let mut isSight = false;
            for (key, value) in n.tags() {
                match key {
                    "amenity" => {
                        isSight = true;
                        match value {
                            "restaurant" | "biergarten" | "cafe" | "fast_food" | "food_court" => {
                                let mut sight = Sight {
                                    node_id: n.id() as usize,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    category: Category::Restaurants,
                                };
                                sights.push(sight);
                                num_sights += 1;
                                node_count += 1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if(!isSight) {
                let mut node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                    //info: "".to_string()
                };
                osm_id_to_node_id.entry(node.osm_id)
                    .or_insert(num_nodes);
                nodes.push(node);
                num_nodes += 1;
                node_count += 1;
            }

             */

            let mut node = GraphNode {
                osm_id: n.id() as usize,
                id: num_nodes,
                lat: n.lat(),
                lon: n.lon(),
                //info: "".to_string()
            };
            osm_id_to_node_id.entry(node.osm_id)
                .or_insert(num_nodes);
            nodes.push(node);
            num_nodes += 1;
            node_count += 1;


            /*
            for (key, value) in n.tags() {
                node.info.push_str("key: (");
                node.info.push_str(key);
                node.info.push_str(") value: (");
                node.info.push_str(value);
                node.info.push_str(")\n");
            }

             */



        } else if let Element::DenseNode(n) = element {
            // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
            /*
            let mut isSight = false;
            for (key, value) in n.tags() {
                match key {
                    "amenity" => {
                        isSight = true;
                        match value {
                            "restaurant" | "biergarten" | "cafe" | "fast_food" | "food_court" => {
                                let mut sight = Sight {
                                    node_id: n.id() as usize,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    category: Category::Restaurants,
                                };
                                sights.push(sight);
                                num_sights += 1;
                                node_count += 1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            if(!isSight) {
                let mut node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                    //info: "".to_string()
                };
                osm_id_to_node_id.entry(node.osm_id)
                    .or_insert(num_nodes);
                nodes.push(node);
                num_nodes += 1;
                node_count += 1;
            }

             */

            let mut node = GraphNode {
                osm_id: n.id() as usize,
                id: num_nodes,
                lat: n.lat(),
                lon: n.lon(),
                //info: "".to_string()
            };
            osm_id_to_node_id.entry(node.osm_id)
                .or_insert(num_nodes);
            nodes.push(node);
            num_nodes += 1;
            dense_count += 1;
        } else if let Element::Way(w) = element {
            // TODO way id; check way tags
            let mut way_ref_iter = w.refs();
            let mut osm_src = way_ref_iter.next().unwrap() as usize;
            for node_id  in way_ref_iter {
                let osm_tgt = node_id as usize;
                let mut edge = Edge {
                    osm_id: w.id() as usize,
                    osm_src: osm_src,
                    osm_tgt: osm_tgt,
                    src: *osm_id_to_node_id.get(&osm_src).unwrap(),
                    tgt: *osm_id_to_node_id.get(&osm_tgt).unwrap(),
                    dist: 0
                };
                let src_node = &nodes[edge.src];
                let tgt_node = &nodes[edge.tgt];
                edge.dist = calc_dist(src_node.lat, src_node.lon, tgt_node.lat, tgt_node.lon);
                //let srcNode = &nodes[edge.src];
                //let tgtNode = &nodes[edge.tgt];
                //let dist = calc_dist(srcNode.lat, srcNode.lon), tgt.;

                //let src_node = &nodes[edge.src];
                //let tgt_node = &nodes[edge.tgt];
                //edge.dist = calc_dist(&src_node.lat, &src_node.lon, &tgt_node.lat, &tgt_node.lon);

                edges.push(edge);
                num_edges += 1;
                way_count += 1;

                osm_src = osm_tgt;
            }
            /*
            if(w.id() == 3999579) {
                println!("way id 3999579:");
                for val in w.refs() {
                    println!("{}", val);
                }
            }
            */
        } else if let Element::Relation(_) = element {
            relation_count += 1;
        }
        if progress_counter % 40000 == 0 {
            trace!("finished processing {} elements", progress_counter);
        }
        progress_counter += 1;
        //println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
    })?;*/*/

    /*edges.sort_unstable_by(|e1, e2| {
        let id1 = e1.src;
        let id2 = e2.src;
        id1.cmp(&id2).then_with(||{
            let id1 = e1.tgt;
            let id2 = e2.tgt;
            id1.cmp(&id2)
        })
    });*/
    Ok(())
}

pub fn write_graph_file(graph_file_path_out: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> std::io::Result<()> {
    let file = File::create(graph_file_path_out)?;
    let mut file = LineWriter::new(file);
    /*
    file.write((format!("Number of Nodes: {}\n", nodes.len())).as_bytes())?;
    file.write((format!("Number of Edges: {}\n", edges.len())).as_bytes())?;
    file.write((format!("osm_id node_id lat lon\n")).as_bytes())?;
    for node in &*nodes {
        file.write((format!("{} {} {} {}\n", node.osm_id, node.id, node.lat, node.lon).as_bytes()))?;
        file.write((format!("info\n{}\n", node.info)).as_bytes())?;
    }
    file.write((format!("osm_id osm_src osm_tgt src tgt dist\n")).as_bytes())?;
    for edge in &*edges {
        file.write((format!("{} {} {} {} {} {}\n", edge.osm_id, edge.osm_src, edge.osm_tgt, edge.src, edge.tgt, edge.dist)).as_bytes())?;
    }
     */
    file.write((format!("{}\n", nodes.len())).as_bytes())?;
    file.write((format!("{}\n", sights.len())).as_bytes())?;
    file.write((format!("{}\n", edges.len())).as_bytes())?;
    for node in &*nodes {
        file.write((format!("{} {} {}\n", node.id, node.lat, node.lon).as_bytes()))?;
    }
    /*
    for sight in &*sights {
        file.write((format!("{} {} {}\n", node.id, node.lat, node.lon).as_bytes()))?;
    }
     */
    for edge in &*edges {
        file.write((format!("{} {} {}\n", edge.src, edge.tgt, edge.dist)).as_bytes())?;
    }
    Ok(())
}