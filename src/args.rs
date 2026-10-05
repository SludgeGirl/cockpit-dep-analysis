use clap::Parser;

/// Tool for analyzing npm dependency vulnerabilities
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to project root. Defaults to current directory
    #[arg(short, long)]
    path: Option<String>,

    /// Do not run import analysis
    #[arg(long)]
    no_imports: bool,

    /// Name of the dependency you want to analyze
    dependency: String,

    /// Print analysis information in json format
    #[arg(short, long)]
    json: bool,

    /// Print verbose output to stderr
    #[arg(short, long)]
    verbose: bool,
}

impl Args {
    pub fn path(&self) -> &str {
        self.path.as_deref().unwrap_or(".")
    }

    pub fn dependency(&self) -> &str {
        &self.dependency
    }

    pub fn no_imports(&self) -> bool {
        self.no_imports
    }

    pub fn json(&self) -> bool {
        self.json
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
