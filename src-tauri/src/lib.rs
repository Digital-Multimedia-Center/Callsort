use serde::Deserialize;
use std::fs;
use csv;

#[derive(Deserialize)]
pub struct SortArgs {
    pub input_path: String,
    pub column_name: String,
    pub output_path: String,
}

fn normalize_call_number(call_number: &str) -> String {
    let mut result = call_number.to_string();
    result = regex::Regex::new(r"([A-Z]+) (\d+)")
        .unwrap()
        .replace_all(&result, "$1$2")
        .to_string();
    result = regex::Regex::new(r"(\d) \.")
        .unwrap()
        .replace_all(&result, "$1.")
        .to_string();
    result.trim().to_string()
}

fn loc_sort_key(call_number: &str) -> String {
    let re = regex::Regex::new(
        r#"^\s*([A-Z]{1,3})(\d{1,4})\.?(\d{1,3})?\s*\.?([A-Z])(\d+)\s*([A-Z]{1,2})?(\d+)?\s*(\d{4})?(.*)?"#
    ).unwrap();
    if let Some(caps) = re.captures(call_number) {
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
    let contents = fs::read_to_string(&args.input_path).map_err(|e| e.to_string())?;

    let mut reader = csv::Reader::from_reader(contents.as_bytes());
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();

    let column_index = headers
        .iter()
        .position(|h| h == args.column_name)
        .ok_or_else(|| format!("Column '{}' not found", args.column_name))?;

    let mut records: Vec<csv::StringRecord> = reader
        .records()
        .filter_map(Result::ok)
        .collect();

    records.sort_by_cached_key(|record| loc_sort_key(&normalize_call_number(&record[column_index])));

    let mut wtr = csv::Writer::from_path(&args.output_path).map_err(|e| e.to_string())?;
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for r in records {
        wtr.write_record(&r).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;

    Ok(format!("File sorted and saved to: {}", args.output_path))
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
