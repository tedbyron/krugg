use std::collections::HashMap;

use ddragon::models::{Champion, champions::ChampionShort};
use serde::Serialize;
// use ugg_types::{mappings, matchups::MatchupData, overview::Overview};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum KruggMessage {
    Champions(HashMap<String, ChampionShort>),
    Champion(Box<Champion>),
    // Overview {
    //     overview: Box<Overview>,
    //     role: mappings::Role,
    // },
    // Matchups {
    //     matchups: Box<MatchupData>,
    //     role: mappings::Role,
    // },
}
