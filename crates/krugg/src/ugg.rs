#![allow(unused)]

use std::{collections::HashMap, num::NonZeroUsize};

use anyhow::{anyhow, bail};
use ddragon::models::{champions::ChampionShort, items::Item, runes::RuneElement};
use lru::LruCache;
use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager, Runtime, async_runtime::Mutex};
use tauri_plugin_http::reqwest::IntoUrl;
use ugg_types::{
    mappings,
    matchups::{MatchupData, WrappedMatchupData},
    overview::{OverviewData, WrappedOverviewData},
    rune::RuneExtended,
};

use crate::ddragon::{Client as DdragonClient, ClientBuilder as DdragonClientBuilder};

type Cache<T> = Mutex<
    LruCache<
        String,
        HashMap<mappings::Region, HashMap<mappings::Rank, HashMap<mappings::Role, T>>>,
    >,
>;
type OverviewCache = Cache<WrappedOverviewData>;
type MatchupCache = Cache<WrappedMatchupData>;
type UggApiVersions = HashMap<String, HashMap<String, String>>;

#[derive(Debug)]
pub struct DdragonClientWrapper {
    ddragon: DdragonClient,
    overview_cache: OverviewCache,
    matchup_cache: MatchupCache,
}

#[derive(Debug)]
pub struct Versions {
    ddragon: String,
    ugg: String,
}

#[derive(Debug)]
pub struct ClientBuilder<'a> {
    version: Option<&'a str>,
    locale: Option<&'a str>,
}

pub struct Client {
    client: DdragonClientWrapper,
    ugg_api_versions: UggApiVersions,

    version: String,
    allowed_versions: Box<[Versions]>,
    patch_version: String,
    champions: HashMap<String, ChampionShort>,
    items: HashMap<String, Item>,
    runes: HashMap<i64, RuneExtended<RuneElement>>,
    summoner_spells: HashMap<i64, String>,
}

