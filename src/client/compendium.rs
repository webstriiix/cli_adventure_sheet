use crate::models::{
    Background, Class, ClassDetailResponse, Feat, Item, Monster, OptionalFeature, Race, Spell,
};

use super::{ApiClient, ApiError};

impl ApiClient {
    pub async fn get_classes(
        &self,
        source: Option<&str>,
        edition: Option<&str>,
    ) -> Result<Vec<Class>, ApiError> {
        let mut req = self.http.get(self.url("/classes"));
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        if let Some(e) = edition {
            req = req.query(&[("edition", e)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_class_detail(
        &self,
        name: &str,
        source: &str,
    ) -> Result<ClassDetailResponse, ApiError> {
        let resp = self
            .auth_get(&format!("/classes/{name}/{source}"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn get_spells(
        &self,
        name: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<Spell>, ApiError> {
        let mut req = self.auth_get("/spells");
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_items(
        &self,
        name: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<Item>, ApiError> {
        let mut req = self.http.get(self.url("/items"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_monsters(
        &self,
        name: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<Monster>, ApiError> {
        let mut req = self.http.get(self.url("/monsters"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_races(
        &self,
        name: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<Race>, ApiError> {
        let mut req = self.http.get(self.url("/races"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_backgrounds(
        &self,
        name: Option<&str>,
        source: Option<&str>,
    ) -> Result<Vec<Background>, ApiError> {
        let mut req = self.http.get(self.url("/backgrounds"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_compendium_feats(&self, name: Option<&str>) -> Result<Vec<Feat>, ApiError> {
        let mut req = self.http.get(self.url("/feats"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_optional_features(
        &self,
        name: Option<&str>,
        source: Option<&str>,
        feature_type: Option<&str>,
    ) -> Result<Vec<OptionalFeature>, ApiError> {
        let mut req = self.http.get(self.url("/optional-features"));
        if let Some(n) = name {
            req = req.query(&[("name", n)]);
        }
        if let Some(s) = source {
            req = req.query(&[("source", s)]);
        }
        if let Some(ft) = feature_type {
            req = req.query(&[("feature_type", ft)]);
        }
        let resp = req.send().await?;
        self.handle_response(resp).await
    }
}
