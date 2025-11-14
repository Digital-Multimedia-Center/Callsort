use std::env;
use std::path::Path;

use calamine::{open_workbook, Error, Xlsx, Reader, RangeDeserializerBuilder};

// read different types into a common table type
fn read_csv() {
    // TODO: implement reading CSV
}

fn read_xlsx(file: &str) -> Result<(), Error> {
    let path = Path::new(file);
    let mut workbook: Xlsx<_> = open_workbook(path)?;

    let sheets = workbook.sheet_names().to_owned();

    let sheet_name = match sheets.first() {
        Some(name) => name.as_str(),
        None => {
            eprintln!("No sheets found in the workbook");
            return Ok(());
        }
    };

    let range = workbook.worksheet_range(sheet_name)?;

    for row in range.rows() {
        println!("row={:?}, row[0]={:?}", row, row.get(0));
    }

    Ok(())
}

// export intermediate table into desired file
fn output_csv() {
    // TODO: implement writing CSV
}

fn output_xlsx() {
    // TODO: implement writing XLSX
}

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
        eprintln!("Unsupported file type: expected .csv or .xlsx, got {}", input_file);
        std::process::exit(1);
    }

    // Example usage (optional)
    if ext == "xlsx" {
        if let Err(e) = read_xlsx(input_file) {
            eprintln!("Error reading XLSX: {}", e);
            std::process::exit(1);
        }
    }
}
