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

pub fn read_table<T: for<'de> Deserialize<'de>>(file_path: &str, table_name: &str) -> Result<Vec<T>> {
    let output = Command::new("mdb-export")
        .arg(file_path)
        .arg(table_name)
        .output()
        .context("Failed to execute mdb-export")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("mdb-export failed for table {}: {}", table_name, String::from_utf8_lossy(&output.stderr)));
    }

    let mut reader = csv::Reader::from_reader(output.stdout.as_slice());
    let mut results = Vec::new();
    for result in reader.deserialize() {
        let record: T = result.with_context(|| format!("Failed to deserialize CSV record in table {}", table_name))?;
        results.push(record);
    }
    Ok(results)
}
