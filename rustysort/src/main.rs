use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();


    if args.len() < 3 {
        eprintln!("Not enough arguments provided, give a file and output destination");
        std::process::exit(1);
    }
    
    let input_file = &args[1];
    let _output_folder = &args[2];

    let ext = Path::new(input_file)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if ext != "csv" && ext != "xlsx" {
        eprintln!("Invalid file type: expected .csv or .xlsx, got {}", input_file);
        std::process::exit(1);
    }
}
