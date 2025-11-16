use std::env;
use std::path::Path;

use calamine::{open_workbook, Data, Error, Xlsx, Reader, RangeDeserializerBuilder};

// read different types into a common table type
fn read_csv() {
    // TODO: implement reading CSV
}

fn read_xlsx(file: &str) -> Result<Vec<Vec<Data>>, Error> {
    let path = Path::new(file);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheets = workbook.sheet_names().to_owned();
    let sheet_name = match sheets.first() {
        Some(name) => name.as_str(),
        None => {
            eprintln!("No sheets found in the workbook");
            return Ok(vec![]);
        }
    };
    let range = workbook.worksheet_range(sheet_name)?;
    
    let mut columns: Vec<Vec<Data>> = vec![Vec::new(); range.width()];
    
    for row in range.rows().skip(1) {
        for (i, cell) in row.iter().enumerate() {
            columns[i].push(cell.clone());
        }
    }

    Ok(columns)
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
    match read_xlsx(input_file) {
        Ok(columns) => {
            if !columns.is_empty() {
                println!("First column:");
                for cell in &columns[0] {
                    println!("{}", cell);
                }
            } else {
                println!("No data found in the first column.");
            }
        }
        Err(e) => {
            eprintln!("Error reading XLSX: {}", e);
            std::process::exit(1);
        }
    }
}

}
