use std::io::{self, Write, Read};
use anyhow::{Context, Result};
use jagua_rs::io::import::Importer;
use jagua_rs::probs::bpp::io::ext_repr::ExtBPInstance;
use jagua_rs::probs::spp::io::ext_repr::ExtSPInstance;
use jagua_rs::probs::{bpp, spp};
use lbf::config::LBFConfig;
use lbf::io::output::{BPOutput, SPOutput};
use lbf::opt::lbf_bpp::LBFOptimizerBP;
use lbf::opt::lbf_spp::LBFOptimizerSP;
use lbf::EPOCH;
use log::info;
use rand::SeedableRng;
use rand::prelude::SmallRng;
use serde_json::{self, Value};

#[derive(serde::Deserialize)]
struct InputData {
    config: LBFConfig,
    problem_type: String,
    instance: Value,
}

#[derive(serde::Serialize)]
struct OutputData {
    success: bool,
    error: Option<String>,
    solution: Option<Value>,
    config: Option<LBFConfig>,
}

fn main() -> Result<()> {
    let mut input = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut input)?;
    
    let input_data: InputData = serde_json::from_str(&input)
        .context("Failed to parse input JSON")?;
    
    let config = input_data.config;
    let problem_type = input_data.problem_type.to_lowercase();
    
    info!("Successfully parsed LBFConfig: {config:?}");
    info!("Problem type: {problem_type}");
    
    let result = match problem_type.as_str() {
        "bpp" | "bin_packing" | "binpacking" => {
            let ext_bp_instance: ExtBPInstance = serde_json::from_value(input_data.instance)
                .context("Failed to parse BPP instance")?;
            solve_bpp(ext_bp_instance, config)
        }
        _ => {
            let output = OutputData {
                success: false,
                error: Some(format!("Unknown problem type: {}", problem_type)),
                solution: None,
                config: None,
            };
            serde_json::to_writer_pretty(io::stdout(), &output)?;
            return Ok(());
        }
    };
    
    match result {
        Ok(output) => {
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Err(e) => {
            let error_output = OutputData {
                success: false,
                error: Some(e.to_string()),
                solution: None,
                config: None,
            };
            serde_json::to_writer_pretty(io::stdout(), &error_output)?;
        }
    }
    
    Ok(())
}

fn solve_bpp(
    ext_instance: ExtBPInstance,
    config: LBFConfig,
) -> Result<OutputData> {
    let importer = Importer::new(
        config.cde_config,
        config.poly_simpl_tolerance,
        config.min_item_separation,
        config.narrow_concavity_cutoff_ratio,
    );
    
    let rng = match config.prng_seed {
        Some(seed) => SmallRng::seed_from_u64(seed),
        None => SmallRng::from_os_rng(),
    };
    
    let instance = bpp::io::import(&importer, &ext_instance)?;
    let sol = LBFOptimizerBP::new(instance.clone(), config.clone(), rng).solve();
    
    let solution = bpp::io::export(&instance, &sol, *EPOCH);
    let solution_value = serde_json::to_value(solution)?;
    
    let output = OutputData {
        success: true,
        error: None,
        solution: Some(solution_value),
        config: Some(config),
    };
    
    Ok(output)
}
