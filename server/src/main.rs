use std::fs;
use std::path::Path;

use clap::Parser as ClapParser;
use jagua_rs::io::json_instance::JsonInstance;
use mimalloc::MiMalloc;
use rand::prelude::SmallRng;
use rand::SeedableRng;

use jagua_rs::io::parser;
use jagua_rs::io::parser::Parser;
use jagua_rs::util::polygon_simplification::PolySimplConfig;
use serde::{Deserialize, Serialize};
use lbf::io::json_output::JsonOutput;
use lbf::io::layout_to_svg::s_layout_to_svg;
use lbf::lbf_config::LBFConfig;
use lbf::lbf_optimizer::LBFOptimizer;
use lbf::{io, EPOCH};
use warp::Filter;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Serialize, Deserialize, Clone)]
struct RequestBody {
    uuid: String,
    input: JsonInstance,
    config: LBFConfig,
}

#[tokio::main]
async fn main() {
    print!("Starting server");
    let run = warp::post()
        .and(warp::path("nest"))
        .and(warp::body::json())
        .map(|body: RequestBody| {
            print!("Running");
            // Here, you would call your existing CLI function with the provided arguments
            // For the sake of this example, let's just return a dummy response

            let uuid = body.uuid;
            let json_instance = body.input;
            let config = body.config;

            let poly_simpl_config = match config.poly_simpl_tolerance {
                Some(tolerance) => PolySimplConfig::Enabled { tolerance },
                None => PolySimplConfig::Disabled,
            };

            let parser = Parser::new(poly_simpl_config, config.cde_config, true);
            let instance = parser.parse(&json_instance);

            let rng = match config.prng_seed {
                Some(seed) => SmallRng::seed_from_u64(seed),
                None => SmallRng::from_entropy(),
            };

            let mut optimizer = LBFOptimizer::new(instance.clone(), config, rng);
            let solution = optimizer.solve();

            let json_output = JsonOutput {
                instance: json_instance.clone(),
                solution: parser::compose_json_solution(&solution, &instance, EPOCH.clone()),
                config: config.clone(),
            };

            let solution_folder = Path::new("../solutions");

            fs::create_dir_all(&solution_folder).unwrap_or_else(|_| {
                panic!(
                    "could not create solution folder: {:?}",
                    solution_folder
                )
            });

            for (i, s_layout) in solution.layout_snapshots.iter().enumerate() {
                let svg_path = solution_folder
                    .join(format!("solution_{}_{}.svg", uuid, i));

                io::write_svg(
                    &s_layout_to_svg(s_layout, &instance, config.svg_draw_options),
                    Path::new(&svg_path),
                );
            }

            warp::reply::json(&json_output)
        });

    warp::serve(run)
        .run(([0, 0, 0, 0], 3030))
        .await;
}


