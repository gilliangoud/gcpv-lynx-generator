use crate::logic::*;
use crate::writer::write_lynx_evt;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn check_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn execute_cycle(
    sync_location: &str, 
    event_file_path: &str, 
    json_path: &str, 
    env_competition_id: Option<i32>
) -> Result<()> {
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

    // Clean up old files
    let _ = fs::remove_file(event_file_path);
    let _ = fs::remove_file(json_path);

    // Fetch data
    let mut races = get_races(sync_location, competition_id).context("Failed to get races")?;
    // Sort races
    races.sort_by(race_compare);
    println!("Got {} races", races.len());

    let programs = get_programs(sync_location, competition_id).context("Failed to get programs")?;
    let lanes = get_lanes(sync_location, competition_id).context("Failed to get lanes")?;
    let competitors = get_competitors(sync_location).context("Failed to get competitors")?;
    let competitors_in_comp = get_competitors_in_competition(sync_location, competition_id).context("Failed to get competitors in comp")?;

    // Write outputs
    write_lynx_evt(
        event_file_path,
        json_path,
        &races,
        &programs,
        &lanes,
        &competitors,
        &competitors_in_comp
    ).context("Failed to write output files")?;

    println!("Done lynx and json");
    Ok(())
}
