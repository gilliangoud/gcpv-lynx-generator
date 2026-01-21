use crate::logic::*;
use crate::writer::write_lynx_evt;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn check_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub struct RaceData {
    pub races: Vec<Race>,
    pub programs: Vec<ProgramItem>,
    pub lanes: Vec<Lane>,
    pub competitors: Vec<Competitor>,
    pub competitors_in_comp: Vec<CompetitorInCompetition>,
}

pub fn fetch_race_data(
    sync_location: &str, 
    env_competition_id: Option<i32>
) -> Result<RaceData> {
    if !check_file_exists(sync_location) {
        anyhow::bail!("File {} not found", sync_location);
    }

    // Determine competition ID
    let competition_id = if let Some(cid) = env_competition_id {
        cid
    } else {
        get_competition_id(sync_location).context("Failed to get competition ID")?
    };

    println!("Competition ID: {}", competition_id);

    // Fetch data
    let mut races = get_races(sync_location, competition_id).context("Failed to get races")?;
    // Sort races
    races.sort_by(race_compare);
    println!("Got {} races", races.len());

    let programs = get_programs(sync_location, competition_id).context("Failed to get programs")?;
    let lanes = get_lanes(sync_location, competition_id).context("Failed to get lanes")?;
    let competitors = get_competitors(sync_location).context("Failed to get competitors")?;
    let competitors_in_comp = get_competitors_in_competition(sync_location, competition_id).context("Failed to get competitors in comp")?;

    Ok(RaceData {
        races,
        programs,
        lanes,
        competitors,
        competitors_in_comp,
    })
}

pub fn execute_cycle(
    sync_location: &str, 
    event_file_path: &str, 
    json_path: &str, 
    env_competition_id: Option<i32>
) -> Result<()> {
    let race_data = fetch_race_data(sync_location, env_competition_id)?;

    // Clean up old files
    let _ = fs::remove_file(event_file_path);
    let _ = fs::remove_file(json_path);

    // Write outputs
    write_lynx_evt(
        event_file_path,
        json_path,
        &race_data.races,
        &race_data.programs,
        &race_data.lanes,
        &race_data.competitors,
        &race_data.competitors_in_comp
    ).context("Failed to write output files")?;

    println!("Done lynx and json");
    Ok(())
}
