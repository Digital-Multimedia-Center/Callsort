use std::fs::File;
use std::path::Path;
use regex::Regex;

use std::sync::Mutex;
use tauri::State;

use calamine::{open_workbook, Data, Reader, Xlsx};
use rust_xlsxwriter::Workbook;
use csv::{Writer, Reader as CsvReader};
use std::error::Error;
use std::cmp::Ordering;

use serde::Serialize;

// STATE FOR TABLES
struct StoredTables(Mutex<Vec<Table>>);

#[derive(Debug, Clone, Serialize)]
struct Table {
    headers: Vec<Value>,
    rows: Vec<Vec<Value>>,
}

// Value enum for reading different types from Excel or CSV
#[derive(Debug, Clone, Serialize)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    DateTime(String),
    Empty,
    Error(String),
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
        Data::Int(n) => Value::Int(*n),
        Data::Float(f) => Value::Float(*f),
        Data::Bool(b) => Value::Bool(*b),
        Data::String(s) => Value::Text(s.clone()),
        Data::DateTime(dt) => Value::DateTime(format!("{:?}", dt)), // convert to string
        Data::DateTimeIso(s) => Value::Text(s.clone()),
        Data::DurationIso(s) => Value::Text(s.clone()),
        Data::Empty => Value::Empty,
        Data::Error(e) => Value::Error(format!("{:?}", e)),
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
fn read_csv(file: &str) -> Result<Table, Box<dyn Error>> {
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

    Ok(Table{headers, rows})
}

// Read XLSX into Value table
fn read_xlsx(file: &str) -> Result<Table, Box<dyn Error>> {
    let path = Path::new(file);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheets = workbook.sheet_names().to_owned();
    let sheet_name = match sheets.first() {
        Some(name) => name.as_str(),
        None => return Ok(Table {
                    headers: Vec::new(),
                    rows: Vec::new(),
                })
    };

    let range = workbook.worksheet_range(sheet_name)?;
    let mut sheet: Vec<Vec<Value>> = Vec::new();
    let headers: Vec<Value> = range.rows().next()
        .map(|row| row.iter().map(|cell| extract_value(cell)).collect())
        .unwrap_or_default();

    for row in range.rows().skip(1) {
        sheet.push(row.iter().map(|cell| extract_value(cell)).collect());
    }

    Ok(Table{headers, rows: sheet})
}

// Sort the table by a given column
fn sort_table(state: &StoredTables, column: usize) -> Result<(), Box<dyn Error>> {
    let re = Regex::new(
        r"^\s*([A-Z]{1,3})([0-9]{1,4})\.?([0-9]{1,3})?\s*\.?([A-Z])([0-9]+)\s*(?:([A-Z]{1,2})([0-9]+)?)?\s*([0-9]{4})?(.*)?"
    )?;

    let mut tables = state.0.lock().unwrap();

    for table in tables.iter_mut() {
        table.rows.sort_by(|a, b| {
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
    }
    Ok(()) 
}

// Write table to CSV
fn output_csv(table: &Table, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);

    // Write headers
    let header_row: Vec<String> = table.headers.iter().map(|v| match v {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Text(s) => s.clone(),
        Value::DateTime(dt) => format!("{:?}", dt),
        Value::Empty => String::new(),
        Value::Error(e) => format!("{:?}", e),
    }).collect();

    wtr.write_record(&header_row)?;

    // Write rows
    for row in &table.rows {
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
fn output_xlsx(table: &Table, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    for (col_idx, header) in table.headers.iter().enumerate() {
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

    for (row_idx, row) in table.rows.iter().enumerate() {
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
#[tauri::command(rename_all = "snake_case")]
fn preview_tables(state: State<StoredTables>) -> Result<Vec<Table>, String> {
    let tables = state.0.lock().map_err(|_| "Failed to lock state")?;
    let previews: Vec<Table> = tables
        .iter()
        .map(|table| Table {
            headers: table.headers.clone(),
            rows: table.rows.iter().take(20).cloned().collect(),
        })
        .collect();
    Ok(previews)
}

#[tauri::command(rename_all = "snake_case")]
fn sort_file(
    column_indices: Vec<usize>,
    output_paths: Vec<String>,
    output_format: String,
    state: State<StoredTables>,
) -> Result<String, String> 
{
    let mut tables = state.0.lock().map_err(|_| "Failed to lock state")?;

    if tables.is_empty() {
        return Err("No tables are loaded in state".to_string());
    }

    if tables.len() != column_indices.len() {
        return Err("Number of column indices does not match number of loaded tables".to_string());
    }

    if tables.len() != output_paths.len() {
        return Err("Number of output paths does not match number of loaded tables".to_string());
    }

    for (i, table) in tables.iter_mut().enumerate() {
        let col = column_indices[i];

        if col >= table.headers.len() {
            return Err(format!("Column index {} is out of bounds for table {}", col, i));
        }

        let re = Regex::new(
            r"^\s*([A-Z]{1,3})([0-9]{1,4})\.?([0-9]{1,3})?\s*\.?([A-Z])([0-9]+)\s*(?:([A-Z]{1,2})([0-9]+)?)?\s*([0-9]{4})?(.*)?"
        ).map_err(|e| e.to_string())?;

        table.rows.sort_by(|a, b| {
            let sa = match &a[col] {
                Value::Text(s) => s.as_str(),
                _ => "",
            };
            let sb = match &b[col] {
                Value::Text(s) => s.as_str(),
                _ => "",
            };
            loc_sort_key(sa, &re).cmp(&loc_sort_key(sb, &re))
        });
    }

    for (i, table) in tables.iter().enumerate() {
        let path = &output_paths[i];
        let fmt = output_format.to_lowercase();

        match fmt.as_str() {
            "csv" => output_csv(table, path).map_err(|e| e.to_string())?,
            "xlsx" => output_xlsx(table, path).map_err(|e| e.to_string())?,
            _ => return Err("Unsupported output format, must be 'csv' or 'xlsx'".to_string()),
        }
    }

    Ok("All tables sorted and written successfully".to_string())
}

#[tauri::command(rename_all = "snake_case")]
fn read_input(input_paths: Vec<String>, state: State<StoredTables>) -> Result<(), String> {
    use std::path::Path;
    let mut stored = state.0.lock().map_err(|_| "Failed to lock state")?;
    stored.clear();

    for path in input_paths {
        let ext = Path::new(&path)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();

        let table = match ext.as_str() {
            "csv" => read_csv(&path),
            "xlsx" => read_xlsx(&path),
            other => return Err(format!("Unsupported input file type: {}", other)),
        }
        .map_err(|e| e.to_string())?;

        stored.push(table);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(StoredTables(Mutex::new(Vec::new())))
        .invoke_handler(tauri::generate_handler![sort_file,read_input, preview_tables])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

