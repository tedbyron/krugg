#![allow(unused)]

use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use ddragon::models::{
    Challenges, Champion,
    challenges::Challenge,
    champions::ChampionShort,
    items::Item,
    maps::Map,
    mission_assets::MissionAsset,
    profile_icons::ProfileIcon,
    runes::RuneElement,
    shared::{BasicDatum, HasImage},
    tft::{self, queues::Queue, regalia::RegaliaData, tactitians::Tactician},
};
use image::DynamicImage;
use lru::LruCache;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager, Runtime, async_runtime::Mutex};
use tauri_plugin_http::reqwest::IntoUrl;
use ugg_types::{
    mappings,
    matchups::{MatchupData, WrappedMatchupData},
    overview::{Overview, WrappedOverviewData},
};

use crate::{
    ddragon::{Client as DdragonClient, ClientBuilder as DdragonClientBuilder},
    error::UggError,
};

type CacheValue<T> = HashMap<mappings::Region, HashMap<mappings::Rank, HashMap<mappings::Role, T>>>;
type Cache<T> = Arc<Mutex<LruCache<String, CacheValue<T>>>>;
type OverviewCache = Cache<WrappedOverviewData>;
type MatchupCache = Cache<WrappedMatchupData>;
type UggApiVersions = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Clone)]
pub struct DdragonClientWrapper {
    ddragon: DdragonClient,
    overview_cache: OverviewCache,
    matchup_cache: MatchupCache,
}

#[derive(Debug, Clone)]
pub struct Versions {
    ddragon: String,
    ugg: String,
}

#[derive(Debug)]
pub struct ClientBuilder<'a> {
    version: Option<&'a str>,
    locale: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct RuneElementWrapper {
    pub slot: u64,
    pub index: u64,
    pub siblings: u64,
    pub parent: String,
    pub parent_id: i64,
    pub rune: RuneElement,
}

#[derive(Debug, Clone)]
pub struct Client {
    ddragon: DdragonClientWrapper,
    ugg_api_versions: UggApiVersions,

    version: String,
    supported_versions: Box<[Versions]>,
    patch_version: String,
    champions: HashMap<String, ChampionShort>,
    items: HashMap<String, Item>,
    runes: HashMap<i64, RuneElementWrapper>,
    summoner_spells: HashMap<i64, String>,
}

macro_rules! forward_ddragon_calls {
    ( $($name:ident : $t:ty ),* $(,)? ) => {
        $(
            pub async fn $name(&self) -> crate::Result<$t> {
                Ok(self.ddragon.$name().await?.data)
            }
        )*
    };
}

impl DdragonClientWrapper {
    pub async fn new<R: Runtime>(
        app: &AppHandle<R>,
        version: Option<&str>,
        locale: Option<&str>,
    ) -> crate::Result<Self> {
        let mut client = DdragonClientBuilder::new().cache(app.path().app_cache_dir()?);
        if let Some(v) = version {
            client = client.version(v);
        }
        if let Some(l) = locale {
            client = client.locale(l);
        }
        let cache_size = NonZeroUsize::new(50).unwrap();

        Ok(Self {
            ddragon: client.build().await?,
            overview_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            matchup_cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
        })
    }

    pub fn client(&self) -> ClientWithMiddleware {
        self.ddragon.client()
    }

    pub const fn version(&self) -> &str {
        self.ddragon.version()
    }

