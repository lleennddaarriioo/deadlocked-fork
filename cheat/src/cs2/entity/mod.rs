use shared::{entity::GrenadeInfo, weapon::Weapon};

use crate::{
    constants::cs2::class,
    cs2::{
        CS2,
        entity::{
            inferno::Inferno, molotov::Molotov, planted_c4::PlantedC4, player::Player,
            smoke::Smoke, weapon::weapon_from_entity,
        },
    },
};

pub mod inferno;
pub mod molotov;
pub mod planted_c4;
pub mod player;
pub mod smoke;
pub mod weapon;
pub mod weapon_class;

#[derive(Debug, Clone)]
pub enum Entity {
    Weapon { weapon: Weapon, entity: u64 },
    Inferno(Inferno),
    Smoke(Smoke),
    Molotov(Molotov),
    Flashbang(u64),
    HeGrenade(u64),
    Decoy(u64),
}

pub fn grenade_info(entity: u64, name: &'static str, cs2: &CS2) -> GrenadeInfo {
    GrenadeInfo {
        entity,
        position: Player::entity(entity).position(cs2),
        name: name.to_owned(),
    }
}

use rayon::prelude::*;

struct BucketResult {
    players: Vec<Player>,
    dead_players: Vec<Player>,
    entities: Vec<Entity>,
    planted_c4: Option<PlantedC4>,
    local_pawn_index: Option<u64>,
}

impl CS2 {
    pub fn cache_entities(&mut self) {
        crate::profile_scope!("cache_entities");
        self.players.clear();
        self.dead_players.clear();
        self.entities.clear();
        self.planted_c4 = None;

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        self.weapon = local_player.weapon(self);

        const NUM_BUCKETS: usize = 64;
        let bucket_pointers = self
            .process
            .read_vec(self.offsets.interface.entity, 0x8 * NUM_BUCKETS);

        let results: Vec<BucketResult> = (0..64)
            .into_par_iter()
            .map(|bucket_index| {
                let bucket_pointer = *bytemuck::from_bytes(&bucket_pointers[bucket_index * 8..(bucket_index + 1) * 8]);
                self.get_entities_in_bucket(bucket_index as u64, bucket_pointer, &local_player)
            })
            .collect();

        for res in results {
            self.players.extend(res.players);
            self.dead_players.extend(res.dead_players);
            self.entities.extend(res.entities);
            if res.planted_c4.is_some() {
                self.planted_c4 = res.planted_c4;
            }
            if let Some(pawn_idx) = res.local_pawn_index {
                self.target.local_pawn_index = pawn_idx;
            }
        }
    }

    fn get_entities_in_bucket(
        &self,
        bucket_index: u64,
        bucket_ptr: u64,
        local_player: &Player,
    ) -> BucketResult {
        let mut result = BucketResult {
            players: Vec::new(),
            dead_players: Vec::new(),
            entities: Vec::new(),
            planted_c4: None,
            local_pawn_index: None,
        };

        if bucket_ptr == 0 || bucket_ptr >> 48 != 0 {
            return result;
        }
        const IDENTITIES_PER_BUCKET: usize = 512;
        let bucket = self.process.read_vec(
            bucket_ptr,
            IDENTITIES_PER_BUCKET * self.offsets.entity_identity.size as usize,
        );
        for index_in_bucket in 0..IDENTITIES_PER_BUCKET {
            let identity_offset = index_in_bucket * self.offsets.entity_identity.size as usize;
            if identity_offset + 24 > bucket.len() {
                continue;
            }

            let entity: u64 = *bytemuck::from_bytes(&bucket[identity_offset..identity_offset + 8]);
            if entity == 0 {
                continue;
            }

            let handle_start = identity_offset + 0x10;
            let handle: u32 = *bytemuck::from_bytes(&bucket[handle_start..handle_start + 4]);
            let handle_index = handle & 0x7FFF;
            let entity_index =
                (bucket_index as usize * IDENTITIES_PER_BUCKET + index_in_bucket) as u32;
            if entity_index != handle_index {
                continue;
            }

            let vtable: u64 = self.process.read(entity);
            let rtti: u64 = self.process.read(vtable - 0x8);
            let name_ptr: u64 = self.process.read(rtti + 0x8);
            let name = self.process.read_string(name_ptr);

            match name.as_str() {
                class::PLAYER_CONTROLLER => {
                    let Some(player) = Player::from_controller(entity, self) else {
                        continue;
                    };

                    if !player.is_valid(self) {
                        result.dead_players.push(player);
                        continue;
                    }

                    if player == *local_player {
                        result.local_pawn_index = Some((handle as u64 & 0x7FFF) - 1);
                    } else {
                        result.players.push(player);
                    }
                }
                class::PLANTED_C4 => {
                    let planted_c4 = PlantedC4::new(entity);
                    if planted_c4.is_relevant(self) {
                        result.planted_c4 = Some(planted_c4);
                    }
                }
                class::INFERNO => {
                    result.entities.push(Entity::Inferno(Inferno::new(entity)));
                }
                class::SMOKE => {
                    result.entities.push(Entity::Smoke(Smoke::new(entity)));
                }
                class::MOLOTOV => result.entities.push(Entity::Molotov(Molotov::new(entity))),
                class::FLASHBANG => result.entities.push(Entity::Flashbang(entity)),
                class::HE_GRENADE => result.entities.push(Entity::HeGrenade(entity)),
                class::DECOY => result.entities.push(Entity::Decoy(entity)),
                _ => {
                    let entity_identity: u64 = self.process.read(entity + 0x10);
                    if entity_identity == 0 {
                        continue;
                    }

                    let name_pointer = self.process.read(entity_identity + 0x20);
                    if name_pointer == 0 {
                        continue;
                    }

                    let name = self.process.read_string(name_pointer);

                    if name.starts_with("weapon_") {
                        if self.entity_has_owner(entity) {
                            continue;
                        }

                        let weapon = weapon_from_entity(entity, self);
                        if weapon == Weapon::Unknown {
                            continue;
                        }

                        result.entities.push(Entity::Weapon { weapon, entity });
                    }
                }
            }
        }
        result
    }
}
