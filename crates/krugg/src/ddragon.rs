#![allow(unused)]

use std::{collections::HashMap, path::PathBuf};

use ddragon::models::{
    Challenges, Champion, Champions, ChampionsFull, Items, Maps, MissionAssets, ProfileIcons,
    Runes, SpellBuffs, SummonerSpells, Translations,
    shared::HasImage,
    tft::{self, Arenas, Augments, HeroAugments, Queues, Regalia, Tacticians, Traits},
};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use image::DynamicImage;
use reqwest_middleware::{
    ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware, RequestBuilder,
};
use serde::{Deserialize, de::DeserializeOwned};
use tauri_plugin_http::reqwest::{Client as ReqwestClient, Url};

use crate::error::DdragonError;

#[derive(Debug, Clone)]
pub struct Client {
    client: ClientWithMiddleware,
    version: String,
    base_url: Url,
    locale: String,
}

#[derive(Debug, Clone)]
pub enum ClientType {
    Middleware(ClientWithMiddleware),
    Plain(ReqwestClient),
}

#[derive(Debug)]
pub struct ClientBuilder<'a> {
    base_url: &'a str,
    client: Option<ClientType>,
    cache_path: Option<PathBuf>,
    version: Option<&'a str>,
    locale: Option<&'a str>,
}

#[derive(Deserialize)]
struct ChampionWrapper {
    format: String,
    version: String,
    data: HashMap<String, Champion>,
}

impl<'a> ClientBuilder<'a> {
    pub const fn new() -> Self {
        Self {
            base_url: "https://ddragon.leagueoflegends.com",
            client: None,
            cache_path: None,
            version: None,
            locale: None,
        }
    }

    pub const fn base_url(mut self, base_url: &'a str) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn client_with_middleware(mut self, client: ClientWithMiddleware) -> Self {
        self.client = Some(ClientType::Middleware(client));
        self
    }

    pub fn client(mut self, client: ReqwestClient) -> Self {
        self.client = Some(ClientType::Plain(client));
        self
    }

    pub fn cache<P: Into<PathBuf>>(mut self, cache_path: P) -> Self {
        self.cache_path = Some(cache_path.into());
        self
    }

    pub const fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub const fn locale(mut self, locale: &'a str) -> Self {
        self.locale = Some(locale);
        self
    }

    async fn get(client: ClientType, url: Url) -> crate::Result<Box<[String]>> {
        Ok(match client {
            ClientType::Middleware(client) => {
                client
                    .get(url)
                    .send()
                    .await?
                    .json::<Box<[String]>>()
                    .await?
            }
            ClientType::Plain(client) => {
                client
                    .get(url)
                    .send()
                    .await?
                    .json::<Box<[String]>>()
                    .await?
            }
        })
    }

    /// Builds the [`Client`]. Adds caching middleware if a
    /// [`ClientType::Plain`] client was provided with `cache_path`.
    pub async fn build(self) -> crate::Result<Client> {
        let client = if let Some(client) = self.client {
            client
        } else {
            // TODO: zstd https://github.com/tauri-apps/plugins-workspace/pull/2561
            ClientType::Plain(ReqwestClient::builder().brotli(true).build()?)
        };
        let base_url = Url::parse(self.base_url)?;
        let versions = Self::get(client.clone(), base_url.join("/api/versions.json")?).await?;
        let version = match self.version {
            Some(v) if versions.iter().any(|version| version == v) => v.to_string(),
            _ => versions
                .first() // List is sorted by version number descending
                .cloned()
                .ok_or(DdragonError::NoLatestVersion)?,
        };
        let locale = match self.locale {
            Some(l)
                if Self::get(client.clone(), base_url.join("/cdn/languages.json")?)
                    .await?
                    .iter()
                    .any(|lang| lang == l) =>
            {
                l
            }
            _ => "en_US",
        };
        let middleware_client = match client {
            ClientType::Middleware(client) => client,
            ClientType::Plain(client) => match self.cache_path {
                Some(path) => MiddlewareClientBuilder::new(client)
                    .with(Cache(HttpCache {
                        mode: CacheMode::ForceCache,
                        manager: CACacheManager { path },
                        options: HttpCacheOptions::default(),
                    }))
                    .build(),
                None => MiddlewareClientBuilder::new(client).build(),
            },
        };

        Ok(Client {
            client: middleware_client,
            version,
            base_url,
            locale: locale.to_owned(),
        })
    }
}

macro_rules! impl_endpoints {
    ( $($name:ident : $path:literal, $t:ty ),* $(,)? ) => {
        $(
            pub async fn $name(&self) -> crate::Result<$t> {
                self.get::<$t>(concat!("./", $path, ".json")).await
            }
        )*
    };
}

impl Client {
    pub async fn new<P: Into<PathBuf>>(cache_path: P) -> crate::Result<Self> {
        ClientBuilder::new().cache(cache_path).build().await
    }

    pub(crate) fn client(&self) -> ClientWithMiddleware {
        self.client.clone()
    }

    pub const fn version(&self) -> &str {
        self.version.as_str()
    }

    pub const fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub const fn locale(&self) -> &str {
        self.locale.as_str()
    }

    fn url(&self) -> crate::Result<Url> {
        Ok(self
            .base_url
            .join(&format!("/cdn/{}/data/{}/", self.version, self.locale))?)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> crate::Result<T> {
        Ok(self
            .client
            .get(self.url()?.join(path)?)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    impl_endpoints! {
        get_challenges: "challenges", Challenges,
        get_champions: "champion", Champions,
        get_champions_full: "championFull", ChampionsFull,
        get_items: "item", Items,
        get_maps: "map", Maps,
        get_mission_assets: "mission-assets", MissionAssets,
        get_profile_icons: "profileicon", ProfileIcons,
        get_runes: "runesReforged", Runes,
        get_spell_buffs: "spellbuffs", SpellBuffs,
        get_summoner_spells: "summoner", SummonerSpells,
        get_translations: "language", Translations,
        get_tft_arenas: "tft-arena", Arenas,
        get_tft_augments: "tft-augments", Augments,
        get_tft_champions: "tft-champion", tft::Champions,
        get_tft_hero_augments: "tft-hero-augments", HeroAugments,
        get_tft_items: "tft-item", tft::Items,
        get_tft_queues: "tft-queues", Queues,
        get_tft_regalia: "tft-regalia", Regalia,
        get_tft_tacticians: "tft-tactician", Tacticians,
        get_tft_traits: "tft-trait", Traits,
    }

    pub async fn get_champion(&self, key: &str) -> crate::Result<Champion> {
        Ok(self
            .get::<ChampionWrapper>(&format!("./champion/{key}.json"))
            .await?
            .data
            .get(key)
            .cloned()
            .ok_or_else(|| DdragonError::NoChampionData(key.to_owned()))?)
    }

    async fn get_image(&self, path: Url) -> crate::Result<DynamicImage> {
        let response = self.client.get(path.as_str()).send().await?;
        Ok(image::load_from_memory(&response.bytes().await?)?)
    }

    pub async fn get_image_of<T: HasImage + Sync>(&self, item: &T) -> crate::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.image_path()
        ))?)
        .await
    }

    pub async fn get_sprite_of<T: HasImage + Sync>(&self, item: &T) -> crate::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.sprite_path()
        ))?)
        .await
    }
}
