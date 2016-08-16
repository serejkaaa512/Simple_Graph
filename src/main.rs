#[macro_use]
extern crate simple_graph;
use simple_graph::graph;
use std::io::prelude::*;
use std::fs::File;


fn main() {
    // let points: Vec<_> = formula!(y(x): f64 = {x*x}, x = [-150f64, 150f64; 1f64]).collect();

    let v = vec![(1f64, -74.756),
                 (2f64, -75.271),
                 (3f64, -75.674),
                 (4f64, -74.405),
                 (5f64, -74.809),
                 (6f64, -75.324),
                 (7f64, -75.727),
                 (8f64, -74.458),
                 (9f64, -74.862),
                 (10f64, -75.592)];



    let bmp = graph::create(v.into_iter(), 740, 480).unwrap();

    let mut file = File::create("graph.bmp").unwrap();
    file.write_all(&bmp).unwrap();



}
