use std::collections::HashMap;

use ddragon::models::champions::ChampionShort;
use serde::Serialize;
use ugg_types::{mappings, matchups::MatchupData, overview::OverviewData};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum KruggMessage {
    Champions(HashMap<String, ChampionShort>),
    Overview {
        overview: Box<OverviewData>,
        role: mappings::Role,
    },
    Matchups {
        matchups: Box<MatchupData>,
        role: mappings::Role,
    },
}
