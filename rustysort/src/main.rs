use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

use once_cell::sync::Lazy;
use regex::Regex;

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

#[derive(Deserialize)]
pub struct SortArgs {
    pub input_path: String,
    pub column_name: String,
    pub output_path: String,
}

fn main() {
    let args = SortArgs {
        input_path: "test/input.csv".to_string(),
        output_path: "test/temp.csv".to_string(),
        column_name: "item_effective_call_number".to_string(),
    };

    match sort_csv(args) {
        Ok(msg) => println!("{}", msg),
        Err(err) => eprintln!("Error: {}", err),
    }
}

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

fn sort_csv(args: SortArgs) -> Result<String, String> {
    // Use BufReader for efficient file reading
    let file = File::open(&args.input_path).map_err(|e| e.to_string())?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();

    let column_index = headers
        .iter()
        .position(|h| h == args.column_name)
        .ok_or_else(|| format!("Column '{}' not found", args.column_name))?;

    let records: Vec<csv::StringRecord> = reader
        .records()
        .filter_map(Result::ok)
        .collect();

    // Step 1: Create a vector of tuples (sort_key, record)
    let mut keyed_records: Vec<(String, csv::StringRecord)> = records
        .into_iter()
        .map(|record| {
            let call_number = &record[column_index];
            let normalized = normalize_call_number(call_number);
            let sort_key = loc_sort_key(&normalized);
            (sort_key, record)
        })
        .collect();

    // Step 2: Sort the tuples by the sort_key
    keyed_records.sort_by(|a, b| a.0.cmp(&b.0));

    // Step 3: Extract the sorted records
    let records: Vec<csv::StringRecord> = keyed_records
        .into_iter()
        .map(|(_, record)| record)
        .collect();

    let mut wtr = csv::Writer::from_path(&args.output_path).map_err(|e| e.to_string())?;
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for r in records {
        wtr.write_record(&r).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;

    Ok(format!("File sorted and saved to: {}", args.output_path))
}
