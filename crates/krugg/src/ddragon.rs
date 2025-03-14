#![allow(unused)]

use std::{collections::HashMap, path::PathBuf};

use anyhow::anyhow;
use ddragon::models::{
    Challenges, Champion, Champions, ChampionsFull, Items, Maps, MissionAssets, ProfileIcons,
    Runes, SpellBuffs, SummonerSpells, Translations,
    shared::HasImage,
    tft::{self, Arenas, Augments, HeroAugments, Queues, Regalia, Tacticians, Traits},
};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use image::DynamicImage;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, de::DeserializeOwned};
use tauri_plugin_http::reqwest::{Client as ReqwestClient, Url};

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

    async fn get(client: ClientType, url: Url) -> anyhow::Result<Box<[String]>> {
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
    pub async fn build(self) -> anyhow::Result<Client> {
        let client = self
            .client
            .map_or_else(|| ClientType::Plain(ReqwestClient::new()), |client| client);
        let base_url = Url::parse(self.base_url)?;
        let versions = Self::get(client.clone(), base_url.join("/api/versions.json")?).await?;
        let version = match self.version {
            Some(v) if versions.iter().any(|version| version == v) => v.to_string(),
            _ => versions
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("no latest version"))?,
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

macro_rules! impl_endpoint {
    ($name:ident, $kind:literal, $path:literal, $t:ty) => {
        pub async fn $name(&self) -> anyhow::Result<$t> {
            self.get::<$t>(concat!("./", $path, ".json")).await
        }
    };
}

impl Client {
    pub async fn new<P: Into<PathBuf>>(cache_path: P) -> anyhow::Result<Self> {
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

    fn url(&self) -> anyhow::Result<Url> {
        Ok(self
            .base_url
            .join(&format!("/cdn/{}/data/{}/", self.version, self.locale))?)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        Ok(self
            .client
            .get(self.url()?.join(path)?)
            .send()
            .await?
            .json::<T>()
            .await?)
    }

    impl_endpoint!(challenges, "challenge", "challenges", Challenges);
    impl_endpoint!(champions, "champion", "champion", Champions);
    impl_endpoint!(
        champions_full,
        "complete champion",
        "championFull",
        ChampionsFull
    );
    impl_endpoint!(items, "item", "item", Items);
    impl_endpoint!(maps, "map", "map", Maps);
    impl_endpoint!(
        mission_assets,
        "mission asset",
        "mission-assets",
        MissionAssets
    );
    impl_endpoint!(profile_icons, "profile icon", "profileicon", ProfileIcons);
    impl_endpoint!(runes, "rune", "runesReforged", Runes);
    impl_endpoint!(spell_buffs, "spell buff", "spellbuffs", SpellBuffs);
    impl_endpoint!(
        summoner_spells,
        "summoner_spells",
        "summoner",
        SummonerSpells
    );
    impl_endpoint!(translations, "translation", "language", Translations);
    impl_endpoint!(tft_arenas, "TFT arena", "tft-arena", Arenas);
    impl_endpoint!(tft_augments, "TFT augment", "tft-augments", Augments);
    impl_endpoint!(
        tft_champions,
        "TFT champion",
        "tft-champion",
        tft::Champions
    );
    impl_endpoint!(
        tft_hero_augments,
        "TFT hero augment",
        "tft-hero-augments",
        HeroAugments
    );
    impl_endpoint!(tft_items, "TFT item", "tft-item", tft::Items);
    impl_endpoint!(tft_queues, "TFT queue", "tft-queues", Queues);
    impl_endpoint!(tft_regalia, "TFT regalia", "tft-regalia", Regalia);
    impl_endpoint!(tft_tacticians, "TFT tactician", "tft-tactician", Tacticians);
    impl_endpoint!(tft_traits, "TFT trait", "tft-trait", Traits);

    pub async fn champion(&self, key: &str) -> anyhow::Result<Champion> {
        self.get::<ChampionWrapper>(&format!("./champion/{key}.json"))
            .await?
            .data
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("no champion data for key {}", key))
    }

    async fn get_image(&self, path: Url) -> anyhow::Result<DynamicImage> {
        let response = self.client.get(path.as_str()).send().await?;
        Ok(image::load_from_memory(&response.bytes().await?)?)
    }

    pub async fn image_of<T: HasImage + Sync>(&self, item: &T) -> anyhow::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.image_path()
        ))?)
        .await
    }

    pub async fn sprite_of<T: HasImage + Sync>(&self, item: &T) -> anyhow::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.sprite_path()
        ))?)
        .await
    }
}
