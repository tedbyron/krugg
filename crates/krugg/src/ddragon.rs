use std::path::PathBuf;

use anyhow::anyhow;
use ddragon::models::{
    Challenges, Champion, Champions, ChampionsFull, Items, Maps, MissionAssets, ProfileIcons,
    Runes, SpellBuffs, SummonerSpells, Translations,
    champion::ChampionWrapper,
    shared::HasImage,
    tft::{self, Arenas, Augments, HeroAugments, Queues, Regalia, Tacticians, Traits},
};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use image::DynamicImage;
use reqwest_middleware::{ClientBuilder as MiddlewareClientBuilder, ClientWithMiddleware};
use serde::{Deserialize, de::DeserializeOwned};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_http::reqwest::{Client as ReqwestClient, Url};

#[derive(Debug, Clone)]
pub struct Client {
    client: ClientWithMiddleware,
    pub version: String,
    base_url: Url,
    pub locale: String,
}

#[derive(Debug, Clone)]
pub enum ClientType {
    Middleware(ClientWithMiddleware),
    Plain(ReqwestClient),
}

#[derive(Debug)]
pub struct ClientBuilder<'a> {
    base_url: &'a str,
    client: Option<Client>,
    cache_dir: Option<PathBuf>,
    version: Option<&'a str>,
    locale: &'a str,
}

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        Self {
            base_url: "https://ddragon.leagueoflegends.com",
            client: None,
            cache_dir: None,
            version: None,
            locale: "en_US",
        }
    }

    pub fn base_url(mut self, base_url: &'a str) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn client_with_middleware(mut self, client: ClientWithMiddleware) -> Self {
        self.client = Some(ClientType::Left(client));
        self
    }

    pub fn client(mut self, client: ReqwestClient) -> Self {
        self.client = Some(ClientType::Right(client));
        self
    }

    pub fn cache<P: Into<PathBuf>>(mut self, cache_dir: P) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    pub fn version<V: AsRef<str>>(mut self, version: Option<V>) -> Self {
        self.version = version.map(|v| v.as_ref());
        self
    }

    pub fn locale<L: AsRef<str>>(mut self, locale: L) -> Self {
        self.locale = locale.as_ref();
        self
    }

    async fn get<T: Deserialize>(&self, path: &str) -> anyhow::Result<T> {
        match client {
            ClientType::Middleware(client) => {
                client
                    .get(base_url.join(path)?)
                    .send()
                    .await?
                    .json::<Box<[str]>>()
                    .await?
            }
            ClientType::Plain(client) => {
                client
                    .get(base_url.join(path)?)
                    .send()
                    .await?
                    .json::<Box<[str]>>()
                    .await?
            }
        }
    }

    pub async fn build(self) -> anyhow::Result<Client> {
        let client = match self.client {
            Some(client) => client,
            None => ClientType::Plain(ReqwestClient::new()),
        };
        let base_url = Url::parse(self.base_url)?;
        let version = match self.version {
            Some(v) => v,
            None => self
                .get::<Box<[str]>>("/api/version.json")
                .await?
                .get(0)
                .ok_or_else(|| anyhow!("no latest version"))?,
        };
        let locale = if self.locale == "en_US"
            || self
                .get::<Box<[str]>>("/cdn/languages.json")
                .await?
                .contains(self.locale)
        {
            self.locale
        } else {
            "en_US"
        };
        let middleware_client = match client {
            ClientType::Middleware(client) => client,
            ClientType::Plain(client) => match self.cache_dir {
                Some(cache_dir) => MiddlewareClientBuilder::new(client).with(Cache(HttpCache {
                    mode: CacheMode::ForceCache,
                    manager: CACacheManager { path: cache_dir },
                    options: HttpCacheOptions::default(),
                })),
                None => MiddlewareClientBuilder::new(client).build(),
            },
        };

        Ok(Client {
            client: middleware_client,
            version: version.to_owned(),
            base_url,
            locale: locale.to_owned(),
        })
    }
}

macro_rules! create_endpoint {
    ($name:ident, $kind:literal, $path:literal, $ret:ty) => {
        pub async fn $name(&self) -> anyhow::Result<$ret> {
            self.get::<$ret>(concat!("./", $path, ".json")).await
        }
    };
}

impl Client {
    pub async fn new(cache_dir: P) -> anyhow::Result<Self> {
        ClientBuilder::new().cache(cache_dir).build().await
    }

    fn url(&self) -> anyhow::Result<Url> {
        self.base_url
            .join(&format!("/cdn/{}/data/{}/", self.version, self.locale))
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

    create_endpoint!(challenges, "challenge", "challenges", Challenges);
    create_endpoint!(champions, "champion", "champion", Champions);
    create_endpoint!(
        champions_full,
        "complete champion",
        "championFull",
        ChampionsFull
    );
    create_endpoint!(items, "item", "item", Items);
    create_endpoint!(maps, "map", "map", Maps);
    create_endpoint!(
        mission_assets,
        "mission asset",
        "mission-assets",
        MissionAssets
    );
    create_endpoint!(profile_icons, "profile icon", "profileicon", ProfileIcons);
    create_endpoint!(runes, "rune", "runesReforged", Runes);
    create_endpoint!(spell_buffs, "spell buff", "spellbuffs", SpellBuffs);
    create_endpoint!(
        summoner_spells,
        "summoner_spells",
        "summoner",
        SummonerSpells
    );
    create_endpoint!(translations, "translation", "language", Translations);
    create_endpoint!(tft_arenas, "TFT arena", "tft-arena", Arenas);
    create_endpoint!(tft_augments, "TFT augment", "tft-augments", Augments);
    create_endpoint!(
        tft_champions,
        "TFT champion",
        "tft-champion",
        tft::Champions
    );
    create_endpoint!(
        tft_hero_augments,
        "TFT hero augment",
        "tft-hero-augments",
        HeroAugments
    );
    create_endpoint!(tft_items, "TFT item", "tft-item", tft::Items);
    create_endpoint!(tft_queues, "TFT queue", "tft-queues", Queues);
    create_endpoint!(tft_regalia, "TFT regalia", "tft-regalia", Regalia);
    create_endpoint!(tft_tacticians, "TFT tactician", "tft-tactician", Tacticians);
    create_endpoint!(tft_traits, "TFT trait", "tft-trait", Traits);

    pub async fn champion(&self, key: &str) -> anyhow::Result<Champion> {
        self.get::<ChampionWrapper>(&format!("./champion/{key}.json"))
            .await?
            .data
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("no champion data for key {}", key))
    }

    async fn get_image(&self, path: Url) -> anyhow::Result<DynamicImage> {
        let response = self.agent.get(path.as_str()).send().await?;
        Ok(image::load_from_memory(&response.bytes().await?)?)
    }

    pub async fn image_of<T: HasImage>(&self, item: &T) -> anyhow::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.image_path()
        ))?)
        .await
    }

    pub async fn sprite_of<T: HasImage>(&self, item: &T) -> anyhow::Result<DynamicImage> {
        self.get_image(self.base_url.join(&format!(
            "/cdn/{}/img/{}",
            &self.version,
            item.sprite_path()
        ))?)
        .await
    }
}
