use std::{collections::HashMap, num::NonZeroUsize};

use ddragon::models::{champions::ChampionShort, items::Item, runes::RuneElement};
use lru::LruCache;
use tauri::{
    AppHandle, Manager, Runtime,
    async_runtime::{self, RwLock},
};
use tauri_plugin_http::reqwest::Client;
use ugg_types::{matchups::Matchups, overview::ChampOverview, rune::RuneExtended};

type ApiVersions = HashMap<String, HashMap<String, String>>;

#[derive(Debug)]
struct DataApi {
    client: Client,
    overview_cache: RwLock<LruCache<String, ChampOverview>>,
    matchup_cache: RwLock<LruCache<String, Matchups>>,
}

#[derive(Debug)]
struct SupportedVersion {
    ddragon: String,
    ugg: String,
}

struct Api {
    api: DataApi,
    ugg_api_version: ApiVersions,

    current_version: String,
    allowed_versions: Box<[SupportedVersion]>,
    patch_version: String,
    champ_data: HashMap<String, ChampionShort>,
    items: HashMap<String, Item>,
    runes: HashMap<i64, RuneExtended<RuneElement>>,
    summoner_spells: HashMap<i64, String>,
}

impl DataApi {
    async fn new<R: Runtime, V: AsRef<str>>(
        app: &AppHandle<R>,
        version: Option<V>,
    ) -> anyhow::Result<Self> {
        if let Some(v) = version {
            client = client.version(v.as_ref());
        }
        let cache_size = NonZeroUsize::new(50).unwrap();

        Ok(Self {
            client: client.build().await?,
            overview_cache: RwLock::new(LruCache::new(cache_size)),
            matchup_cache: RwLock::new(LruCache::new(cache_size)),
        })
    }
}
