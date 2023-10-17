use std::path::{Path, PathBuf};

use clap::Parser;
use inkwell::{context::Context, module::Module, support::LLVMString};
use std::{
    fs::{File, OpenOptions},
    io::Write,
};
use walkdir::WalkDir;
/// Read some lines of a file
#[derive(Debug, Parser)]
struct Cli {
    output: String,
}
fn main() {
    let args = Cli::parse();
    let cwd = std::env::current_dir().unwrap();
    let sum_path = Path::new(&args.output);
    let mut logger = Logger::new(&cwd);
    let context = Context::create();
    let summation_module = context.create_module("sum");
    for entry in WalkDir::new(cwd).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if let Some(extension) = entry.path().extension() {
                if extension == "bc" {
                    if let Ok(module) = Module::parse_bitcode_from_path(entry.path(), &context) {
                        let status = summation_module.link_in_module(module.clone());
                        logger.log_bytecode(entry.path(), status);
                    }
                }
            }
        }
    }
    summation_module.write_bitcode_to_path(sum_path);
}

struct Logger {
    bc_file: File,
}
impl Logger {
    fn new(cwd: &PathBuf) -> Self {
        let mut bc_path = cwd.clone();
        bc_path.push("llvm_bc.csv");
        if bc_path.exists() {
            std::fs::remove_file(&bc_path).unwrap();
        }

        let bc_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(bc_path)
            .unwrap();

        Logger { bc_file }
    }

    fn log_bytecode(&mut self, path: &Path, status: Result<(), LLVMString>) {
        let failed = u8::from(status.is_err());
        let message = status
            .as_ref()
            .err()
            .map(|e| e.to_string())
            .unwrap_or("".to_string());
        let entry = format!("{},{},{}\n", path.to_string_lossy(), failed, message);
        let file = &mut self.bc_file;
        file.write_all(entry.as_bytes()).unwrap();
        file.flush().unwrap();
    }
}
