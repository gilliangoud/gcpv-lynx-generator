use crate::logic::{Race, Lane, ProgramItem, CompetitorInCompetition, Competitor};
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use serde::Serialize;
use crate::logic::letter_to_number;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonLane {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helmet_id: Option<i32>,
    pub name: String,
    pub affiliation_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub competitor_id: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonRace {
    pub name: String,
    pub title: String,
    pub event: String,
    pub heat: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<i32>,
    pub track: i32,
    pub lanes: Vec<JsonLane>,
}

pub fn generate_race_json(
    races: &[Race], 
    programs: &[ProgramItem], 
    lanes: &[Lane], 
    competitors: &[Competitor], 
    competitors_in_comp: &[CompetitorInCompetition]
) -> Result<Vec<JsonRace>> {
    let mut json_races = Vec::new();

    // Maps for faster lookup
    let program_map: std::collections::HashMap<i32, &ProgramItem> = programs.iter().map(|p| (p.id, p)).collect();
    let comp_in_comp_map: std::collections::HashMap<i32, &CompetitorInCompetition> = competitors_in_comp.iter().map(|c| (c.id, c)).collect();
    let comp_map: std::collections::HashMap<String, &Competitor> = competitors.iter().filter_map(|c| c.id.as_ref().map(|id| (id.clone(), c))).collect();

    for race in races {
        let program_item = program_map.get(&race.program_item_id);
        
        // Race Name parsing (e.g. 101A)
        let event_name = race.name.chars().filter(|c| !c.is_alphabetic()).collect::<String>();
        let heat_letters: String = race.name.chars().filter(|c| c.is_alphabetic()).collect();
        let heat = letter_to_number(&heat_letters);
        
        let length_val = program_item.and_then(|p| p.length).unwrap_or(0);
        let track_val = program_item.map(|p| p.track).unwrap_or(100);
        let group_str = program_item.and_then(|p| p.group.as_ref()).map(|s| s.as_str()).unwrap_or("");
        
        // Filter lanes for this race
        let mut race_lanes: Vec<&Lane> = lanes.iter().filter(|l| l.race_id == race.id).collect();
        // Sort by start position
        race_lanes.sort_by_key(|l| l.start_position.unwrap_or(999));

        let mut json_lanes_vec = Vec::new();

        for lane in race_lanes {
            let skater_comp_id = lane.skater_in_competition_id;
            let competitor_in_comp = comp_in_comp_map.get(&skater_comp_id);
            
            let comp_info = competitor_in_comp.and_then(|cic| {
                cic.competitor_id.as_ref().and_then(|cid| comp_map.get(cid))
            });

            // JSON Lane
            let aff_url = format!("C:/Users/Goud/Desktop/SpeedSkating/logos/provinces/{}.png", 
                competitor_in_comp.and_then(|c| c.affiliation.as_ref()).map(|s| s.as_str()).unwrap_or("")
            );
            
            let first_name = comp_info.and_then(|c| c.first_name.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let last_name = comp_info.and_then(|c| c.last_name.as_ref()).map(|s| s.as_str()).unwrap_or("");

            json_lanes_vec.push(JsonLane {
                start_position: lane.start_position,
                helmet_id: competitor_in_comp.and_then(|c| c.helmet_id),
                name: format!("{} {}", first_name, last_name).trim().to_string(),
                affiliation_url: aff_url,
                last_name: comp_info.and_then(|c| c.last_name.clone()),
                first_name: comp_info.and_then(|c| c.first_name.clone()),
                affiliation: competitor_in_comp.and_then(|c| c.affiliation.clone()),
                competitor_id: comp_info.and_then(|c| c.id.clone()),
            });
        }

        json_races.push(JsonRace {
            name: race.name.clone(),
            title: format!("{} - {}m  {} ({}m)", race.name, length_val, group_str, track_val),
            event: event_name,
            heat,
            group: program_item.and_then(|p| p.group.clone()),
            length: program_item.and_then(|p| p.length),
            track: track_val,
            lanes: json_lanes_vec,
        });
    }

    Ok(json_races)
}

pub fn write_lynx_evt(
    file_path: &str, 
    json_path: &str,
    races: &[Race], 
    programs: &[ProgramItem], 
    lanes: &[Lane], 
    competitors: &[Competitor], 
    competitors_in_comp: &[CompetitorInCompetition]
) -> Result<()> {
    
    // Prepare writers
    let mut evt_file = File::create(file_path)?;
    
    // Generate JSON structure locally for EVT iteration (redundant but simpler for now or we just rewrite EVT logic too)
    // Actually, EVT logic is separate. We can keep EVT writing logic here, but JSON logic is extracted.
    // The previous implementation mixed them.
    // Let's re-implement EVT writing loop separately because the JSON generation loop didn't write EVT.
    
    // OR we just iterate again for EVT.
    
    // Maps for faster lookup
    let program_map: std::collections::HashMap<i32, &ProgramItem> = programs.iter().map(|p| (p.id, p)).collect();
    let comp_in_comp_map: std::collections::HashMap<i32, &CompetitorInCompetition> = competitors_in_comp.iter().map(|c| (c.id, c)).collect();
    let comp_map: std::collections::HashMap<String, &Competitor> = competitors.iter().filter_map(|c| c.id.as_ref().map(|id| (id.clone(), c))).collect();

    for race in races {
        let program_item = program_map.get(&race.program_item_id);
        
        let length_val = program_item.and_then(|p| p.length).unwrap_or(0);
        let track_val = program_item.map(|p| p.track).unwrap_or(100);
        let group_str = program_item.and_then(|p| p.group.as_ref()).map(|s| s.as_str()).unwrap_or("");
        
        writeln!(evt_file, "{},1,01,{} {} {}m {}m", 
            race.name, 
            race.name, 
            group_str, 
            length_val, 
            track_val
        )?;

        // Filter lanes for this race
        let mut race_lanes: Vec<&Lane> = lanes.iter().filter(|l| l.race_id == race.id).collect();
        // Sort by start position
        race_lanes.sort_by_key(|l| l.start_position.unwrap_or(999));

        for lane in race_lanes {
            let skater_comp_id = lane.skater_in_competition_id;
            let competitor_in_comp = comp_in_comp_map.get(&skater_comp_id);
            
            let comp_info = competitor_in_comp.and_then(|cic| {
                cic.competitor_id.as_ref().and_then(|cid| comp_map.get(cid))
            });

            let helmet = competitor_in_comp.and_then(|c| c.helmet_id).unwrap_or(0);
            let start_pos = lane.start_position.unwrap_or(0);
            let last_name = comp_info.and_then(|c| c.last_name.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let first_name = comp_info.and_then(|c| c.first_name.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let affiliation = competitor_in_comp.and_then(|c| c.affiliation.as_ref()).map(|s| s.as_str()).unwrap_or("");
            let comp_id_str = comp_info.and_then(|c| c.id.as_ref()).map(|s| s.as_str()).unwrap_or("");

            writeln!(evt_file, ",{},{},{},{},{},,{}", 
                helmet, start_pos, last_name, first_name, affiliation, comp_id_str
            )?;
        }
    }

    // Write JSON using the shared function
    let json_races = generate_race_json(races, programs, lanes, competitors, competitors_in_comp)?;
    let json_file = File::create(json_path)?;
    serde_json::to_writer_pretty(json_file, &json_races)?;

    Ok(())
}
