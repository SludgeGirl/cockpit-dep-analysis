use std::{collections::HashMap, process::Command};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NpmDependency {
    version: String,
    resolved: Option<String>,
    overridden: Option<bool>,
    dependencies: Option<HashMap<String, NpmDependency>>,
}

/// Output of "npm ls --package-lock-only --json <package>"
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NpmLs {
    name: String,
    dependencies: HashMap<String, NpmDependency>,
}

impl NpmLs {
    pub fn new(name: &str) -> Self {
        let ls_output = Command::new("npm")
            .arg("ls")
            .arg("--package-lock-only")
            .arg("--json")
            .arg(name)
            .output()
            .expect("failed to run npm ls")
            .stdout;

        let mut ls_json: NpmLs = serde_json::from_slice(&ls_output).unwrap();
        ls_json.name = name.to_string();
        ls_json
    }

    pub fn top_level_deps(&self) -> Vec<&str> {
        self.dependencies.keys().map(|key| key.as_str()).collect()
    }

    pub fn is_top_level_dep(&self) -> bool {
        self.dependencies.keys().any(|key| key == &self.name)
    }
}
