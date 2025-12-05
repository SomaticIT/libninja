use crate::extractor::extract_spec;
use anyhow::{anyhow, Result};
use clap::{Args, ValueEnum};
use convert_case::{Case, Casing};
use hir::{Config, Language};
use openapiv3::{OpenAPI, VersionedOpenAPI};
use std::fs::File;
use std::path::{Path, PathBuf};

/// CLI flags
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Flag {
    /// Only used by Rust. Adds ormlite::TableMeta flags to the code.
    Ormlite,
    /// Only used by Rust (for now). Adds fake::Dummy flags to the code.
    Fake,
}

#[derive(Args, Debug)]
pub struct Generate {
    /// Service name.
    #[clap(short, long = "lang", default_value = "rust")]
    pub language: Language,

    /// Toggle whether to generate examples.
    /// Defaults to true
    #[clap(long, default_value = "true")]
    examples: bool,

    #[clap(short, long)]
    output_dir: Option<String>,

    #[clap(short, long)]
    config: Vec<Flag>,

    /// List of additional namespaced traits to derive on generated structs.
    #[clap(long)]
    derive: Vec<String>,

    /// The "service" name. E.g. if we want to generate a library for the Stripe API, this would be "Stripe".
    name: String,

    /// Path to the OpenAPI spec file.
    spec_filepath: String,
}

impl Generate {
    pub fn new() -> Self {
        Generate {
            language: Language::Rust,
            examples: true,
            output_dir: None,
            config: Vec::new(),
            derive: Vec::new(),
            name: String::new(),
            spec_filepath: String::new(),
        }
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }
    pub fn with_examples(mut self, examples: bool) -> Self {
        self.examples = examples;
        self
    }
    pub fn with_output_dir(mut self, output_dir: String) -> Self {
        self.output_dir = Some(output_dir);
        self
    }
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    pub fn with_spec_filepath(mut self, spec_filepath: String) -> Self {
        self.spec_filepath = spec_filepath;
        self
    }
    pub fn add_derive(mut self, derive: String) -> Self {
        self.derive.push(derive);
        self
    }
    pub fn with_derive(mut self, derive: Vec<String>) -> Self {
        self.derive = derive;
        self
    }
    pub fn add_config(mut self, config: Flag) -> Self {
        self.config.push(config);
        self
    }
    pub fn with_config(mut self, config: Vec<Flag>) -> Self {
        self.config = config;
        self
    }

    pub fn run(self) -> Result<()> {
        let spec = PathBuf::from(self.spec_filepath);
        let spec = read_spec(&spec)?;
        let output_dir = PathBuf::from(self.output_dir.unwrap_or_else(|| ".".to_string()));
        let spec = extract_spec(&spec)?;
        let config = Config {
            name: self.name.to_case(Case::Pascal),
            dest: output_dir,
            derives: self.derive,
            build_examples: self.examples,
            ormlite: false,
        };
        match self.language {
            Language::Rust => codegen_rust::generate_rust_library(spec, config),
        }
    }
}

pub fn read_spec(path: &Path) -> Result<OpenAPI> {
    let file = File::open(path).map_err(|_| anyhow!("{:?}: OpenAPI file not found.", path))?;
    let ext = path
        .extension()
        .map(|s| s.to_str().expect("Extension isn't utf8"))
        .unwrap_or_else(|| "yaml");
    let openapi: VersionedOpenAPI = match ext {
        "yaml" | "yml" => serde_yaml::from_reader(file)?,
        "json" => serde_json::from_reader(file)?,
        _ => panic!("Unknown file extension"),
    };
    let openapi = openapi.upgrade();
    Ok(openapi)
}
