use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use regex::Regex;

use calamine::{open_workbook, Data, Reader, Xlsx};
use rust_xlsxwriter::Workbook;
use csv::{Writer, Reader as CsvReader};
use std::error::Error;
use std::cmp::Ordering;

#[derive(Deserialize)]
pub struct SortArgs {
    pub input_path: String,
    pub column_name: String,
    pub output_path: String,
    pub output_format: String, // "csv" or "xlsx"
}

// Value enum for reading different types from Excel or CSV
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

// Struct for library call number sorting
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

// Extract Value from calamine Data
pub fn extract_value(data: &Data) -> Value {
    match data {
        Data::Int(n)        => Value::Int(*n),
        Data::Float(f)      => Value::Float(*f),
        Data::Bool(b)       => Value::Bool(*b),
        Data::String(s)     => Value::Text(s.clone()),
        Data::DateTime(dt)  => Value::DateTime(dt.clone()),
        Data::DateTimeIso(s) => Value::Text(s.clone()),
        Data::DurationIso(s) => Value::Text(s.clone()),
        Data::Empty         => Value::Empty,
        Data::Error(err)    => Value::Error(err.clone()),
    }
}

// Generate sorting key for LOC call numbers
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

// Implement sorting for LocKey
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

// Read CSV into Value table
fn read_csv(file: &str) -> Result<(Vec<Vec<Value>>, Vec<Value>), Box<dyn Error>> {
    let file = File::open(file)?;
    let mut rdr = CsvReader::from_reader(file);

    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut headers: Vec<Value> = Vec::new();

    if let Some(result) = rdr.headers().ok() {
        headers = result.iter().map(|s| Value::Text(s.to_string())).collect();
    }

    for result in rdr.records() {
        let record = result?;
        let row: Vec<Value> = record.iter().map(|cell| {
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

// Read XLSX into Value table
fn read_xlsx(file: &str) -> Result<(Vec<Vec<Value>>, Vec<Value>), Box<dyn Error>> {
    let path = Path::new(file);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheets = workbook.sheet_names().to_owned();
    let sheet_name = match sheets.first() {
        Some(name) => name.as_str(),
        None => return Ok((Vec::new(), Vec::new())),
    };

    let range = workbook.worksheet_range(sheet_name)?;
    let mut sheet: Vec<Vec<Value>> = Vec::new();
    let headers: Vec<Value> = range.rows().next()
        .map(|row| row.iter().map(|cell| extract_value(cell)).collect())
        .unwrap_or_default();

    for row in range.rows().skip(1) {
        sheet.push(row.iter().map(|cell| extract_value(cell)).collect());
    }

    Ok((sheet, headers))
}

// Sort the table by a given column
fn sort_table(table: &mut Vec<Vec<Value>>, column: usize) -> Result<(), Box<dyn Error>> {
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
        loc_sort_key(sa, &re).cmp(&loc_sort_key(sb, &re))
    });

    Ok(())
}

// Write table to CSV
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

// Write table to XLSX
fn output_xlsx(table: &Vec<Vec<Value>>, headers: &Vec<Value>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col_idx, header) in headers.iter().enumerate() {
        let value = match header {
            Value::Text(s) => s.clone(),
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::DateTime(dt) => format!("{:?}", dt),
            Value::Empty => String::new(),
            Value::Error(e) => format!("{:?}", e),
        };
        worksheet.write_string(0, col_idx as u16, &value)?;
    }

    for (row_idx, row) in table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            match cell {
                Value::Int(n) => worksheet.write_number((row_idx + 1) as u32, col_idx as u16, *n as f64)?,
                Value::Float(f) => worksheet.write_number((row_idx + 1) as u32, col_idx as u16, *f)?,
                Value::Bool(b) => worksheet.write_boolean((row_idx + 1) as u32, col_idx as u16, *b)?,
                Value::Text(s) => worksheet.write_string((row_idx + 1) as u32, col_idx as u16, s)?,
                Value::DateTime(dt) => worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &format!("{:?}", dt))?,
                Value::Empty => worksheet,
                Value::Error(e) => worksheet.write_string((row_idx + 1) as u32, col_idx as u16, &format!("{:?}", e))?,
            };
        }
    }

    workbook.save(path)?;
    Ok(())
}

// Tauri command
#[tauri::command]
fn sort_file(args: SortArgs) -> Result<String, String> {
    use std::path::Path;

    let ext = Path::new(&args.input_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let result = match ext.as_str() {
        "csv" => read_csv(&args.input_path),
        "xlsx" => read_xlsx(&args.input_path),
        _ => return Err(format!("Unsupported input file type: {}", ext)),
    };

    let (mut rows, headers) = result.map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Err("Input file contains no data".to_string());
    }

    let column_index = headers.iter()
        .position(|v| match v {
            Value::Text(s) => s == &args.column_name,
            _ => false,
        })
        .ok_or_else(|| format!("Column '{}' not found", args.column_name))?;

    sort_table(&mut rows, column_index).map_err(|e| e.to_string())?;

    let output_file_path = Path::new(&args.output_path);

    match args.output_format.to_lowercase().as_str() {
        "csv" => output_csv(&rows, &headers, output_file_path.to_str().unwrap())
            .map_err(|e| e.to_string())?,
        "xlsx" => output_xlsx(&rows, &headers, output_file_path.to_str().unwrap())
            .map_err(|e| e.to_string())?,
        _ => return Err("Unsupported output format, must be 'csv' or 'xlsx'".to_string()),
    }

    Ok(format!("File sorted and saved to: {}", output_file_path.display()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![sort_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

