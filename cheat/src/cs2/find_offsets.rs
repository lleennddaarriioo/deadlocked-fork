use std::time::Instant;

use crate::{constants::cs2, cs2::{CS2, offsets::Offsets, schema::Schema}};

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let start = Instant::now();
        let mut offsets = Offsets::default();

        offsets.library.client = self
            .process
            .module_base_address(cs2::CLIENT_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::CLIENT_LIB);
                0
            });
        offsets.library.engine = self
            .process
            .module_base_address(cs2::ENGINE_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::ENGINE_LIB);
                0
            });
        offsets.library.tier0 = self
            .process
            .module_base_address(cs2::TIER0_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::TIER0_LIB);
                0
            });
        offsets.library.input = self
            .process
            .module_base_address(cs2::INPUT_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::INPUT_LIB);
                0
            });
        offsets.library.sdl = self
            .process
            .module_base_address(cs2::SDL_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::SDL_LIB);
                0
            });
        offsets.library.schema = self
            .process
            .module_base_address(cs2::SCHEMA_LIB)
            .unwrap_or_else(|| {
                utils::warn!("missing module {}", cs2::SCHEMA_LIB);
                0
            });

        let resource_offset = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
            .unwrap_or_else(|| {
                utils::warn!("could not get offset for GameResourceServiceClient");
                0
            });
        offsets.interface.resource = resource_offset;

        offsets.interface.entity = if resource_offset != 0 {
            self.process.read::<usize>(offsets.interface.resource + 0x50) + 0x10
        } else {
            0
        };

        let cvar_address = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
            .unwrap_or_else(|| {
                utils::warn!("could not get convar interface offset");
                0
            });
        offsets.interface.cvar = cvar_address;

        let input_address = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
            .unwrap_or_else(|| {
                utils::warn!("could not get input interface offset");
                0
            });
        offsets.interface.input = input_address;

        if let Some(local_player) = self
            .process
            .scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", offsets.library.client)
        {
            offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);
        } else {
            utils::warn!("could not find local player offset");
        }

        if offsets.interface.input != 0 {
            offsets.direct.button_state = self.process.read::<u32>(
                self.process
                    .get_interface_function(offsets.interface.input, 19)
                    + 0x14,
            ) as usize;
        }

        if let Some(view_matrix) = self
            .process
            .scan("C6 83 ? ? 00 00 01 4C 8D 05", offsets.library.client)
        {
            offsets.direct.view_matrix =
                self.process
                    .get_relative_address(view_matrix + 0x0A, 0x0, 0x04);
        } else {
            utils::warn!("could not find view matrix offset");
        }

        if let Some(sdl_window) = self
            .process
            .get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus")
        {
            let sdl_window = self.process.get_relative_address(sdl_window, 0x02, 0x06);
            let sdl_window = self.process.read(sdl_window);
            offsets.direct.sdl_window = self.process.get_relative_address(sdl_window, 0x03, 0x07);
        } else {
            utils::warn!("could not find sdl window offset");
        }



        // xref "lobby_mapveto"
        offsets.direct.global_vars = if let Some(global_vars) = self.process.scan(
            "48 8D 05 ? ? ? ? 45 31 E4 48 8B 00 8B 78 10",
            offsets.library.client,
        ) {
            self.process.get_relative_address(global_vars, 0x03, 0x07)
        } else {
            utils::warn!("could not find global vars offset");
            0
        };

        if let Some(vphys_world) = self.process.scan(
            "4c 8d 35 ? ? ? ? 49 8b 3e e8 ? ? ? ? 48 89 c2",
            offsets.library.client,
        ).or_else(|| {
            self.process.scan(
                "48 8b 0d ? ? ? ? 48 85 c9 74 ? 48 8b 01",
                offsets.library.client,
            )
        }) {
            let vphys_world_global_ptr = self.process.get_relative_address(vphys_world, 3, 7);
            offsets.direct.vphys_world = vphys_world_global_ptr;
        } else {
            utils::warn!("could not find vphys_world offset (radar walls disabled)");
        }

        let ffa_address = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
            .unwrap_or_else(|| {
                utils::warn!("could not get mp_teammates_are_enemies convar offset");
                0
            });
        offsets.convar.ffa = ffa_address;

        let sensitivity_address = self
            .process
            .get_convar(offsets.interface.cvar, "sensitivity")
            .unwrap_or_else(|| {
                utils::warn!("could not get sensitivity convar offset");
                0
            });
        offsets.convar.sensitivity = sensitivity_address;

        let schema = Schema::new(&self.process, offsets.library.schema);
        if schema.is_none() {
            utils::warn!("could not initialize schema system");
        }
        let client = schema.as_ref().and_then(|s| s.get_library(cs2::CLIENT_LIB));
        if client.is_none() {
            utils::warn!("could not find client library in schema");
        }

        let get_offset = |class_name: &str, field_name: &str| -> usize {
            client
                .as_ref()
                .and_then(|c| c.get(class_name, field_name))
                .unwrap_or_else(|| {
                    utils::warn!("missing {}::{}", class_name, field_name);
                    0
                })
        };

        offsets.controller.steam_id = get_offset("CBasePlayerController", "m_steamID");
        offsets.controller.name = get_offset("CBasePlayerController", "m_iszPlayerName");
        offsets.controller.pawn = get_offset("CBasePlayerController", "m_hPawn");
        offsets.controller.desired_fov = get_offset("CBasePlayerController", "m_iDesiredFOV");
        offsets.controller.owner_entity = get_offset("C_BaseEntity", "m_hOwnerEntity");
        offsets.controller.color = get_offset("CCSPlayerController", "m_iCompTeammateColor");
        offsets.controller.action_tracking_services =
            get_offset("CCSPlayerController", "m_pActionTrackingServices");

        offsets.entity.health = get_offset("C_BaseEntity", "m_iHealth");
        offsets.entity.max_health = get_offset("C_BaseEntity", "m_iMaxHealth");
        offsets.entity.team = get_offset("C_BaseEntity", "m_iTeamNum");
        offsets.entity.life_state = get_offset("C_BaseEntity", "m_lifeState");
        offsets.entity.game_scene_node = get_offset("C_BaseEntity", "m_pGameSceneNode");
        offsets.entity.velocity = get_offset("C_BaseEntity", "m_vecVelocity");
        offsets.pawn.armor = get_offset("C_CSPlayerPawn", "m_ArmorValue");
        offsets.pawn.fov_multiplier = get_offset("C_BasePlayerPawn", "m_flFOVSensitivityAdjust");
        offsets.pawn.eye_offset = get_offset("C_BaseModelEntity", "m_vecViewOffset");
        offsets.pawn.eye_angles = get_offset("C_CSPlayerPawn", "m_angEyeAngles");
        offsets.pawn.flags = get_offset("C_BaseEntity", "m_fFlags");
        offsets.pawn.shots_fired = get_offset("C_CSPlayerPawn", "m_iShotsFired");
        offsets.pawn.view_angles = get_offset("C_BasePlayerPawn", "v_angle");
        offsets.pawn.spotted_state = get_offset("C_CSPlayerPawn", "m_entitySpottedState");
        offsets.pawn.crosshair_entity = get_offset("C_CSPlayerPawn", "m_iIDEntIndex");
        offsets.pawn.is_scoped = get_offset("C_CSPlayerPawn", "m_bIsScoped");
        offsets.pawn.flash_alpha = get_offset("C_CSPlayerPawnBase", "m_flFlashMaxAlpha");
        offsets.pawn.flash_duration = get_offset("C_CSPlayerPawnBase", "m_flFlashDuration");
        offsets.pawn.deathmatch_immunity = get_offset("C_CSPlayerPawn", "m_bGunGameImmunity");
        offsets.pawn.is_defusing = get_offset("C_CSPlayerPawn", "m_bIsDefusing");
        offsets.pawn.movement_services = client
            .as_ref()
            .and_then(|c| {
                c.get("C_BasePlayerPawn", "m_pMovementServices")
                    .or_else(|| c.get("C_CSPlayerPawn", "m_pMovementServices"))
            })
            .unwrap_or_else(|| {
                utils::warn!("missing m_pMovementServices");
                0
            });

        offsets.pawn.stamina = client
            .as_ref()
            .and_then(|c| {
                c.get("CCSPlayer_MovementServices", "m_flStamina")
                    .or_else(|| c.get("CPlayer_MovementServices_Humanoid", "m_flStamina"))
                    .or_else(|| c.get("CPlayer_MovementServices", "m_flStamina"))
                    .or_else(|| c.get("CCSPlayer_MovementServices", "m_flStaminaJump"))
                    .or_else(|| c.get("C_CSPlayerPawn", "m_flVelocityModifier"))
            })
            .unwrap_or_else(|| {
                utils::warn!("missing m_flStamina in movement services");
                0
            });

        offsets.pawn.camera_services = get_offset("C_BasePlayerPawn", "m_pCameraServices");
        offsets.pawn.item_services = get_offset("C_BasePlayerPawn", "m_pItemServices");
        offsets.pawn.weapon_services = get_offset("C_BasePlayerPawn", "m_pWeaponServices");
        offsets.pawn.observer_services = get_offset("C_BasePlayerPawn", "m_pObserverServices");
        offsets.pawn.aim_punch_services = get_offset("C_CSPlayerPawn", "m_pAimPunchServices");

        offsets.game_scene_node.dormant = get_offset("CGameSceneNode", "m_bDormant");
        offsets.game_scene_node.origin = get_offset("CGameSceneNode", "m_vecAbsOrigin");
        offsets.game_scene_node.model_state = get_offset("CSkeletonInstance", "m_modelState");

        offsets.model_state.skeleton_instance =
            get_offset("CBodyComponentSkeletonInstance", "m_skeletonInstance");

        offsets.smoke.did_smoke_effect =
            get_offset("C_SmokeGrenadeProjectile", "m_bDidSmokeEffect");
        offsets.smoke.smoke_color = get_offset("C_SmokeGrenadeProjectile", "m_vSmokeColor");

        offsets.molotov.is_incendiary = get_offset("C_MolotovProjectile", "m_bIsIncGrenade");

        offsets.inferno.is_burning = get_offset("C_Inferno", "m_bFireIsBurning");
        offsets.inferno.fire_count = get_offset("C_Inferno", "m_fireCount");
        offsets.inferno.fire_positions = get_offset("C_Inferno", "m_firePositions");

        offsets.spotted_state.mask = get_offset("EntitySpottedState_t", "m_bSpottedByMask");

        offsets.action_tracking.round_kills = get_offset(
            "CCSPlayerController_ActionTrackingServices",
            "m_iNumRoundKills",
        );
        offsets.action_tracking.round_damage = get_offset(
            "CCSPlayerController_ActionTrackingServices",
            "m_flTotalRoundDamageDealt",
        );

        offsets.camera_services.fov = get_offset("CCSPlayerBase_CameraServices", "m_iFOV");

        offsets.item_services.has_defuser =
            get_offset("CCSPlayer_ItemServices", "m_bHasDefuser");
        offsets.item_services.has_helmet = get_offset("CCSPlayer_ItemServices", "m_bHasHelmet");

        offsets.weapon_services.active_weapon =
            get_offset("CPlayer_WeaponServices", "m_hActiveWeapon");
        offsets.weapon_services.weapons = get_offset("CPlayer_WeaponServices", "m_hMyWeapons");

        offsets.observer_services.target =
            get_offset("CPlayer_ObserverServices", "m_hObserverTarget");

        let aim_punch_tick = get_offset("CCSPlayer_AimPunchServices", "m_unpredictableBaseTick");
        offsets.aim_punch_services.aim_punch_cache = if aim_punch_tick != 0 {
            aim_punch_tick - 0x18
        } else {
            0
        };

        offsets.weapon.attribute_manager = get_offset("C_EconEntity", "m_AttributeManager");
        offsets.weapon.item = get_offset("C_AttributeContainer", "m_Item");
        offsets.weapon.clip_primary = get_offset("C_BasePlayerWeapon", "m_iClip1");
        offsets.weapon.reserve_ammo = get_offset("C_BasePlayerWeapon", "m_pReserveAmmo");
        offsets.weapon.inaccuracy = client
            .as_ref()
            .and_then(|c| {
                c.get("C_CSWeaponBase", "m_fAccuracyPenalty")
                    .or_else(|| c.get("C_BasePlayerWeapon", "m_fAccuracyPenalty"))
            })
            .unwrap_or_else(|| {
                utils::warn!("missing m_fAccuracyPenalty");
                0
            });

        offsets.econ_item_view.item_definition_index =
            get_offset("C_EconItemView", "m_iItemDefinitionIndex");

        offsets.planted_c4.is_ticking = get_offset("C_PlantedC4", "m_bBombTicking");
        offsets.planted_c4.blow_time = get_offset("C_PlantedC4", "m_flC4Blow");
        offsets.planted_c4.being_defused = get_offset("C_PlantedC4", "m_bBeingDefused");
        offsets.planted_c4.is_defused = get_offset("C_PlantedC4", "m_bBombDefused");
        offsets.planted_c4.has_exploded = get_offset("C_PlantedC4", "m_bHasExploded");
        offsets.planted_c4.defuse_time_left = get_offset("C_PlantedC4", "m_flDefuseCountDown");

        offsets.entity_identity.size = client
            .as_ref()
            .and_then(|c| c.get_class("CEntityIdentity"))
            .map(|c| c.size() as usize)
            .unwrap_or_else(|| {
                utils::warn!("missing class CEntityIdentity");
                0
            });

        utils::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }
}