    async fn get<T: DeserializeOwned, U: IntoUrl>(&self, url: U) -> crate::Result<T> {
        Ok(self
            .ddragon
            .client()
            .get(url)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    pub async fn get_supported_versions(&self) -> crate::Result<Box<[String]>> {
        self.get(self.ddragon.base_url().join("/api/versions.json")?)
            .await
    }

    forward_ddragon_calls!(
        get_champions: HashMap<String, ChampionShort>,
        get_champions_full: HashMap<String, Champion>,
        get_items: HashMap<String, Item>,
        get_maps: HashMap<String, Map>,
        get_mission_assets: HashMap<String, MissionAsset>,
        get_profile_icons: HashMap<String, ProfileIcon>,
        get_translations: HashMap<String, String>,
        get_tft_arenas: HashMap<String, BasicDatum>,
        get_tft_augments: HashMap<String, BasicDatum>,
        get_tft_champions: HashMap<String, tft::champions::Champion>,
        get_tft_hero_augments: HashMap<String, BasicDatum>,
        get_tft_items: HashMap<String, BasicDatum>,
        get_tft_queues: HashMap<String, Queue>,
        get_tft_regalia: RegaliaData,
        get_tft_tacticians: HashMap<String, Tactician>,
        get_tft_traits: HashMap<String, BasicDatum>,
    );

    pub async fn get_challenges(&self) -> crate::Result<HashMap<i64, Challenge>> {
        Ok(self
            .ddragon
            .get_challenges()
            .await?
            .into_iter()
            .map(|c| (c.id, c))
            .collect())
    }

    pub async fn get_runes(&self) -> crate::Result<HashMap<i64, RuneElementWrapper>> {
        let runes = self.ddragon.get_runes().await?;
        let mut map = HashMap::new();

        for class in runes {
            for (slot_idx, slot) in class.slots.into_iter().enumerate() {
                let siblings = slot.runes.len() as u64;
                for (i, rune) in slot.runes.into_iter().enumerate() {
                    map.insert(
                        rune.id,
                        RuneElementWrapper {
                            rune,
                            slot: slot_idx as u64,
                            index: i as u64,
                            siblings,
                            parent: class.name.clone(),
                            parent_id: class.id,
                        },
                    );
                }
            }
        }

        Ok(map)
    }

    pub async fn get_spell_buffs(&self) -> crate::Result<HashMap<i64, String>> {
        Ok(self
            .ddragon
            .get_spell_buffs()
            .await?
            .spell_buffs
            .into_iter()
            .map(|s| (s.id, s.name))
            .collect())
    }

    pub async fn get_summoner_spells(&self) -> crate::Result<HashMap<i64, String>> {
        Ok(self
            .ddragon
            .get_summoner_spells()
            .await?
            .data
            .into_iter()
            .filter_map(|(_, s)| {
                if let Ok(k) = s.key.parse::<i64>() {
                    Some((k, s.name))
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn get_champion(&self, key: &str) -> crate::Result<Champion> {
        self.ddragon.get_champion(key).await
    }

    pub async fn get_image_of<T: HasImage + Sync>(&self, item: &T) -> crate::Result<DynamicImage> {
        self.ddragon.get_image_of(item).await
    }

    pub async fn get_sprite_of<T: HasImage + Sync>(&self, item: &T) -> crate::Result<DynamicImage> {
        self.ddragon.get_sprite_of(item).await
    }

    pub async fn get_ugg_api_versions(&self) -> crate::Result<UggApiVersions> {
        self.get("https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json").await
    }
}

impl Client {
    pub async fn new<R: Runtime>(
        app: &AppHandle<R>,
        version: Option<&str>,
        locale: Option<&str>,
    ) -> crate::Result<Self> {
        let mut ddragon = DdragonClientWrapper::new(app, version, locale).await?;
        let mut version = ddragon.version().to_owned();
        let ddragon_versions = ddragon.get_supported_versions().await?;
        let ugg_api_versions = ddragon.get_ugg_api_versions().await?;
        let supported_versions = ddragon_versions
            .into_iter()
            .filter_map(|v| {
                let ugg = v.split('.').take(2).collect::<Box<[_]>>().join("_");
                // BUG: https://github.com/rust-lang/rust-clippy/issues/14449
                #[allow(clippy::map_entry)]
                if ugg_api_versions.contains_key(&ugg) {
                    Some(Versions { ddragon: v, ugg })
                } else {
                    None
                }
            })
            .collect::<Box<[_]>>();

        if let Some(v) = supported_versions.first() {
            if !supported_versions
                .iter()
                .any(|Versions { ddragon, .. }| ddragon == &version)
            {
                ddragon = DdragonClientWrapper::new(app, Some(&v.ddragon), locale).await?;
                ddragon.version().clone_into(&mut version);
            }
        } else {
            return Err(UggError::NoSupportedVersions.into());
        }

        let champions = ddragon.get_champions().await?;
        let items = ddragon.get_items().await?;
        let runes = ddragon.get_runes().await?;
        let summoner_spells = ddragon.get_summoner_spells().await?;

        let mut version_split = version.split('.').collect::<Vec<_>>();
        version_split.remove(version_split.len() - 1);
        let patch_version = version_split.join("_");

        Ok(Self {
            ddragon,
            ugg_api_versions,

            version,
            supported_versions,
            patch_version,
            champions,
            items,
            runes,
            summoner_spells,
        })
    }

    pub fn ddragon(&self) -> DdragonClientWrapper {
        self.ddragon.clone()
    }

    pub const fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn search_champion(&self, name: &str) -> &ChampionShort {
        if self.champions.contains_key(name) {
            &self.champions[name]
        } else {
            let mut d_min = usize::MAX;
            let mut closest_champ: &ChampionShort = &self.champions["Annie"];

            let mut d_min_prefix = usize::MAX;
            let mut closest_champ_prefix: Option<&ChampionShort> = None;

            for value in self.champions.values() {
                let query_cmp = name.to_lowercase();
                let champ_cmp = value.name.to_lowercase();
                // Prefer matches where the query is an exact prefix.
                let d = levenshtein::levenshtein(query_cmp.as_str(), champ_cmp.as_str());
                if champ_cmp.starts_with(&query_cmp) {
                    if d <= d_min_prefix {
                        d_min_prefix = d;
                        closest_champ_prefix = Some(value);
                    }
                } else if d <= d_min {
                    d_min = d;
                    closest_champ = value;
                }
            }

            closest_champ_prefix.unwrap_or(closest_champ)
        }
    }

    fn ugg_api_version<'a>(
        api_versions: &'a UggApiVersions,
        patch: &'a str,
        key: &'a str,
    ) -> &'a str {
        match api_versions.get(patch) {
            Some(patch) if patch.contains_key(key) => &patch[key],
            _ => "1.5.0",
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn get_ugg_data<WrappedData, B, KeyFn, MapFn, Data>(
        &self,
        cache: &Cache<WrappedData>,
        cache_key: String,
        path: &str,
        region: mappings::Region,
        role: mappings::Role,
        max_by_key: KeyFn,
        map: MapFn,
    ) -> crate::Result<(Data, mappings::Role)>
    where
        WrappedData: Clone + DeserializeOwned,
        B: Ord,
        KeyFn: FnMut(&(&mappings::Role, &WrappedData)) -> B,
        MapFn: FnOnce((&mappings::Role, &WrappedData)) -> (Data, mappings::Role),
    {
        let data = {
            let mut lock = cache.lock().await;
            let data = if let Some(data) = lock.get(&cache_key).cloned() {
                Ok(data)
            } else {
                self.ddragon
                    .get(&format!("https://stats2.u.gg/lol/1.5/{path}.json"))
                    .await
            }?;
            lock.put(cache_key, data.clone());
            data
        };
        let data_by_role = mappings::Rank::preferred_order()
            .iter()
            .find_map(|rank| {
                data.get(&region)
                    .and_then(|region_data| region_data.get(rank))
            })
            .ok_or(UggError::MissingRegionOrRank)?;

        Ok(data_by_role
            .get_key_value(&role)
            .or_else(|| {
                data_by_role
                    .iter()
                    .max_by_key(max_by_key)
                    .map(|(role, _)| role)
                    .and_then(|r| data_by_role.get_key_value(r))
            })
            .map(map)
            .ok_or(UggError::MissingRole)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_overview(
        &self,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
        build: mappings::Build,
    ) -> crate::Result<(Overview, mappings::Role)> {
        let version =
            Self::ugg_api_version(&self.ugg_api_versions, &self.patch_version, "overview");
        let path = [
            build.to_api_string(),
            &self.patch_version,
            &mode.to_api_string(),
            &champ.key,
            version,
        ]
        .join("/");

        self.get_ugg_data(
            &self.ddragon.overview_cache,
            format!("{path}{region}{role}"),
            &path,
            region,
            role,
            |(_, data)| data.data.matches(),
            |(role, data)| (data.data.clone(), *role),
        )
        .await
    }

    pub async fn get_matchups(
        &self,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
    ) -> crate::Result<(MatchupData, mappings::Role)> {
        let version =
            Self::ugg_api_version(&self.ugg_api_versions, &self.patch_version, "matchups");
        let path = [
            "matchups",
            &self.patch_version,
            &mode.to_api_string(),
            &champ.key,
            version,
        ]
        .join("/");

        self.get_ugg_data(
            &self.ddragon.matchup_cache,
            format!("{path}{region}{role}"),
            &path,
            region,
            role,
            |(_, data)| data.data.total_matches,
            |(role, data)| (data.data.clone(), *role),
        )
        .await
    }
}

impl<'a> ClientBuilder<'a> {
    pub const fn new() -> Self {
        Self {
            version: None,
            locale: None,
        }
    }

    pub const fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub const fn locale(mut self, locale: &'a str) -> Self {
        self.locale = Some(locale);
        self
    }

    pub async fn build<R: Runtime>(self, app: &AppHandle<R>) -> crate::Result<Client> {
        Client::new(app, self.version, self.locale).await
    }
}