impl DdragonClientWrapper {
    pub async fn new<R: Runtime>(
        app: &AppHandle<R>,
        version: Option<&str>,
        locale: Option<&str>,
    ) -> anyhow::Result<Self> {
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
            overview_cache: Mutex::new(LruCache::new(cache_size)),
            matchup_cache: Mutex::new(LruCache::new(cache_size)),
        })
    }

    pub fn client(&self) -> ClientWithMiddleware {
        self.ddragon.client()
    }

    pub const fn version(&self) -> &str {
        self.ddragon.version()
    }

    async fn get<T: DeserializeOwned, U: IntoUrl>(&self, url: U) -> anyhow::Result<T> {
        Ok(self
            .ddragon
            .client()
            .get(url)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    pub async fn get_supported_versions(&self) -> anyhow::Result<Box<[String]>> {
        self.get(self.ddragon.base_url().join("/api/versions.json")?)
            .await
    }

    pub async fn get_champions(&self) -> anyhow::Result<HashMap<String, ChampionShort>> {
        Ok(self.ddragon.champions().await?.data)
    }

    pub async fn get_items(&self) -> anyhow::Result<HashMap<String, Item>> {
        Ok(self.ddragon.items().await?.data)
    }

    pub async fn get_runes(&self) -> anyhow::Result<HashMap<i64, RuneExtended<RuneElement>>> {
        let runes = self.ddragon.runes().await?;
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
        let summoners = self.ddragon.summoner_spells().await?;
        let mut data = HashMap::new();

        for (_, spell) in summoners.data {
            data.insert(spell.key.parse::<i64>()?, spell.name);
        }

        Ok(data)
    }

    pub async fn get_ugg_api_versions(&self) -> anyhow::Result<UggApiVersions> {
        self.get("https://static.bigbrain.gg/assets/lol/riot_patch_update/prod/ugg/ugg-api-versions.json").await
    }

    fn ugg_api_version<'a>(api_versions: &'a UggApiVersions, patch: &'a str) -> &'a str {
        match api_versions.get(patch) {
            Some(patch) if patch.contains_key("overview") => &patch["overview"],
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
    ) -> anyhow::Result<(Data, mappings::Role)>
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
                self.get(&format!("https://stats2.u.gg/lol/1.5/{path}.json"))
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
            .ok_or_else(|| anyhow!("missing region or rank"))?;

        data_by_role
            .get_key_value(&role)
            .or_else(|| {
                data_by_role
                    .iter()
                    .max_by_key(max_by_key)
                    .map(|(role, _)| role)
                    .and_then(|r| data_by_role.get_key_value(r))
            })
            .map(map)
            .ok_or_else(|| anyhow!("missing role"))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_stats(
        &self,
        api_versions: &UggApiVersions,
        patch: &str,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
        build: mappings::Build,
    ) -> anyhow::Result<(OverviewData, mappings::Role)> {
        let version = Self::ugg_api_version(api_versions, patch);
        let path = [
            build.to_api_string(),
            patch,
            &mode.to_api_string(),
            &champ.key,
            version,
        ]
        .join("/");

        self.get_ugg_data(
            &self.overview_cache,
            format!("{path}{region}{role}"),
            &path,
            region,
            role,
            |(_, data)| data.data.matches,
            |(role, data)| (data.data.clone(), *role),
        )
        .await
    }

    pub async fn get_matchups(
        &self,
        api_versions: &UggApiVersions,
        patch: &str,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
    ) -> anyhow::Result<(MatchupData, mappings::Role)> {
        let version = Self::ugg_api_version(api_versions, patch);
        let path = [
            "matchups",
            patch,
            &mode.to_api_string(),
            &champ.key,
            version,
        ]
        .join("/");

        self.get_ugg_data(
            &self.matchup_cache,
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

impl Client {
    pub async fn new<R: Runtime>(
        app: &AppHandle<R>,
        version: Option<&str>,
        locale: Option<&str>,
    ) -> anyhow::Result<Self> {
        let mut client = DdragonClientWrapper::new(app, version, locale).await?;
        let mut version = client.version().to_owned();
        let allowed_versions = client.get_supported_versions().await?;
        let ugg_api_versions = client.get_ugg_api_versions().await?;
        let supported_versions = allowed_versions
            .into_iter()
            .filter_map(|v| {
                let ugg = v.split('.').take(2).collect::<Box<[_]>>().join(".");
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
                client = DdragonClientWrapper::new(app, Some(&v.ddragon), locale).await?;
                version = client.version().to_owned();
            }
        } else {
            bail!("no supported versions");
        }

        let champions = client.get_champions().await?;
        let items = client.get_items().await?;
        let runes = client.get_runes().await?;
        let summoner_spells = client.get_summoner_spells().await?;

        let mut version_split = version.split('.').collect::<Vec<_>>();
        version_split.remove(version_split.len() - 1);
        let patch_version = version_split.join("_");

        Ok(Self {
            client,
            ugg_api_versions,

            version,
            allowed_versions: supported_versions,
            patch_version,
            champions,
            items,
            runes,
            summoner_spells,
        })
    }

    pub fn client(&self) -> ClientWithMiddleware {
        self.client.client()
    }

    pub const fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn find_champion(&self, name: &str) -> &ChampionShort {
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

    pub async fn get_stats(
        &self,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
        build: mappings::Build,
    ) -> anyhow::Result<(OverviewData, mappings::Role)> {
        self.client
            .get_stats(
                &self.ugg_api_versions,
                &self.patch_version,
                champ,
                role,
                region,
                mode,
                build,
            )
            .await
    }

    pub async fn get_matchups(
        &self,
        champ: &ChampionShort,
        role: mappings::Role,
        region: mappings::Region,
        mode: mappings::Mode,
    ) -> anyhow::Result<(MatchupData, mappings::Role)> {
        self.client
            .get_matchups(
                &self.ugg_api_versions,
                &self.patch_version,
                champ,
                role,
                region,
                mode,
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

    pub async fn build<R: Runtime>(self, app: &AppHandle<R>) -> anyhow::Result<Client> {
        Client::new(app, self.version, self.locale).await
    }
}
