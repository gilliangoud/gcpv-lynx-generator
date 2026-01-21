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
use odbc_api::{Environment, ConnectionOptions, Cursor};
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
    
    match conn.execute(&query, ())? {
        Some(mut cursor) => {
            let mut results = Vec::new();
            
            // Get column names
            let num_cols = cursor.num_result_cols()?;
            let mut col_names = Vec::new();
            for i in 1..=num_cols {
                let mut name = String::new();
                cursor.col_name(i as u16, &mut name)?;
                col_names.push(name);
            }

            // Iterate rows
            // For simplicity with generic T, we'll fetch as text and let serde_json try to handle it?
            // No, as discussed, if T expects int, string "123" might fail in serde_json depending on config.
            // But usually JSON parsers don't auto-convert string to int.
            // However, our structs use Option<i32> etc.
            // Let's try to fetch correct types if possible, or easiest: fetch everything as text and use a custom deserializer?
            // Actually, `csv` crate deserializes everything from text.
            // Maybe we should just construct a CSV string from ODBC and feed it to the CSV reader?
            // That matches the previous logic perfectly and avoids type mapping issues!
            // Yes, generating dynamic CSV in memory is safer for type compatibility with existing structs.
            
            let batch_size = 500;
            let mut buffers = TextRowSet::new(batch_size, &cursor)?;
            let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;

            // We'll write to a buffer in CSV format
            let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
            // Write headers
            wtr.write_record(&col_names)?;

            while let Some(batch) = row_set_cursor.fetch()? {
                for i in 0..batch.num_rows() {
                    let mut record = Vec::new();
                    for col_idx in 0..num_cols {
                        let val = batch.at(col_idx as usize, i).unwrap_or(&[]);
                        record.push(String::from_utf8_lossy(val).to_string());
                    }
                    wtr.write_record(&record)?;
                }
            }

            let data = wtr.into_inner()?;
            // println!("ODBC CSV: {}", String::from_utf8_lossy(&data));

            let mut reader = csv::Reader::from_reader(data.as_slice());
            for result in reader.deserialize() {
                let record: T = result.with_context(|| format!("Failed to deserialize generated CSV record from ODBC for {}", table_name))?;
                results.push(record);
            }
            
            Ok(results)
        },
        None => Ok(Vec::new()), // No results?
    }
}

#[cfg(not(target_os = "windows"))]
fn read_table_fallback<T: for<'de> Deserialize<'de>>(_file_path: &str, _table_name: &str) -> Result<Vec<T>> {
    Err(anyhow::anyhow!("ODBC fallback not supported on this OS"))
}
