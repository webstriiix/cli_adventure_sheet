use uuid::Uuid;

use serde_json::json;

use crate::models::{
    AddCharacterClassRequest, AddInventoryRequest, AddSpellRequest, AsiChoiceRequest, Character,
    CharacterFeat, CharacterSpell, CharacterSpellSlot, CreateCharacterRequest, Feat, InventoryItem,
    PatchCharacterClassRequest, UpdateCharacterRequest, UpdateInventoryRequest, UpdateSpellRequest,
};

use super::{ApiClient, ApiError};

// ── Character CRUD ──

impl ApiClient {
    pub async fn create_character(
        &self,
        req: &CreateCharacterRequest,
    ) -> Result<Character, ApiError> {
        let resp = self.auth_post("/characters").json(req).send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_characters(&self) -> Result<Vec<Character>, ApiError> {
        let resp = self.auth_get("/characters").send().await?;
        self.handle_response(resp).await
    }

    pub async fn get_character(&self, id: Uuid) -> Result<Character, ApiError> {
        let resp = self.auth_get(&format!("/characters/{id}")).send().await?;
        self.handle_response(resp).await
    }

    pub async fn update_character(
        &self,
        id: Uuid,
        req: &UpdateCharacterRequest,
    ) -> Result<Character, ApiError> {
        let resp = self
            .auth_put(&format!("/characters/{id}"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn delete_character(&self, id: Uuid) -> Result<(), ApiError> {
        let resp = self
            .auth_delete(&format!("/characters/{id}"))
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }

    // ── Feats ──

    pub async fn get_feats(&self, character_id: Uuid) -> Result<Vec<CharacterFeat>, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/feats"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn remove_feat(&self, character_id: Uuid, feat_id: i32) -> Result<(), ApiError> {
        let resp = self
            .auth_delete(&format!("/characters/{character_id}/feats/{feat_id}"))
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }

    // ── Spells ──

    pub async fn get_character_spells(
        &self,
        character_id: Uuid,
    ) -> Result<Vec<CharacterSpell>, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/spells"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn add_spell(
        &self,
        character_id: Uuid,
        req: &AddSpellRequest,
    ) -> Result<CharacterSpell, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/spells"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn update_spell(
        &self,
        character_id: Uuid,
        spell_id: i32,
        req: &UpdateSpellRequest,
    ) -> Result<CharacterSpell, ApiError> {
        let resp = self
            .auth_put(&format!("/characters/{character_id}/spells/{spell_id}"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn remove_spell(&self, character_id: Uuid, spell_id: i32) -> Result<(), ApiError> {
        let resp = self
            .auth_delete(&format!("/characters/{character_id}/spells/{spell_id}"))
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }

    // ── Inventory ──

    pub async fn get_inventory(&self, character_id: Uuid) -> Result<Vec<InventoryItem>, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/inventory"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn add_inventory_item(
        &self,
        character_id: Uuid,
        req: &AddInventoryRequest,
    ) -> Result<InventoryItem, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/inventory"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn update_inventory_item(
        &self,
        character_id: Uuid,
        inventory_id: i32,
        req: &UpdateInventoryRequest,
    ) -> Result<InventoryItem, ApiError> {
        let resp = self
            .auth_put(&format!(
                "/characters/{character_id}/inventory/{inventory_id}"
            ))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn remove_inventory_item(
        &self,
        character_id: Uuid,
        item_id: i32,
    ) -> Result<(), ApiError> {
        let resp = self
            .auth_delete(&format!("/characters/{character_id}/inventory/{item_id}"))
            .send()
            .await?;
        self.handle_empty_response(resp).await
    }

    // ── Resources ──

    pub async fn get_spell_slots(
        &self,
        character_id: Uuid,
    ) -> Result<Vec<CharacterSpellSlot>, ApiError> {
        // Try the collection endpoint first
        let resp = self
            .auth_get(&format!("/characters/{character_id}/spell-slots"))
            .send()
            .await?;
        if !resp.status().is_success() {
            // Collection endpoint unavailable; fetch each slot level individually
            let mut slots = Vec::new();
            for level in 1..=9 {
                let r = self
                    .auth_get(&format!("/characters/{character_id}/spell-slots/{level}"))
                    .send()
                    .await?;
                if r.status().is_success() {
                    if let Ok(slot) = self.handle_response::<CharacterSpellSlot>(r).await {
                        if slot.expended > 0 {
                            slots.push(slot);
                        }
                    }
                }
            }
            return Ok(slots);
        }
        self.handle_response(resp).await
    }

    pub async fn get_hit_dice(
        &self,
        character_id: Uuid,
    ) -> Result<Vec<crate::models::CharacterHitDice>, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/hit-dice"))
            .send()
            .await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        self.handle_response(resp).await
    }

    pub async fn patch_spell_slot(
        &self,
        character_id: Uuid,
        level: i32,
        expended: i32,
    ) -> Result<CharacterSpellSlot, ApiError> {
        let resp = self
            .auth_patch(&format!("/characters/{character_id}/spell-slots/{level}"))
            .json(&json!({ "expended": expended }))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn patch_hit_dice(
        &self,
        character_id: Uuid,
        size: i32,
        expended: i32,
    ) -> Result<crate::models::CharacterHitDice, ApiError> {
        let resp = self
            .auth_patch(&format!("/characters/{character_id}/hit-dice/{size}"))
            .json(&json!({ "expended": expended }))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn patch_feature_uses(
        &self,
        character_id: Uuid,
        feat_id: i32,
        uses_remaining: i32,
    ) -> Result<crate::models::CharacterFeat, ApiError> {
        let resp = self
            .auth_patch(&format!("/characters/{character_id}/features/{feat_id}"))
            .json(&json!({ "uses_remaining": uses_remaining }))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    // ── Actions ──

    pub async fn get_character_actions(
        &self,
        character_id: Uuid,
    ) -> Result<crate::models::actions::CharacterActionsResponse, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/actions"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    // ── Available Feats & ASI Choice ──

    /// GET /characters/{id}/available-feats
    /// Returns feats the character qualifies for at their current level/class.
    pub async fn get_available_feats(&self, character_id: Uuid) -> Result<Vec<Feat>, ApiError> {
        let resp = self
            .auth_get(&format!("/characters/{character_id}/available-feats"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    /// POST /characters/{id}/asi-choice
    /// Submit an ASI (ability score increase) or feat choice for a level-up milestone.
    pub async fn post_asi_choice(
        &self,
        character_id: Uuid,
        req: &AsiChoiceRequest,
    ) -> Result<Character, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/asi-choice"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    // ── Multiclass ──

    /// POST /characters/{id}/classes — add a new class (multiclass).
    /// Returns the updated Character object.
    pub async fn add_character_class(
        &self,
        character_id: Uuid,
        req: &AddCharacterClassRequest,
    ) -> Result<Character, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/classes"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    /// PATCH /characters/{id}/classes/{class_id} — update level and/or subclass.
    /// Returns the updated Character object.
    pub async fn patch_character_class(
        &self,
        character_id: Uuid,
        class_id: i32,
        req: &PatchCharacterClassRequest,
    ) -> Result<Character, ApiError> {
        let resp = self
            .auth_patch(&format!("/characters/{character_id}/classes/{class_id}"))
            .json(req)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    // ── Death Saves ──

    pub async fn patch_death_saves(
        &self,
        character_id: Uuid,
        successes: i32,
        failures: i32,
    ) -> Result<Character, ApiError> {
        let resp = self
            .auth_patch(&format!("/characters/{character_id}/death-saves"))
            .json(&json!({ "successes": successes, "failures": failures }))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    // ── Resting ──

    pub async fn long_rest(&self, character_id: Uuid) -> Result<Character, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/long-rest"))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn short_rest(&self, character_id: Uuid) -> Result<Character, ApiError> {
        let resp = self
            .auth_post(&format!("/characters/{character_id}/short-rest"))
            .json(&json!({ "hit_dice_spent": {} }))
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn patch_resource_uses(
        &self,
        character_id: Uuid,
        resource_name: &str,
        uses_remaining: i32,
    ) -> Result<(), ApiError> {
        let resp = self
            .auth_patch(&format!(
                "/characters/{character_id}/resources/{resource_name}"
            ))
            .json(&serde_json::json!({ "uses_remaining": uses_remaining }))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            self.handle_response::<()>(resp).await
        }
    }
}
