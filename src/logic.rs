use crate::db::*;
use anyhow::{Context, Result};
use std::collections::HashMap;

// Competition Logic
pub fn get_competition_id(file_path: &str) -> Result<i32> {
    let competitions: Vec<TCompetition> = read_table(file_path, "TCompetition")?;
    if competitions.is_empty() {
        anyhow::bail!("No competition found in mdb");
    }
    if competitions.len() > 1 {
        anyhow::bail!("Multiple competitions found in mdb");
    }
    // Unwrap safely
    competitions[0].no_competition.context("Competition ID is missing or invalid")
}

// Competitor Logic
#[derive(Debug, Clone)]
pub struct Competitor {
    pub id: Option<String>,
    pub no_patineur: i32,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_date: Option<String>,
    pub sex: Option<String>,
    pub division: Option<String>,
    pub category_id: Option<i32>,
    pub club_id: Option<i32>,
}

pub fn get_competitors(file_path: &str) -> Result<Vec<Competitor>> {
    let raw_competitors: Vec<TPatineurs> = read_table(file_path, "TPatineurs")?;
    Ok(raw_competitors.into_iter()
        .filter_map(|c| {
            Some(Competitor {
                id: c.code_pat,
                no_patineur: c.no_patineur?, // critical
                first_name: c.prenom,
                last_name: c.nom,
                birth_date: c.date_naissance,
                sex: c.sexe,
                division: c.division,
                category_id: c.no_categorie,
                club_id: c.no_club,
            })
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct CompetitorInCompetition {
    pub id: i32,
    pub competitor_id: Option<String>,
    pub club_id: Option<i32>,
    pub affiliation: Option<String>,
    pub club_name: Option<String>,
    pub rank: Option<i32>,
    pub removed: Option<bool>,
    pub group: Option<String>,
    pub helmet_id: Option<i32>,
}

pub fn get_competitors_in_competition(file_path: &str, competition_id: i32) -> Result<Vec<CompetitorInCompetition>> {
    let competitors = get_competitors(file_path)?;
    let clubs: Vec<TClubs> = read_table(file_path, "TClubs")?;
    let raw_comps: Vec<TPatineurCompe> = read_table(file_path, "TPatineur_compe")?;

    let comp_map: HashMap<i32, Competitor> = competitors.into_iter().map(|c| (c.no_patineur, c)).collect();
    let club_map: HashMap<i32, TClubs> = clubs.into_iter().filter_map(|c| c.no_club.map(|id| (id, c))).collect();

    Ok(raw_comps.into_iter()
        .filter(|c| c.no_competition == Some(competition_id))
        .filter_map(|c| {
            let id = c.no_pat_compe?;
            let patineur_id = c.no_patineur?;
            let competitor = comp_map.get(&patineur_id);
            let club_id = c.no_club;
            let club = club_id.and_then(|id| club_map.get(&id));
            
            Some(CompetitorInCompetition {
                id,
                competitor_id: competitor.and_then(|comp| comp.id.clone()),
                club_id, // stored as Option
                affiliation: club.and_then(|cl| cl.abreviation.clone().or(cl.nom_du_club.clone())),
                club_name: club.and_then(|cl| cl.nom_du_club.clone()),
                rank: c.rang,
                removed: c.retirer.map(|v| v != 0),
                group: c.groupe,
                helmet_id: c.no_casque,
            })
        })
        .collect())
}

// Program and Race Logic
#[derive(Debug, Clone)]
pub struct Distance {
    pub id: i32,
    pub name: Option<String>,
    pub length: Option<i32>,
    pub track: i32,
}

pub fn get_distances(file_path: &str) -> Result<Vec<Distance>> {
    let raw: Vec<TDistancesStandards> = read_table(file_path, "TDistances_Standards")?;
    Ok(raw.into_iter().filter_map(|row| {
        let track = if row.distance.as_ref().map(|d| d.contains("(111)")).unwrap_or(false) { 111 } else { 100 };
        Some(Distance {
            id: row.no_distance?,
            name: row.distance,
            length: row.longueur_epreuve,
            track,
        })
    }).collect())
}

#[derive(Debug, Clone)]
pub struct ProgramItem {
    pub id: i32,
    pub competition_id: i32,
    pub distance_id: i32,
    pub distance: Option<String>,
    pub group: Option<String>,
    pub length: Option<i32>,
    pub track: i32,
}

pub fn get_programs(file_path: &str, competition_id: i32) -> Result<Vec<ProgramItem>> {
    let distances = get_distances(file_path)?;
    let dist_map: HashMap<i32, Distance> = distances.into_iter().map(|d| (d.id, d)).collect();
    
    let raw: Vec<TProgCourses> = read_table(file_path, "TProg_Courses")?;
    
    Ok(raw.into_iter()
        .filter(|row| row.no_competition == Some(competition_id))
        .filter_map(|row| {
            let dist_id = row.no_distance?;
            let distance = dist_map.get(&dist_id);
            Some(ProgramItem {
                id: row.cle_distances_compe?,
                competition_id: row.no_competition?,
                distance_id: dist_id,
                distance: row.distance,
                group: row.groupe,
                length: distance.and_then(|d| d.length),
                track: distance.map(|d| d.track).unwrap_or(100),
            })
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct Race {
    pub id: i32,
    pub name: String,
    pub distance: Option<i32>,
    pub track: i32,
    pub program_item_id: i32,
    pub sequence: Option<i32>,
    pub round: Option<String>,
}

pub fn get_races(file_path: &str, competition_id: i32) -> Result<Vec<Race>> {
    let programs = get_programs(file_path, competition_id)?;
    let prog_map: HashMap<i32, ProgramItem> = programs.iter().map(|p| (p.id, p.clone())).collect();
    
    let raw: Vec<TVagues> = read_table(file_path, "TVagues")?;
    
    Ok(raw.into_iter()
        .filter_map(|row| {
            let cle_dist = row.cle_distances_compe?;
            let prog = prog_map.get(&cle_dist)?;
            if prog.competition_id != competition_id {
                return None;
            }
            Some(Race {
                id: row.cle_tvagues?,
                name: row.no_vague.unwrap_or_default(),
                distance: prog.length,
                track: prog.track,
                program_item_id: prog.id,
                sequence: row.seq,
                round: row.qual_ou_fin,
            })
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct Lane {
    pub id: i32,
    pub race_id: i32,
    pub skater_in_competition_id: i32,
    pub skater_upid: Option<String>,
    pub time: Option<String>,
    pub position: Option<i32>,
    pub start_position: Option<i32>,
}

pub fn get_lanes(file_path: &str, competition_id: i32) -> Result<Vec<Lane>> {
    let races = get_races(file_path, competition_id)?;
    let race_ids: Vec<i32> = races.iter().map(|r| r.id).collect();
    
    let competitors = get_competitors_in_competition(file_path, competition_id)?;
    let comp_map: HashMap<i32, CompetitorInCompetition> = competitors.into_iter().map(|c| (c.id, c)).collect();
    
    let raw: Vec<TPatVagues> = read_table(file_path, "TPatVagues")?;
    
    Ok(raw.into_iter()
        .filter_map(|lane| {
            let race_id = lane.cle_tvagues?;
            if !race_ids.contains(&race_id) {
                return None;
            }
            let no_pat_compe = lane.no_pat_compe?;
            let comp = comp_map.get(&no_pat_compe);

            Some(Lane {
                id: lane.cle_tpat_vagues?,
                race_id,
                skater_in_competition_id: no_pat_compe,
                skater_upid: comp.and_then(|c| c.competitor_id.clone()),
                time: lane.temps,
                position: lane.rang,
                start_position: lane.no_casque,
            })
        })
        .collect())
}

// Helpers
pub fn letter_to_number(letter: &str) -> i32 {
    let letter = letter.to_lowercase();
    if let Some(c) = letter.chars().next() {
        (c as i32) - 97 + 1
    } else {
        0
    }
}

pub fn race_compare(a: &Race, b: &Race) -> std::cmp::Ordering {
    // Assuming simple comparison for now, or recreating the JS logic
    // JS: aNum - bNum or letter compare
    let a_num_str: String = a.name.chars().take_while(|c| c.is_numeric()).collect();
    let b_num_str: String = b.name.chars().take_while(|c| c.is_numeric()).collect();
    
    let a_num = a_num_str.parse::<i32>().unwrap_or(0);
    let b_num = b_num_str.parse::<i32>().unwrap_or(0);
    
    if a_num != b_num {
        return a_num.cmp(&b_num);
    }
    
    // Fallback sort
    a.name.cmp(&b.name)
}
