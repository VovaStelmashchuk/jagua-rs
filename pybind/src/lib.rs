use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

use jagua_rs::io::json_instance::JsonInstance;
use mimalloc::MiMalloc;
use rand::prelude::SmallRng;
use rand::SeedableRng;

use jagua_rs::io::parser;
use jagua_rs::io::parser::Parser;
use jagua_rs::util::polygon_simplification::PolySimplConfig;
use lbf::io::json_output::JsonOutput;
use lbf::lbf_config::LBFConfig;
use lbf::lbf_optimizer::LBFOptimizer;
use lbf::EPOCH;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Serialize, Deserialize, Clone)]
struct RequestBody {
    uuid: String,
    input: JsonInstance,
    config: LBFConfig,
}

/// This function replicates your REST API logic (without SVG creation)
/// and exposes it as a Python-callable function.
#[pyfunction]
fn run_nest(json_request: &str) -> PyResult<String> {
    // Deserialize the JSON input.
    let request: RequestBody = serde_json::from_str(json_request)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let uuid = request.uuid;
    let json_instance = request.input;
    let config = request.config;

    // Set up polygon simplification configuration.
    let poly_simpl_config = match config.poly_simpl_tolerance {
        Some(tolerance) => PolySimplConfig::Enabled { tolerance },
        None => PolySimplConfig::Disabled,
    };

    // Create a parser instance and parse the input.
    let parser = Parser::new(poly_simpl_config, config.cde_config, true);
    let instance = parser.parse(&json_instance);

    // Set up the random number generator.
    let rng = match config.prng_seed {
        Some(seed) => SmallRng::seed_from_u64(seed),
        None => SmallRng::from_entropy(),
    };

    // Run the optimizer.
    let mut optimizer = LBFOptimizer::new(instance.clone(), config.clone(), rng);
    let solution = optimizer.solve();

    // Compose the JSON output.
    let json_output = JsonOutput {
        instance: json_instance,
        solution: parser::compose_json_solution(&solution, &instance, EPOCH.clone()),
        config: config,
    };

    // Serialize the output back to a JSON string.
    serde_json::to_string(&json_output)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}

/// Define the Python module.
#[pymodule]
fn nest_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_nest, m)?)?;
    Ok(())
}
