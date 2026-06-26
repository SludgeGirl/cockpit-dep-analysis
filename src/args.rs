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
}
