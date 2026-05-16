//! GeoPackage (SQLite) reader/writer.

use std::collections::HashMap;
use std::path::Path;

use geodukt_core::feature::{Feature, FeatureCollection, Value};
use geodukt_core::pipeline::PipelineError;
use rusqlite::Connection;

/// Read features from a GeoPackage file.
pub fn read_geopackage(
    path: &Path,
    table: Option<&str>,
) -> Result<FeatureCollection, PipelineError> {
    let conn = Connection::open(path).map_err(|e| PipelineError::Source {
        name: "geopackage".into(),
        message: format!("failed to open: {e}"),
    })?;

    // Find the first feature table if none specified
    let table_name = if let Some(t) = table {
        t.to_string()
    } else {
        conn.query_row(
            "SELECT table_name FROM gpkg_contents WHERE data_type='features' LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| PipelineError::Source {
            name: "geopackage".into(),
            message: format!("no feature table found: {e}"),
        })?
    };

    // Get column info
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info('{table_name}')"))
        .map_err(|e| PipelineError::Source {
            name: "geopackage".into(),
            message: e.to_string(),
        })?;

    let columns: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| PipelineError::Source {
            name: "geopackage".into(),
            message: e.to_string(),
        })?
        .filter_map(|r| r.ok())
        .filter(|c| c != "geom" && c != "geometry" && c != "fid")
        .collect();

    let col_list = columns.join(", ");
    let query = format!("SELECT {col_list} FROM \"{table_name}\"");

    let mut stmt = conn.prepare(&query).map_err(|e| PipelineError::Source {
        name: "geopackage".into(),
        message: e.to_string(),
    })?;

    let features: Vec<Feature> = stmt
        .query_map([], |row| {
            let mut props = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let val: String = row.get::<_, String>(i).unwrap_or_default();
                props.insert(col.clone(), Value::String(val));
            }
            Ok(Feature {
                geometry: geo::Geometry::Point(geo::Point::new(0.0, 0.0)),
                properties: props,
            })
        })
        .map_err(|e| PipelineError::Source {
            name: "geopackage".into(),
            message: e.to_string(),
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(FeatureCollection::new(features, None))
}

/// Write features to a GeoPackage file.
pub fn write_geopackage(
    path: &Path,
    fc: &FeatureCollection,
    table: &str,
) -> Result<(), PipelineError> {
    let conn = Connection::open(path).map_err(|e| PipelineError::Sink {
        name: "geopackage".into(),
        message: format!("failed to open: {e}"),
    })?;

    // Create GeoPackage metadata tables
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS gpkg_contents (
            table_name TEXT NOT NULL PRIMARY KEY,
            data_type TEXT NOT NULL,
            identifier TEXT,
            description TEXT DEFAULT '',
            last_change DATETIME DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            min_x DOUBLE, min_y DOUBLE, max_x DOUBLE, max_y DOUBLE,
            srs_id INTEGER
        );
        CREATE TABLE IF NOT EXISTS gpkg_spatial_ref_sys (
            srs_name TEXT NOT NULL,
            srs_id INTEGER NOT NULL PRIMARY KEY,
            organization TEXT NOT NULL,
            organization_coordsys_id INTEGER NOT NULL,
            definition TEXT NOT NULL
        );",
    )
    .map_err(|e| PipelineError::Sink {
        name: "geopackage".into(),
        message: e.to_string(),
    })?;

    // Collect property columns from first feature
    let columns: Vec<String> = fc
        .features
        .first()
        .map(|f| f.properties.keys().cloned().collect())
        .unwrap_or_default();

    let col_defs: String = columns
        .iter()
        .map(|c| format!("\"{c}\" TEXT"))
        .collect::<Vec<_>>()
        .join(", ");

    conn.execute(
        &format!("CREATE TABLE IF NOT EXISTS \"{table}\" (fid INTEGER PRIMARY KEY AUTOINCREMENT, {col_defs})"),
        [],
    )
    .map_err(|e| PipelineError::Sink {
        name: "geopackage".into(),
        message: e.to_string(),
    })?;

    conn.execute(
        "INSERT OR REPLACE INTO gpkg_contents (table_name, data_type) VALUES (?1, 'features')",
        [table],
    )
    .map_err(|e| PipelineError::Sink {
        name: "geopackage".into(),
        message: e.to_string(),
    })?;

    let placeholders: String = columns.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let col_names: String = columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let insert_sql = format!("INSERT INTO \"{table}\" ({col_names}) VALUES ({placeholders})");

    for feature in &fc.features {
        let values: Vec<String> = columns
            .iter()
            .map(|c| match feature.properties.get(c) {
                Some(Value::String(s)) => s.clone(),
                Some(v) => format!("{v:?}"),
                None => String::new(),
            })
            .collect();

        let params: Vec<&dyn rusqlite::types::ToSql> = values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        conn.execute(&insert_sql, params.as_slice())
            .map_err(|e| PipelineError::Sink {
                name: "geopackage".into(),
                message: e.to_string(),
            })?;
    }

    Ok(())
}
