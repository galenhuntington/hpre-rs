use std::env;
use std::fs;
use std::io::{self, Read};

fn abort(msg: &str) -> ! {
    eprintln!("hpre: {}", msg);
    std::process::exit(1);
}

fn run_process(input: &str) -> String {
    match hpre::process(input) {
        Ok(output) => output,
        Err(msg) => abort(&msg),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap_or_else(|e| {
                abort(&format!("Cannot read stdin: {}", e));
            });
            print!("{}", run_process(&input));
        }
        2 if args[1] == "--version" => {
            println!("hpre v{}", env!("CARGO_PKG_VERSION"));
        }
        4 => {
            let name = &args[1];
            let infile = &args[2];
            let outfile = &args[3];
            let input = fs::read_to_string(infile).unwrap_or_else(|e| {
                abort(&format!("Cannot read {}: {}", infile, e));
            });
            let output = format!("{{-# LINE 1 \"{}\" #-}}\n{}", name, run_process(&input));
            fs::write(outfile, output).unwrap_or_else(|e| {
                abort(&format!("Cannot write {}: {}", outfile, e));
            });
        }
        _ => {
            abort("Usage: hpre [name infile outfile]");
        }
    }
}
