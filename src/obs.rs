use std::{env, fs, process::Command};

use crate::{args::Args, vprintln};

pub struct Obs<'a> {
    args: &'a Args,
}

/// TODO: apply patches to sources
impl<'a> Obs<'a> {
    pub fn new(args: &'a Args) -> Self {
        Self { args }
    }

    /// return true if current directory is an obs project
    pub fn initialize_obs(&self) -> bool {
        if !self.is_obs_project() {
            return false;
        }

        let current_dir = env::current_dir().unwrap();
        let tar_file = self.find_tar().unwrap();
        let code_dir = self.code_folder(&tar_file);
        self.extract_tar(&tar_file);
        env::set_current_dir(code_dir).unwrap();

        self.install_obs_node_modules(current_dir.to_str().unwrap());
        self.install_pkg_and_build();

        true
    }

    fn is_obs_project(&self) -> bool {
        let paths = fs::read_dir("./").unwrap();

        for path in paths {
            let path = path.unwrap().path();
            if path.extension().is_some_and(|ex| ex == "spec") {
                vprintln!(
                    self.args,
                    "obs project detected: '{}' found",
                    path.display()
                );
                return true;
            }
        }

        vprintln!(self.args, "No obs project detected: .spec file not found");
        false
    }

    fn find_tar(&self) -> Result<String, String> {
        let paths = fs::read_dir("./").unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let path = path.to_str().unwrap();
            if path.ends_with(".tar.gz") || path.ends_with(".tar.xz") {
                vprintln!(self.args, "tar file '{}' found", path);
                return Ok(path.into());
            }
        }

        vprintln!(self.args, "Failed to find .tar.gz or .tar.xz file");

        Err("tar file not found".into())
    }

    /// Get Cockpit library from upstream.
    /// This is not always the extact version that's used in final build
    /// but it's a good approximation.
    fn install_pkg_and_build(&self) {
        vprintln!(self.args, "Installing cockpit.js and building the project");
        // local-npm-registry ../node_modules install --include=dev --ignore-scripts
        let _ = Command::new("make")
            .output()
            .expect("failed to run make")
            .stdout;
    }

    fn install_obs_node_modules(&self, current_dir: &str) {
        vprintln!(self.args, "Installing node modules via local-npm-registry");
        // local-npm-registry ../node_modules install --include=dev --ignore-scripts
        let _ = Command::new("local-npm-registry")
            .arg(format!("{current_dir}/node_modules"))
            .arg("install")
            .arg("--include=dev")
            .arg("--ignore-scripts")
            .output()
            .expect("failed to run local-npm-registry install")
            .stdout;
    }

    fn code_folder(&self, tar_name: &str) -> String {
        vprintln!(
            self.args,
            "Finding name of the source code folder in '{tar_name}'"
        );
        let tar_f_output = Command::new("tar")
            .arg("-tf")
            .arg(tar_name)
            .output()
            .expect("failed to run tar -tf")
            .stdout;
        let file_list = String::from_utf8_lossy(&tar_f_output);
        // assuming the tar file only has a single folder in it
        let folder_name = file_list.lines().next().unwrap();
        folder_name.into()
    }

    fn extract_tar(&self, tar_name: &str) {
        vprintln!(self.args, "extracting file: '{tar_name}'");
        let _ = Command::new("tar")
            .arg("-xf")
            .arg(tar_name)
            .output()
            .expect("failed to run tar -xf")
            .stdout;
    }
}
