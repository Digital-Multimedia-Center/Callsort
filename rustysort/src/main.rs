use std::env;
use std::fs::File;
use std::path::Path;
use std::cmp::Ordering;
use std::error::Error;

use calamine::{open_workbook, Data, Error as CalamineError, Xlsx, Reader, RangeDeserializerBuilder};
use rust_xlsxwriter::{Workbook, XlsxError};
use csv::{Writer, Reader as CsvReader};
use regex::Regex;

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

#[derive(Debug)]
struct LocKey<'a> {
    class_letters: &'a str,
    class_number: i32,
    decimal_part: i32,
    cutter1_letter: &'a str,
    cutter1_number: i32,
    cutter2_letter: &'a str,
    cutter2_number: i32,
    year: i32,
    trailing: &'a str,
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

fn loc_sort_key<'a>(s: &'a str, re: &Regex) -> LocKey<'a> {
    if let Some(caps) = re.captures(s) {
        let g = |i| caps.get(i).map(|m| m.as_str()).unwrap_or("");

        LocKey {
            class_letters: g(1),
            class_number: g(2).parse().unwrap_or(0),
            decimal_part: g(3).parse().unwrap_or(-1),
            cutter1_letter: g(4),
            cutter1_number: g(5).parse().unwrap_or(0),
            cutter2_letter: g(6),
            cutter2_number: g(7).parse().unwrap_or(0),
            year: g(8).parse().unwrap_or(0),
            trailing: g(9),
        }
    } else {
        LocKey {
            class_letters: s,
            class_number: 0,
            decimal_part: -1,
            cutter1_letter: "",
            cutter1_number: 0,
            cutter2_letter: "",
            cutter2_number: 0,
            year: 0,
            trailing: "",
        }
    }
}


impl<'a> Ord for LocKey<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.class_letters,
            self.class_number,
            self.decimal_part,
            self.cutter1_letter,
            self.cutter1_number,
            self.cutter2_letter,
            self.cutter2_number,
            self.year,
            self.trailing,
        )
            .cmp(&(
                other.class_letters,
                other.class_number,
                other.decimal_part,
                other.cutter1_letter,
                other.cutter1_number,
                other.cutter2_letter,
                other.cutter2_number,
                other.year,
                other.trailing,
            ))
    }
}

impl<'a> PartialOrd for LocKey<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for LocKey<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<'a> Eq for LocKey<'a> {}

// read different types into a common table type
fn read_csv(file: &str) -> Result<(Vec<Vec<Value>>, Vec<Value>), Box<dyn Error>> {
    let file = File::open(file)?;
    let mut rdr = CsvReader::from_reader(file);

    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut headers: Vec<Value> = Vec::new();

    // Read headers
    if let Some(result) = rdr.headers().ok() {
        headers = result.iter().map(|s| Value::Text(s.to_string())).collect();
    }

    // Read records
    for result in rdr.records() {
        let record = result?;
        let row: Vec<Value> = record.iter().map(|cell| {
            // Try parsing each value as number or boolean first
            if let Ok(n) = cell.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(f) = cell.parse::<f64>() {
                Value::Float(f)
            } else if let Ok(b) = cell.parse::<bool>() {
                Value::Bool(b)
            } else if cell.is_empty() {
                Value::Empty
            } else {
                Value::Text(cell.to_string())
            }
        }).collect();

        rows.push(row);
    }

    Ok((rows, headers))
}

fn read_xlsx(file: &str) -> Result<(Vec<Vec<Value>>, Vec<Value>), Box<dyn Error>> {
    let path = Path::new(file);
    let mut workbook: Xlsx<_> = open_workbook(path)?;

    let sheets = workbook.sheet_names().to_owned();
    let sheet_name = match sheets.first() {
        Some(name) => name.as_str(),
        None => {
            eprintln!("No sheets found in the workbook");
            return Ok((Vec::new(), Vec::new()));
        }
    };

    let range = workbook.worksheet_range(sheet_name)?; // This returns calamine::Result<Range<Data>>

    let mut sheet: Vec<Vec<Value>> = Vec::new();

    // Extract headers
    let headers: Vec<Value> = match range.rows().next() {
        Some(row) => row.iter().map(|cell| extract_value(cell)).collect(),
        None => Vec::new(),
    };

    // Extract data rows
    for row in range.rows().skip(1) {
        let row_values: Vec<Value> = row.iter().map(|cell| extract_value(cell)).collect();
        sheet.push(row_values);
    }

    Ok((sheet, headers))
}

