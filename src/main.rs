use clap::Parser;

use crate::{imports::ImportAnalyzer, npm::NpmLs};

mod args;
mod imports;
mod npm;

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

fn main() {
    let args = args::Args::parse();

    let npm_ls = NpmLs::new(args.dependency());
    // println!("Depedency tree for '{}':", args[1]);
    // println!("{npm_ls:#?}");
    println!("Dependency info for '{}':\n", args.dependency());

    let mut all_builds = true;
    println!("Used by following project dependencies:");
    for dep in npm_ls.top_level_deps() {
        print!("\t{dep}");
        if DEP_BUILD_ALLOW_LIST.contains(&dep) {
            print!(" (known build dependency)")
        } else {
            all_builds = false;
        }
        println!();
    }

    println!();
    if npm_ls.is_top_level_dep() {
        println!("Dependency is a direct dependency");
    } else {
        println!("Dependency is NOT a direct dependency");
    }

    println!();
    if all_builds {
        println!("All dependencies are known build dependencies");
    } else {
        println!("All dependencies are NOT known build dependencies");
    }

    if !args.no_imports() {
        println!();
        let analysis = ImportAnalyzer::new(args.path(), args.dependency());
        let founds = analysis.analyze("build.js");
        if founds.is_empty() {
            println!("Dependency is not used by build.js");
        } else {
            println!("Dependency is used by build.js in following paths");
            for found in founds {
                println!("\t{found}");
            }
        }

        println!();
        let founds = analysis.analyze("src/index.tsx");
        if founds.is_empty() {
            println!("Dependency is not used by the application");
        } else {
            println!("Dependency is used by application in following paths");
            for found in founds {
                println!("\t{found}");
            }
        }
    }
}
