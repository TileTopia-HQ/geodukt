//! geodukt CLI — declarative geospatial ETL pipeline tool.

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use geodukt_core::manifest::Manifest;
use geodukt_core::pipeline::Pipeline;
use geodukt_io::geojson_io::{MultiFormatReader, MultiFormatWriter};
use geodukt_transforms::registry::default_registry;

#[derive(Parser)]
#[command(name = "geodukt", about = "Declarative geospatial ETL pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the pipeline defined in geodukt.toml
    Run {
        /// Path to manifest file
        #[arg(short, long, default_value = "geodukt.toml")]
        manifest: PathBuf,
    },
    /// Validate the pipeline DAG without executing
    Validate {
        /// Path to manifest file
        #[arg(short, long, default_value = "geodukt.toml")]
        manifest: PathBuf,
    },
    /// Show the execution order (DAG topological sort)
    Graph {
        /// Path to manifest file
        #[arg(short, long, default_value = "geodukt.toml")]
        manifest: PathBuf,
    },
    /// Initialize a new pipeline project
    Init {
        /// Project directory name
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { manifest } => cmd_run(&manifest),
        Command::Validate { manifest } => cmd_validate(&manifest),
        Command::Graph { manifest } => cmd_graph(&manifest),
        Command::Init { name } => cmd_init(&name),
    }
}

fn load_manifest(path: &PathBuf) -> Manifest {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {e}", path.display());
        std::process::exit(1);
    });
    Manifest::from_toml(&content).unwrap_or_else(|e| {
        eprintln!("Error parsing manifest: {e}");
        std::process::exit(1);
    })
}

fn cmd_run(path: &PathBuf) {
    let manifest = load_manifest(path);
    let pipeline = Pipeline::new(manifest).unwrap_or_else(|e| {
        eprintln!("Pipeline error: {e}");
        std::process::exit(1);
    });

    let transforms = default_registry();
    let reader = MultiFormatReader;
    let writer = MultiFormatWriter;

    let report = pipeline
        .execute(&reader, &transforms, &writer)
        .unwrap_or_else(|e| {
            eprintln!("Execution error: {e}");
            std::process::exit(1);
        });

    println!("Pipeline completed successfully:");
    for step in &report.steps {
        println!("  {} — {} features", step.name, step.feature_count);
    }
}

fn cmd_validate(path: &PathBuf) {
    let manifest = load_manifest(path);
    let pipeline = Pipeline::new(manifest).unwrap_or_else(|e| {
        eprintln!("Validation failed: {e}");
        std::process::exit(1);
    });

    let order = pipeline.validate().unwrap_or_else(|e| {
        eprintln!("Validation failed: {e}");
        std::process::exit(1);
    });

    println!("Pipeline is valid. Execution order:");
    for (i, name) in order.iter().enumerate() {
        println!("  {}. {name}", i + 1);
    }
}

fn cmd_graph(path: &PathBuf) {
    let manifest = load_manifest(path);
    let pipeline = Pipeline::new(manifest).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });

    let order = pipeline.validate().unwrap();
    println!("DAG execution order:");
    for (i, name) in order.iter().enumerate() {
        if i > 0 {
            println!("    ↓");
        }
        println!("  [{name}]");
    }
}

fn cmd_init(name: &str) {
    let dir = PathBuf::from(name);
    fs::create_dir_all(&dir).unwrap_or_else(|e| {
        eprintln!("Error creating directory: {e}");
        std::process::exit(1);
    });
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::create_dir_all(dir.join("output")).unwrap();

    let manifest = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"

[[source]]
name = "input"
format = "geojson"
path = "data/input.geojson"

[[transform]]
name = "processed"
input = "input"
operation = "centroid"

[[sink]]
name = "output"
input = "processed"
format = "geojson"
path = "output/result.geojson"
"#
    );

    fs::write(dir.join("geodukt.toml"), manifest).unwrap();
    println!("Initialized new geodukt project in {name}/");
}