fn sort_table(table: &mut Vec<Vec<Value>>, column: usize) -> Result<(), Box<dyn Error>> {
    let row_count = table[0].len();
    let col_count = table.len();


    let re = Regex::new(
        r"^\s*([A-Z]{1,3})([0-9]{1,4})\.?([0-9]{1,3})?\s*\.?([A-Z])([0-9]+)\s*(?:([A-Z]{1,2})([0-9]+)?)?\s*([0-9]{4})?(.*)?"
    ).unwrap();

    table.sort_by(|a, b| {
        let sa = match &a[column] {
            Value::Text(s) => s.as_str(),
            _ => "",
        };

        let sb = match &b[column] {
            Value::Text(s) => s.as_str(),
            _ => "",
        };

        let ka = loc_sort_key(sa, &re);
        let kb = loc_sort_key(sb, &re);

        ka.cmp(&kb)
    });

    Ok(())
}


// export intermediate table into desired file
fn output_csv(table: &Vec<Vec<Value>>, headers: &Vec<Value>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);

    let header_row: Vec<String> = headers.iter().map(|v| match v {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Text(s) => s.clone(),
        Value::DateTime(dt) => format!("{:?}", dt),
        Value::Empty => String::new(),
        Value::Error(e) => format!("{:?}", e),
    }).collect();

    wtr.write_record(&header_row)?;

    for row in table {
        let record: Vec<String> = row.iter().map(|v| match v {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Text(s) => s.clone(),
            Value::DateTime(dt) => format!("{:?}", dt),
            Value::Empty => String::new(),
            Value::Error(e) => format!("{:?}", e),
        }).collect();

        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn output_xlsx(
    table: &Vec<Vec<Value>>,
    headers: &Vec<Value>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Write headers as strings
    for (col_idx, header) in headers.iter().enumerate() {
        let value = match header {
            Value::Text(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::DateTime(dt) => format!("{:?}", dt), // Or format as string
            Value::Empty => String::new(),
            Value::Error(e) => format!("{:?}", e),
        };
        worksheet.write_string(0, col_idx as u16, &value)?;
    }

    // Write table rows in appropriate type
    for (row_idx, row) in table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            match cell {
                Value::Int(n) => {
                    worksheet.write_number((row_idx + 1) as u32, col_idx as u16, *n as f64)?;
                }
                Value::Float(f) => {
                    worksheet.write_number((row_idx + 1) as u32, col_idx as u16, *f)?;
                }
                Value::Bool(b) => {
                    worksheet.write_boolean((row_idx + 1) as u32, col_idx as u16, *b)?;
                }
                Value::Text(s) => {
                    worksheet.write_string((row_idx + 1) as u32, col_idx as u16, s)?;
                }
                Value::DateTime(dt) => {
                    let dt_str = format!("{:?}", dt);
                    worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &dt_str)?;
                }
                Value::Empty => {
                    // worksheet.write_blank((row_idx + 1) as u32, col_idx as u16)?;
                }
                Value::Error(e) => {
                    let e_str = format!("{:?}", e);
                    worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &e_str)?;
                }
            }
        }
    }

    workbook.save(path)?;
    Ok(())
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    let ext = Path::new(input_file)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Determine input file type
    let result: Result<(Vec<Vec<Value>>, Vec<Value>), Box<dyn std::error::Error>> = match ext.as_str() {
        "xlsx" => read_xlsx(input_file),
        "csv" => read_csv(input_file),
        _ => {
            eprintln!("Unsupported file type: expected .csv or .xlsx, got {}", input_file);
            std::process::exit(1);
        }
    };

    // Handle reading errors
    let (mut rows, headers) = match result {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", input_file, e);
            std::process::exit(1);
        }
    };

    if rows.is_empty() {
        println!("No data found in '{}'", input_file);
        std::process::exit(0);
    }

    // Example: sort by column 12 if it exists
    let sort_column_index = 12;
    if rows[0].len() > sort_column_index {
        println!("Column before sorting:");
        for row in rows.iter().take(10) {
            println!("{:#?}", row[sort_column_index]);
        }

        if let Err(e) = sort_table(&mut rows, sort_column_index) {
            eprintln!("Failed to sort table: {}", e);
        } else {
            println!("Column after sorting:");
            for row in rows.iter().take(10) {
                println!("{:#?}", row[sort_column_index]);
            }
        }
    }

    // Determine output type by extension
    let output_ext = Path::new(output_file)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let write_result = match output_ext.as_str() {
        "xlsx" => output_xlsx(&rows, &headers, output_file),
        "csv" => output_csv(&rows, &headers, output_file),
        _ => {
            eprintln!("Unsupported output file type: '{}'", output_file);
            std::process::exit(1);
        }
    };

    if let Err(e) = write_result {
        eprintln!("Failed to write output '{}': {}", output_file, e);
        std::process::exit(1);
    } else {
        println!("Output successfully written to '{}'", output_file);
    }
}

