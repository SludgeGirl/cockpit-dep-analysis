use std::{env::args, process::exit};

use crate::npm::NpmLs;

mod npm;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Usage: {} <depedency name>", args[0]);
        exit(0);
    }

    let npm_ls = NpmLs::new(&args[1]);
    println!("Depedency tree for '{}':", args[1]);
    println!("{npm_ls:#?}");

    println!("Used by following project dependenies:");
    for dep in npm_ls.top_level_deps() {
        println!("\t{dep}");
    }

    println!();
    if npm_ls.is_top_level_dep() {
        println!("Depedency is a direct dependency");
    } else {
        println!("Depedency is NOT a direct dependency");
    }
}
