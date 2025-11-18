use std::env;
use std::path::Path;

use calamine::{open_workbook, Data, Error, Xlsx, Reader, RangeDeserializerBuilder};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    DateTime(calamine::ExcelDateTime),
    Empty,
    Error(calamine::CellErrorType),
}

pub fn extract_value(data: &Data) -> Value {
    match data {
        Data::Int(n)        => Value::Int(*n),
        Data::Float(f)      => Value::Float(*f),
        Data::Bool(b)       => Value::Bool(*b),
        Data::String(s)     => Value::Text(s.clone()),
        Data::DateTime(dt)  => Value::DateTime(dt.clone()),
        Data::DateTimeIso(s)    => Value::Text(s.clone()),      // added
        Data::DurationIso(s)    => Value::Text(s.clone()),      // added
        Data::Empty         => Value::Empty,
        Data::Error(err)    => Value::Error(err.clone())
    }
}

// read different types into a common table type
fn read_csv() {
    // TODO: implement reading CSV
}

fn read_xlsx(file: &str) -> Result<Vec<Vec<Value>>, Error> {
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
    
    let mut columns: Vec<Vec<Value>> = vec![Vec::new(); range.width()];
    
    for row in range.rows().skip(1) {
        for (i, cell) in row.iter().enumerate() {
            columns[i].push(extract_value(cell));
        }
    }

    Ok(columns)
}

fn sort_table(mut table: Vec<Vec<Value>>, column: usize) -> Result<Vec<Vec<Value>>, Error> {
    // convert column-major (Vec<Vec<Value>>) into row-major (Vec<Vec<Value>>)
    let row_count = table.get(0).map(|c| c.len()).unwrap_or(0);
    let col_count = table.len();

    let mut rows: Vec<Vec<Value>> = (0..row_count)
        .map(|r| {
            (0..col_count)
                .map(|c| table[c][r].clone())
                .collect()
        })
        .collect();

    // sort rows by the target column
    rows.sort_by(|a, b| {
        match (&a[column], &b[column]) {
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            // fallback for mixed types
            _ => std::cmp::Ordering::Equal,
        }
    });

    // convert back: row-major -> column-major
    let mut sorted_cols = vec![Vec::new(); col_count];
    for row in rows {
        for (c, val) in row.into_iter().enumerate() {
            sorted_cols[c].push(val);
        }
    }

    Ok(sorted_cols)
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
                println!("Column:");
                for cell in &columns[5] {
                    println!("{:#?}", cell);
                }
                
                println!("SORTING COLUMNS");

                let sorted = sort_table(columns, 5).unwrap();
                for cell in &sorted[5] {
                    println!("{:#?}", cell);
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
