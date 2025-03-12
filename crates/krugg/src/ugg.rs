use std::{collections::HashMap, num::NonZeroUsize};

use ddragon::models::{champions::ChampionShort, items::Item, runes::RuneElement};
use lru::LruCache;
use serde::de::DeserializeOwned;
use tauri::{
    AppHandle, Manager, Runtime,
    async_runtime::{self, RwLock},
};
use tauri_plugin_http::reqwest::IntoUrl;
use ugg_types::{matchups::Matchups, overview::ChampOverview, rune::RuneExtended};

use crate::ddragon::{Client, ClientBuilder};

type UggApiVersions = HashMap<String, HashMap<String, String>>;

#[derive(Debug)]
pub struct DataApi {
    client: Client,
    overview_cache: RwLock<LruCache<String, ChampOverview>>,
    matchup_cache: RwLock<LruCache<String, Matchups>>,
}

#[derive(Debug)]
pub struct Versions {
    ddragon: String,
    ugg: String,
}

pub struct Api {
    api: DataApi,
    ugg_api_versions: UggApiVersions,

    current_version: String,
    allowed_versions: Box<[Versions]>,
    patch_version: String,
    champ_data: HashMap<String, ChampionShort>,
    items: HashMap<String, Item>,
    runes: HashMap<i64, RuneExtended<RuneElement>>,
    summoner_spells: HashMap<i64, String>,
}

impl DataApi {
    pub async fn new<R: Runtime>(
        app: &AppHandle<R>,
        version: Option<&str>,
        locale: Option<&str>,
    ) -> anyhow::Result<Self> {
        let client = ClientBuilder::new()
            .cache(app.path().app_cache_dir()?)
            .version(version)
            .locale(locale);
        let cache_size = NonZeroUsize::new(50).unwrap();

        Ok(Self {
            client: client.build().await?,
            overview_cache: RwLock::new(LruCache::new(cache_size)),
            matchup_cache: RwLock::new(LruCache::new(cache_size)),
        })
    }

    async fn get<T: DeserializeOwned, U: IntoUrl>(&self, path: U) -> anyhow::Result<T> {
        Ok(self
            .client
            .client()
            .get(path)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    pub const fn version(&self) -> &str {
        self.client.version()
    }

    pub async fn get_supported_versions(&self) -> anyhow::Result<Vec<String>> {
        self.get(self.client.base_url().join("/api/versions.json")?)
            .await
    }

    pub async fn get_champions(&self) -> anyhow::Result<HashMap<String, ChampionShort>> {
        Ok(self.client.champions().await?.data)
    }

    pub async fn get_items(&self) -> anyhow::Result<HashMap<String, Item>> {
        Ok(self.client.items().await?.data)
    }

    pub async fn get_runes(&self) -> anyhow::Result<HashMap<i64, RuneExtended<RuneElement>>> {
        let runes = self.client.runes().await?;
        let mut data = HashMap::new();

        for class in runes {
            for (slot_idx, slot) in class.slots.iter().enumerate() {
                for (i, rune) in slot.runes.iter().enumerate() {
                    data.insert(
                        rune.id,
                        RuneExtended {
                            rune: rune.clone(),
                            slot: slot_idx as u64,
                            index: i as u64,
                            siblings: slot.runes.len() as u64,
                            parent: class.name.clone(),
                            parent_id: class.id,
                        },
                    );
                }
            }
        }

        Ok(data)
    }

    pub async fn get_summoner_spells(&self) -> anyhow::Result<HashMap<i64, String>> {
        let summoners = self.client.summoner_spells().await?;
        let mut data = HashMap::new();

        for (_, spell) in summoners.data {
            data.insert(spell.key.parse::<i64>()?, spell.name);
        }

        Ok(data)
    }

    pub async fn get_ugg_api_versions(&self) -> anyhow::Result<UggApiVersions> {
        self.get("https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json").await
    }
}
