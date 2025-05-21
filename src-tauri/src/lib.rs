use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

use calamine::{open_workbook_auto, Reader, Data};

fn read_excel_column(
    path: &str,
    _column_name: &str,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No worksheet found")?
        .map_err(|e| e.to_string())?;

    let mut headers: Vec<String> = Vec::new();
    let mut data: Vec<Vec<String>> = Vec::new();

    for (i, row) in range.rows().enumerate() {
        let string_row: Vec<String> = row
            .iter()
            .map(|cell| match cell {
                Data::Empty => "".to_string(),
                other => other.to_string(),
            })
            .collect();

        if i == 0 {
            headers = string_row;
        } else {
            data.push(string_row);
        }
    }

    Ok((headers, data))
}

#[derive(Deserialize)]
pub struct SortArgs {
    pub input_path: String,
    pub column_name: String,
    pub output_path: String,
}

static RE_ALPHA_NUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([A-Z]+) (\d+)").unwrap()
});

static RE_DOT_SPACE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\d) \.").unwrap()
});

static LOC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"^\s*([A-Z]{1,3})(\d{1,4})\.?(\d{1,3})?\s*\.?([A-Z])(\d+)\s*([A-Z]{1,2})?(\d+)?\s*(\d{4})?(.*)?"#
    ).unwrap()
});

fn normalize_call_number(call_number: &str) -> String {
    let mut result = call_number.to_string();
    result = RE_ALPHA_NUM.replace_all(&result, "$1$2").to_string();
    result = RE_DOT_SPACE.replace_all(&result, "$1.").to_string();
    result.trim().to_string()
}

fn loc_sort_key(call_number: &str) -> String {
    if let Some(caps) = LOC_RE.captures(call_number) {
        format!(
            "{: <3}{:0>4}{:0>3}{}{:0>5}{}{:0>5}{}{}",
            &caps[1],
            &caps[2],
            caps.get(3).map_or("000", |m| m.as_str()),
            caps.get(4).map_or("", |m| m.as_str()),
            caps.get(5).map_or("00000", |m| m.as_str()),
            caps.get(6).map_or("", |m| m.as_str()),
            caps.get(7).map_or("00000", |m| m.as_str()),
            caps.get(8).map_or("0000", |m| m.as_str()),
            caps.get(9).map_or("", |m| m.as_str())
        )
    } else {
        call_number.to_string()
    }
}

#[tauri::command]
fn sort_csv(args: SortArgs) -> Result<String, String> {
    let extension = Path::new(&args.input_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (headers, records): (Vec<String>, Vec<Vec<String>>) = match extension.as_str() {
        "xlsx" | "xls" | "xlsb" => read_excel_column(&args.input_path, &args.column_name)?,
        "csv" => {
            let file = File::open(&args.input_path).map_err(|e| e.to_string())?;
            let mut reader = csv::Reader::from_reader(BufReader::new(file));
            let headers = reader.headers().map_err(|e| e.to_string())?.clone();
            let records: Vec<Vec<String>> = reader
                .records()
                .filter_map(Result::ok)
                .map(|r| r.iter().map(|s| s.to_string()).collect())
                .collect();
            (headers.iter().map(|s| s.to_string()).collect(), records)
        }
        _ => return Err("Unsupported file format".into()),
    };

    let column_index = headers
        .iter()
        .position(|h| h == &args.column_name)
        .ok_or_else(|| format!("Column '{}' not found", args.column_name))?;

    let mut keyed_records: Vec<(String, Vec<String>)> = records
        .into_iter()
        .map(|record| {
            let call_number = record.get(column_index).unwrap_or(&"".to_string()).clone();
            let normalized = normalize_call_number(&call_number);
            let sort_key = loc_sort_key(&normalized);
            (sort_key, record)
        })
        .collect();

    keyed_records.sort_by(|a, b| a.0.cmp(&b.0));

    let sorted_records: Vec<Vec<String>> = keyed_records
        .into_iter()
        .map(|(_, record)| record)
        .collect();

    let input_filename = Path::new(&args.input_path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("Invalid input file name")?;

    let output_file_name = format!("{}_sorted.csv", input_filename);
    let mut output_file_path = PathBuf::from(&args.output_path);
    output_file_path.push(output_file_name);

    let mut wtr = csv::Writer::from_path(&output_file_path).map_err(|e| e.to_string())?;
    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    for record in sorted_records {
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;

    Ok(format!(
        "File sorted and saved to: {}",
        output_file_path.display()
    ))
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![sort_csv])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
