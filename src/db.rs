use serde::Deserialize;
use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct TCompetition {
    #[serde(rename = "NoCompetition")]
    pub no_competition: Option<i32>,
    #[serde(rename = "Lieu")]
    pub lieu: Option<String>,
    #[serde(rename = "Date")]
    pub date: Option<String>,
    #[serde(rename = "NoClub")]
    pub no_club: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TPatineurs {
    #[serde(rename = "NoPatineur")]
    pub no_patineur: Option<i32>,
    #[serde(rename = "Prenom")]
    pub prenom: Option<String>,
    #[serde(rename = "Nom")]
    pub nom: Option<String>,
    #[serde(rename = "Date de naissance")]
    pub date_naissance: Option<String>,
    #[serde(rename = "Sexe")]
    pub sexe: Option<String>,
    #[serde(rename = "Division")]
    pub division: Option<String>,
    #[serde(rename = "NoCategorie")]
    pub no_categorie: Option<i32>,
    #[serde(rename = "NoClub")]
    pub no_club: Option<i32>,
    #[serde(rename = "CodePat")]
    pub code_pat: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TPatineurCompe {
    #[serde(rename = "NoPatCompe")]
    pub no_pat_compe: Option<i32>,
    #[serde(rename = "NoCompetition")]
    pub no_competition: Option<i32>,
    #[serde(rename = "NoPatineur")]
    pub no_patineur: Option<i32>,
    #[serde(rename = "NoCategorie")]
    pub no_categorie: Option<i32>,
    #[serde(rename = "NoClub")]
    pub no_club: Option<i32>,
    #[serde(rename = "Rang")]
    pub rang: Option<i32>,
    #[serde(rename = "Retirer")]
    pub retirer: Option<i32>,
    #[serde(rename = "Groupe")]
    pub groupe: Option<String>,
    #[serde(rename = "NoCasque")]
    pub no_casque: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TClubs {
    #[serde(rename = "NoClub")]
    pub no_club: Option<i32>,
    #[serde(rename = "Nom du Club")]
    pub nom_du_club: Option<String>,
    #[serde(rename = "Commentaire")]
    pub commentaire: Option<String>,
    #[serde(rename = "NoRegion")]
    pub no_region: Option<i32>,
    #[serde(rename = "Abreviation")]
    pub abreviation: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TDistancesStandards {
    #[serde(rename = "NoDistance")]
    pub no_distance: Option<i32>,
    #[serde(rename = "Distance")]
    pub distance: Option<String>,
    #[serde(rename = "LongueurEpreuve")]
    pub longueur_epreuve: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TProgCourses {
    #[serde(rename = "CleDistancesCompe")]
    pub cle_distances_compe: Option<i32>,
    #[serde(rename = "NoCompetition")]
    pub no_competition: Option<i32>,
    #[serde(rename = "NoDistance")]
    pub no_distance: Option<i32>,
    #[serde(rename = "Distance")]
    pub distance: Option<String>,
    #[serde(rename = "NoVague")]
    pub no_vague: Option<String>,
    #[serde(rename = "Groupe")]
    pub groupe: Option<String>,
    #[serde(rename = "OrdreSequence")]
    pub ordre_sequence: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TVagues {
    #[serde(rename = "CleTVagues")]
    pub cle_tvagues: Option<i32>,
    #[serde(rename = "NoVague")]
    pub no_vague: Option<String>,
    #[serde(rename = "CleDistancesCompe")]
    pub cle_distances_compe: Option<i32>,
    #[serde(rename = "Qual_ou_Fin")]
    pub qual_ou_fin: Option<String>,
    #[serde(rename = "Seq")]
    pub seq: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TPatVagues {
    #[serde(rename = "CleTVagues")]
    pub cle_tvagues: Option<i32>,
    #[serde(rename = "NoPatCompe")]
    pub no_pat_compe: Option<i32>,
    #[serde(rename = "Temps")]
    pub temps: Option<String>,
    #[serde(rename = "Rang")]
    pub rang: Option<i32>,
    #[serde(rename = "NoCasque")]
    pub no_casque: Option<i32>,
    #[serde(rename = "CleTPatVagues")]
    pub cle_tpat_vagues: Option<i32>,
}

fn get_mdb_export_command() -> String {
    if cfg!(target_os = "windows") {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(dir) = exe_path.parent() {
                let local_mdb = dir.join("mdb-export.exe");
                if local_mdb.exists() {
                    return local_mdb.to_string_lossy().to_string();
                }
            }
        }
    }
    "mdb-export".to_string()
}

pub fn read_table<T: for<'de> Deserialize<'de>>(file_path: &str, table_name: &str) -> Result<Vec<T>> {
    // Try mdb-export first
    let output = Command::new(get_mdb_export_command())
        .arg(file_path)
        .arg(table_name)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            // Check raw output for debugging
            if std::env::var("DEBUG_CSV").is_ok() {
                println!("CSV Output for {}: {}", table_name, String::from_utf8_lossy(&output.stdout));
            }

            let mut reader = csv::Reader::from_reader(output.stdout.as_slice());
            let mut results = Vec::new();
            for result in reader.deserialize() {
                let record: T = result.with_context(|| format!("Failed to deserialize CSV record in table {}", table_name))?;
                results.push(record);
            }
            Ok(results)
        }
        err => {
            // If mdb-export failed, check if we are on Windows and try fallback
            if cfg!(target_os = "windows") {
                println!("mdb-export not found or failed, trying ODBC fallback for {}", table_name);
                return read_table_fallback(file_path, table_name)
                    .context("Both mdb-export and ODBC fallback failed");
            }
            
            // Otherwise return the original error if it was an execution error, or the failure output
            match err {
                Ok(output) => {
                     Err(anyhow::anyhow!("mdb-export failed for table {}: {}", table_name, String::from_utf8_lossy(&output.stderr)))
                }
                Err(e) => Err(anyhow::anyhow!("Failed to execute mdb-export: {}", e)),
            }
        }
    }
}

#[cfg(target_os = "windows")]
use odbc_api::{Environment, ConnectionOptions, Cursor, ResultSetMetadata};
#[cfg(target_os = "windows")]
use odbc_api::buffers::TextRowSet;

#[cfg(target_os = "windows")]
fn read_table_fallback<T: for<'de> Deserialize<'de>>(file_path: &str, table_name: &str) -> Result<Vec<T>> {
    let env = Environment::new()?;
    
    // Construct connection string for Access
    // Driver name might vary "Microsoft Access Driver (*.mdb, *.accdb)" is standard for ACE
    let conn_string = format!("Driver={{Microsoft Access Driver (*.mdb, *.accdb)}};Dbq={};", file_path);
    
    let conn = env.connect_with_connection_string(&conn_string, ConnectionOptions::default())
        .context("Failed to connect to Access DB via ODBC. Ensure Microsoft Access Database Engine 2016 Redistributable is installed.")?;

    let query = format!("SELECT * FROM [{}]", table_name);
    
    let maybe_cursor = conn.execute(&query, ())?;
    
    if let Some(mut cursor) = maybe_cursor {
        let mut results = Vec::new();
        
        // Get column names
        let num_cols = cursor.num_result_cols()?;
        let mut col_names = Vec::new();
        for i in 1..=num_cols {
             let name = cursor.col_name(i as u16)?;
             col_names.push(name);
        }

        // Processing block to ensure borrows are dropped
        let csv_data = {
            let batch_size = 500;
            // Use for_cursor to automatically create text buffers for all columns
            let mut buffers = TextRowSet::for_cursor(batch_size, &mut cursor, Some(4096))?;
            let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

            let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
            wtr.write_record(&col_names)?;

            while let Some(batch) = row_set_cursor.fetch()? {
                for i in 0..batch.num_rows() {
                    let mut record = Vec::new();
                    for col_idx in 0..num_cols {
                        let val = batch.at(col_idx as usize, i).unwrap_or(&[]);
                        let val_str = String::from_utf8_lossy(val).to_string();
                        // Access ODBC driver sometimes returns integers as "123.0"
                        // Since we deal with loosely typed CSV, we can strip ".0" suffix if present
                        if val_str.ends_with(".0") {
                            record.push(val_str.trim_end_matches(".0").to_string());
                        } else {
                            record.push(val_str);
                        }
                    }
                    wtr.write_record(&record)?;
                }
            }
            wtr.into_inner()?
        };

        // Deserialize with diagnostics
        let mut reader = csv::Reader::from_reader(csv_data.as_slice());
        let headers = reader.headers().cloned().unwrap_or_default();
        
        let mut records = reader.records();
        while let Some(r) = records.next() {
            let record = r.context("Failed to read generated CSV record")?;
            let t_result: Result<T, _> = record.deserialize(Some(&headers));
            match t_result {
                Ok(rec) => results.push(rec),
                Err(e) => {
                    eprintln!("Failed to deserialize record in table {}", table_name);
                    eprintln!("Headers: {:?}", headers);
                    eprintln!("Record: {:?}", record);
                    eprintln!("Error: {}", e);
                    return Err(anyhow::anyhow!("ODBC Deserialization Error in {}: {}", table_name, e));
                }
            }
        }
        
        Ok(results)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(not(target_os = "windows"))]
fn read_table_fallback<T: for<'de> Deserialize<'de>>(_file_path: &str, _table_name: &str) -> Result<Vec<T>> {
   Err(anyhow::anyhow!("ODBC fallback not supported on this OS"))
}
