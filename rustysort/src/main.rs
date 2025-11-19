use std::env;
use std::path::Path;
use std::cmp::Ordering;

use calamine::{open_workbook, Data, Error, Xlsx, Reader, RangeDeserializerBuilder};
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
    
    let mut sheet: Vec<Vec<Value>> = Vec::new();
    
    for row in range.rows().skip(1) {
        let row_values: Vec<Value> = row
            .iter()
            .map(|cell| extract_value(cell))
            .collect();

        sheet.push(row_values);
    }

    Ok(sheet)
}

    fn sort_table(table: &mut Vec<Vec<Value>>, column: usize) -> Result<(), Error> {
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

    if ext == "xlsx" {
        match read_xlsx(input_file) {
            Ok(mut rows) => {
                if !rows.is_empty() {
                    // Define a column index
                    let sort_column_index = 12;
                    let MAX_PRINT = rows.len().min(10);

                    println!("Column:");

                    // PRINT RAW COLUMN
                    for row in &rows[0..MAX_PRINT] {
                        println!("{:#?}", row[sort_column_index]);

                    }
                    
                    println!("SORTING COLUMNS");

                    sort_table(&mut rows, sort_column_index).unwrap();

                    // PRINT SORTED COLUMN
                    for row in &rows[0..MAX_PRINT] {
                        println!("{:#?}", row[sort_column_index]);
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
