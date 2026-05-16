use crate::core::models::CollectionIndex;
use crate::core::normalization::normalize_name;
use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

const NAME_FIELD_CANDIDATES: &[&str] = &["Name", "Card Name", "CardName", "card_name", "name"];
const COUNT_FIELD_CANDIDATES: &[&str] = &[
    "Quantity", "Count", "Owned", "Total", "Qty", "Copies", "quantity", "count", "owned", "total",
];

#[derive(Clone, Debug)]
pub struct CollectionSchema {
    pub fields: Vec<String>,
    pub name_field: Option<String>,
    pub count_field: Option<String>,
    pub sample_rows: Vec<BTreeMap<String, String>>,
}

fn find_field(fieldnames: &[String], candidates: &[&str]) -> Option<String> {
    for candidate in candidates {
        if fieldnames.iter().any(|field| field == candidate) {
            return Some((*candidate).to_string());
        }
    }
    for candidate in candidates {
        let lowered = candidate.to_lowercase();
        if let Some(field) = fieldnames
            .iter()
            .find(|field| field.to_lowercase().trim() == lowered)
        {
            return Some(field.clone());
        }
    }
    None
}

pub fn inspect_collection_schema(path: impl AsRef<Path>) -> Result<CollectionSchema> {
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(path.as_ref())?;
    inspect_collection_schema_from_reader(reader)
}

pub fn inspect_collection_schema_from_str(text: &str) -> Result<CollectionSchema> {
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.as_bytes());
    inspect_collection_schema_from_reader(reader)
}

fn inspect_collection_schema_from_reader<R: Read>(
    mut reader: csv::Reader<R>,
) -> Result<CollectionSchema> {
    let fields: Vec<String> = reader.headers()?.iter().map(str::to_string).collect();
    let mut sample_rows = Vec::new();
    for record in reader.records().take(3) {
        let record = record?;
        let mut row = BTreeMap::new();
        for (field, value) in fields.iter().zip(record.iter()) {
            row.insert(field.clone(), value.to_string());
        }
        sample_rows.push(row);
    }

    Ok(CollectionSchema {
        name_field: find_field(&fields, NAME_FIELD_CANDIDATES),
        count_field: find_field(&fields, COUNT_FIELD_CANDIDATES),
        fields,
        sample_rows,
    })
}

pub fn parse_collection_csv(
    path: impl AsRef<Path>,
    name_field: Option<&str>,
    count_field: Option<&str>,
) -> Result<CollectionIndex> {
    let path = path.as_ref();
    let schema = inspect_collection_schema(path)?;
    let reader = csv::ReaderBuilder::new().flexible(true).from_path(path)?;
    parse_collection_csv_from_reader(
        reader,
        Some(path.to_string_lossy().to_string()),
        schema,
        name_field,
        count_field,
    )
}

pub fn parse_collection_csv_from_str(
    source_label: impl Into<String>,
    text: &str,
    name_field: Option<&str>,
    count_field: Option<&str>,
) -> Result<CollectionIndex> {
    let source_label = source_label.into();
    let schema = inspect_collection_schema_from_str(text)?;
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.as_bytes());
    parse_collection_csv_from_reader(reader, Some(source_label), schema, name_field, count_field)
}

fn parse_collection_csv_from_reader<R: Read>(
    mut reader: csv::Reader<R>,
    source_label: Option<String>,
    schema: CollectionSchema,
    name_field: Option<&str>,
    count_field: Option<&str>,
) -> Result<CollectionIndex> {
    let resolved_name = name_field
        .map(str::to_string)
        .or_else(|| schema.name_field.clone())
        .ok_or_else(|| {
            anyhow!(
                "Could not identify card-name field. Fields: {:?}",
                schema.fields
            )
        })?;
    let resolved_count = count_field
        .map(str::to_string)
        .or_else(|| schema.count_field.clone())
        .ok_or_else(|| {
            anyhow!(
                "Could not identify owned-count field. Fields: {:?}",
                schema.fields
            )
        })?;

    if !schema.fields.contains(&resolved_name) {
        return Err(anyhow!(
            "Could not identify card-name field. Fields: {:?}",
            schema.fields
        ));
    }
    if !schema.fields.contains(&resolved_count) {
        return Err(anyhow!(
            "Could not identify owned-count field. Fields: {:?}",
            schema.fields
        ));
    }

    let headers: Vec<String> = reader.headers()?.iter().map(str::to_string).collect();
    let name_idx = headers
        .iter()
        .position(|field| field == &resolved_name)
        .unwrap();
    let count_idx = headers
        .iter()
        .position(|field| field == &resolved_count)
        .unwrap();

    let mut collection = CollectionIndex {
        source_path: source_label,
        name_field: Some(resolved_name),
        count_field: Some(resolved_count),
        ..CollectionIndex::default()
    };

    for (row_idx, record) in reader.records().enumerate() {
        let line_number = row_idx + 2;
        let record = record?;
        let raw_name = record.get(name_idx).unwrap_or("").trim();
        if raw_name.is_empty() {
            collection.warnings.push(format!(
                "Line {line_number}: skipped row with empty card name"
            ));
            continue;
        }
        let raw_count = record.get(count_idx).unwrap_or("0").trim();
        let mut count = match raw_count.parse::<f64>() {
            Ok(value) => value as i64,
            Err(_) => {
                collection.warnings.push(format!(
                    "Line {line_number}: nonnumeric count {raw_count:?}; treated as 0"
                ));
                0
            }
        };
        if count < 0 {
            collection.warnings.push(format!(
                "Line {line_number}: negative count for {raw_name}; treated as 0"
            ));
            count = 0;
        }
        let key = normalize_name(raw_name);
        *collection.counts.entry(key.clone()).or_insert(0) += count as u32;
        collection
            .display_names
            .entry(key)
            .or_insert_with(|| raw_name.to_string());
    }

    Ok(collection)
}
