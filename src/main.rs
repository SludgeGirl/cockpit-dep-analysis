use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{imports::ImportAnalyzer, npm::NpmLs, obs::Obs};

mod args;
mod imports;
mod npm;
mod obs;
mod util;

/// Known dependencies that are only used for building projects
const DEP_BUILD_ALLOW_LIST: &[&str] = &[
    // TODO: support wildcards
    "@types/react",
    "@types/react-dom",
    "@typescript-eslint/eslint-plugin",
    "esbuild-wasm",
    "eslint",
    "eslint-config-standard",
    "eslint-config-standard-jsx",
    "eslint-config-standard-react",
    "eslint-plugin-import",
    "eslint-plugin-node",
    "eslint-plugin-promise",
    "eslint-plugin-react",
    "eslint-plugin-react-hooks",
    "qunit",
    "stylelint",
    "stylelint-config-recommended-scss",
    "stylelint-config-standard",
    "stylelint-config-standard-scss",
    "stylelint-formatter-pretty",
    "typescript",
];

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Analysis {
    update_recommended: bool,
    /// Is dependency used by the project at all
    /// If this is false, all other values are meaningless
    is_used: bool,
    /// Is dependency directly used (false means that it's a sub dependency)
    is_direct_dep: bool,
    direct_deps: Vec<String>,
    /// Is dependency only used by known build tools
    is_build_only: bool,

    /// Is dependency used by the application
    /// or any of the runtime sub dependencies
    ///
    /// This is only defined if import analysis is enabled
    is_app_dep: Option<bool>,
    app_imports: Option<Vec<String>>,

    /// Is used by build.js or it's dependencies
    ///
    /// This is only defined if import analysis is enabled
    is_build_js_dep: Option<bool>,
    build_js_imports: Option<Vec<String>>,
}

impl Analysis {
    fn print(&self, args: &args::Args) {
        println!("Dependency info for '{}':\n", args.dependency());

        if !self.is_used {
            println!("Dependency is not used by the project");
            return;
        }

        println!("Used by following project dependencies:");
        for dep in &self.direct_deps {
            print!("\t{dep}");
            if DEP_BUILD_ALLOW_LIST.contains(&dep.as_str()) {
                print!(" (known build dependency)")
            }
            println!();
        }

        println!();
        if self.is_direct_dep {
            println!("Dependency is a direct dependency");
        } else {
            println!("Dependency is NOT a direct dependency");
        }

        println!();
        if self.is_build_only {
            println!("All dependencies are known build dependencies");
        } else {
            println!("All dependencies are NOT known build dependencies");
        }

        if self.is_build_only {
            println!("\n\n");
            println!("Dependency is only used by known build dependencies. No need to update");
            return;
        }

        if args.no_imports() {
            return;
        }

        println!();
        if !self.is_build_js_dep.unwrap() {
            println!("Dependency is not used by build.js");
        } else {
            println!("Dependency is used by build.js in following paths");
            for found in self.build_js_imports.as_ref().unwrap() {
                println!("\t{found}");
            }
        };

        println!();
        if !self.is_app_dep.unwrap() {
            println!("Dependency is not used by the application");
        } else {
            println!("Dependency is used by application in following paths");
            for found in self.app_imports.as_ref().unwrap() {
                println!("\t{found}");
            }
        }

        println!("\n\n");
        if !self.is_app_dep.unwrap() {
            println!("Dependency is not used in runtime. No need to update");
        } else {
            println!("Dependency is used in runtime. Please update it");
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Analysis {
    fn default() -> Self {
        Self {
            update_recommended: false,
            is_used: false,
            is_direct_dep: false,
            direct_deps: vec![],
            is_build_only: false,
            is_app_dep: None,
            app_imports: None,
            is_build_js_dep: None,
            build_js_imports: None,
        }
    }
}

fn run_npm(args: &args::Args, analysis: &mut Analysis) {
    let npm_ls = if let Some(npm) = NpmLs::new(args.dependency()) {
        analysis.is_used = true;
        npm
    } else {
        analysis.is_used = false;
        return;
    };

    let mut all_builds = true;
    analysis.direct_deps = npm_ls
        .top_level_deps()
        .iter()
        .map(|dep| str::to_string(*dep))
        .collect();

    for dep in npm_ls.top_level_deps() {
        if !DEP_BUILD_ALLOW_LIST.contains(&dep) {
            all_builds = false;
        }
    }

    analysis.is_build_only = all_builds;
    analysis.is_direct_dep = npm_ls.is_top_level_dep();
}

fn run_import(args: &args::Args, analysis_out: &mut Analysis) {
    let mut analysis = ImportAnalyzer::new(args.path(), args.dependency());
    let founds = analysis.analyze("build.js");
    analysis_out.build_js_imports = Some(founds.clone());

    if founds.is_empty() {
        analysis_out.is_build_js_dep = Some(false);
    } else {
        analysis_out.is_build_js_dep = Some(true);
    };

    let founds = analysis.analyze("src/index.tsx");
    analysis_out.app_imports = Some(founds.clone());
    if founds.is_empty() {
        analysis_out.is_app_dep = Some(false);
    } else {
        analysis_out.is_app_dep = Some(true);
        analysis_out.update_recommended = true;
    }
}

fn main() {
    let args = args::Args::parse();
    let mut analysis = Analysis::default();
    run_npm(&args, &mut analysis);

    if !args.no_imports() && analysis.is_used && !analysis.is_build_only {
        let obs = Obs::new(&args);
        if obs.initialize_obs() {
            vprintln!(args, "Analyzing obs project");
        } else {
            vprintln!(args, "Analyzing regular project");
        }
        run_import(&args, &mut analysis);
    }

    if args.json() {
        println!("{}", serde_json::to_string(&analysis).unwrap());
    } else {
        analysis.print(&args);
    }
}
