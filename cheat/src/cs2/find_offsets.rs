use std::time::Instant;

use crate::{
    constants::cs2,
    cs2::{CS2, offsets::Offsets, schema::Schema},
};

impl CS2 {
    pub fn find_offsets(&self) -> Option<Offsets> {
        let start = Instant::now();
        let mut offsets = Offsets::default();

        offsets.library.client = self.process.module_base_address(cs2::CLIENT_LIB)?;
        offsets.library.engine = self.process.module_base_address(cs2::ENGINE_LIB)?;
        offsets.library.tier0 = self.process.module_base_address(cs2::TIER0_LIB)?;
        offsets.library.input = self.process.module_base_address(cs2::INPUT_LIB)?;
        offsets.library.sdl = self.process.module_base_address(cs2::SDL_LIB)?;
        offsets.library.schema = self.process.module_base_address(cs2::SCHEMA_LIB)?;

        let Some(resource_offset) = self
            .process
            .get_interface_offset(offsets.library.engine, "GameResourceServiceClientV0")
        else {
            utils::warn!("could not get offset for GameResourceServiceClient");
            return None;
        };
        offsets.interface.resource = resource_offset;

        offsets.interface.entity =
            self.process.read::<u64>(offsets.interface.resource + 0x50) + 0x10;

        let Some(cvar_address) = self
            .process
            .get_interface_offset(offsets.library.tier0, "VEngineCvar0")
        else {
            utils::warn!("could not get convar interface offset");
            return None;
        };
        offsets.interface.cvar = cvar_address;
        let Some(input_address) = self
            .process
            .get_interface_offset(offsets.library.input, "InputSystemVersion0")
        else {
            utils::warn!("could not get input interface offset");
            return None;
        };
        offsets.interface.input = input_address;

        let Some(local_player) = self
            .process
            .scan("48 83 3D ? ? ? ? 00 0F 95 C0 C3", offsets.library.client)
        else {
            utils::warn!("could not find local player offset");
            return None;
        };
        offsets.direct.local_player = self.process.get_relative_address(local_player, 0x03, 0x08);
        offsets.direct.button_state = self.process.read::<u32>(
            self.process
                .get_interface_function(offsets.interface.input, 19)
                + 0x14,
        ) as u64;

        let Some(view_matrix) = self
            .process
            .scan("C6 83 ? ? 00 00 01 4C 8D 05", offsets.library.client)
        else {
            utils::warn!("could not find view matrix offset");
            return None;
        };

        offsets.direct.view_matrix =
            self.process
                .get_relative_address(view_matrix + 0x0A, 0x0, 0x04);

        let Some(sdl_window) = self
            .process
            .get_module_export(offsets.library.sdl, "SDL_GetKeyboardFocus")
        else {
            utils::warn!("could not find sdl window offset");
            return None;
        };
        let sdl_window = self.process.get_relative_address(sdl_window, 0x02, 0x06);
        let sdl_window = self.process.read(sdl_window);
        offsets.direct.sdl_window = self.process.get_relative_address(sdl_window, 0x03, 0x07);

        let Some(planted_c4) = self.process.scan(
            "48 8D 35 ? ? ? ? 66 0F EF C0 C6 05 ? ? ? ? 01 48 8D 3D",
            offsets.library.client,
        ) else {
            utils::warn!("could not find planted c4 offset");
            return None;
        };
        offsets.direct.planted_c4 = self.process.get_relative_address(planted_c4, 0x03, 0x0E);

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
        ) {
            let vphys_world_global_ptr = self.process.get_relative_address(vphys_world, 3, 7);
            offsets.direct.vphys_world = vphys_world_global_ptr;
        } else {
            utils::warn!("could not find vphys_world offset (radar walls disabled)");
        }

        let ffa_address = self
            .process
            .get_convar(offsets.interface.cvar, "mp_teammates_are_enemies")
            .unwrap_or_else(|| {
                utils::warn!("could not get mp_tammates_are_enemies convar offset");
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

        let schema = Schema::new(&self.process, offsets.library.schema)?;
        let client = schema.get_library(cs2::CLIENT_LIB)?;

        offsets.controller.steam_id = client.get("CBasePlayerController", "m_steamID").unwrap_or_else(|| { utils::warn!("missing CBasePlayerController::m_steamID"); 0 });
        offsets.controller.name = client.get("CBasePlayerController", "m_iszPlayerName").unwrap_or_else(|| { utils::warn!("missing CBasePlayerController::m_iszPlayerName"); 0 });
        offsets.controller.pawn = client.get("CBasePlayerController", "m_hPawn").unwrap_or_else(|| { utils::warn!("missing CBasePlayerController::m_hPawn"); 0 });
        offsets.controller.desired_fov = client.get("CBasePlayerController", "m_iDesiredFOV").unwrap_or_else(|| { utils::warn!("missing CBasePlayerController::m_iDesiredFOV"); 0 });
        offsets.controller.owner_entity = client.get("C_BaseEntity", "m_hOwnerEntity").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_hOwnerEntity"); 0 });
        offsets.controller.color = client.get("CCSPlayerController", "m_iCompTeammateColor").unwrap_or_else(|| { utils::warn!("missing CCSPlayerController::m_iCompTeammateColor"); 0 });
        offsets.controller.action_tracking_services =
            client.get("CCSPlayerController", "m_pActionTrackingServices").unwrap_or_else(|| { utils::warn!("missing CCSPlayerController::m_pActionTrackingServices"); 0 });

        offsets.pawn.health = client.get("C_BaseEntity", "m_iHealth").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_iHealth"); 0 });
        offsets.pawn.armor = client.get("C_CSPlayerPawn", "m_ArmorValue").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_ArmorValue"); 0 });
        offsets.pawn.team = client.get("C_BaseEntity", "m_iTeamNum").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_iTeamNum"); 0 });
        offsets.pawn.life_state = client.get("C_BaseEntity", "m_lifeState").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_lifeState"); 0 });
        offsets.pawn.fov_multiplier = client.get("C_BasePlayerPawn", "m_flFOVSensitivityAdjust").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::m_flFOVSensitivityAdjust"); 0 });
        offsets.pawn.game_scene_node = client.get("C_BaseEntity", "m_pGameSceneNode").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_pGameSceneNode"); 0 });
        offsets.pawn.eye_offset = client.get("C_BaseModelEntity", "m_vecViewOffset").unwrap_or_else(|| { utils::warn!("missing C_BaseModelEntity::m_vecViewOffset"); 0 });
        offsets.pawn.eye_angles = client.get("C_CSPlayerPawn", "m_angEyeAngles").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_angEyeAngles"); 0 });
        offsets.pawn.velocity = client.get("C_BaseEntity", "m_vecAbsVelocity").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_vecAbsVelocity"); 0 });
        offsets.pawn.flags = client.get("C_BaseEntity", "m_fFlags").unwrap_or_else(|| { utils::warn!("missing C_BaseEntity::m_fFlags"); 0 });
        offsets.pawn.shots_fired = client.get("C_CSPlayerPawn", "m_iShotsFired").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_iShotsFired"); 0 });
        offsets.pawn.view_angles = client.get("C_BasePlayerPawn", "v_angle").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::v_angle"); 0 });
        offsets.pawn.spotted_state = client.get("C_CSPlayerPawn", "m_entitySpottedState").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_entitySpottedState"); 0 });
        offsets.pawn.crosshair_entity = client.get("C_CSPlayerPawn", "m_iIDEntIndex").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_iIDEntIndex"); 0 });
        offsets.pawn.is_scoped = client.get("C_CSPlayerPawn", "m_bIsScoped").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_bIsScoped"); 0 });
        offsets.pawn.flash_alpha = client.get("C_CSPlayerPawnBase", "m_flFlashMaxAlpha").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawnBase::m_flFlashMaxAlpha"); 0 });
        offsets.pawn.flash_duration = client.get("C_CSPlayerPawnBase", "m_flFlashDuration").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawnBase::m_flFlashDuration"); 0 });
        offsets.pawn.deathmatch_immunity = client.get("C_CSPlayerPawn", "m_bGunGameImmunity").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_bGunGameImmunity"); 0 });
        offsets.pawn.is_defusing = client.get("C_CSPlayerPawn", "m_bIsDefusing").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_bIsDefusing"); 0 });
        offsets.pawn.movement_services = client.get("C_BasePlayerPawn", "m_pMovementServices")
            .or_else(|| client.get("C_CSPlayerPawn", "m_pMovementServices"))
            .unwrap_or_else(|| { utils::warn!("missing m_pMovementServices"); 0 });

        offsets.pawn.stamina = client.get("CCSPlayer_MovementServices", "m_flStamina")
            .or_else(|| client.get("CPlayer_MovementServices_Humanoid", "m_flStamina"))
            .or_else(|| client.get("CPlayer_MovementServices", "m_flStamina"))
            .or_else(|| client.get("CCSPlayer_MovementServices", "m_flStaminaJump"))
            .or_else(|| client.get("C_CSPlayerPawn", "m_flVelocityModifier"))
            .unwrap_or_else(|| { utils::warn!("missing m_flStamina in movement services"); 0 });

        offsets.pawn.camera_services = client.get("C_BasePlayerPawn", "m_pCameraServices").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::m_pCameraServices"); 0 });
        offsets.pawn.item_services = client.get("C_BasePlayerPawn", "m_pItemServices").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::m_pItemServices"); 0 });
        offsets.pawn.weapon_services = client.get("C_BasePlayerPawn", "m_pWeaponServices").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::m_pWeaponServices"); 0 });
        offsets.pawn.observer_services = client.get("C_BasePlayerPawn", "m_pObserverServices").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerPawn::m_pObserverServices"); 0 });
        offsets.pawn.aim_punch_services = client.get("C_CSPlayerPawn", "m_pAimPunchServices").unwrap_or_else(|| { utils::warn!("missing C_CSPlayerPawn::m_pAimPunchServices"); 0 });

        offsets.game_scene_node.dormant = client.get("CGameSceneNode", "m_bDormant").unwrap_or_else(|| { utils::warn!("missing CGameSceneNode::m_bDormant"); 0 });
        offsets.game_scene_node.origin = client.get("CGameSceneNode", "m_vecAbsOrigin").unwrap_or_else(|| { utils::warn!("missing CGameSceneNode::m_vecAbsOrigin"); 0 });
        offsets.game_scene_node.model_state = client.get("CSkeletonInstance", "m_modelState").unwrap_or_else(|| { utils::warn!("missing CSkeletonInstance::m_modelState"); 0 });

        offsets.model_state.skeleton_instance =
            client.get("CBodyComponentSkeletonInstance", "m_skeletonInstance").unwrap_or_else(|| { utils::warn!("missing CBodyComponentSkeletonInstance::m_skeletonInstance"); 0 });

        offsets.smoke.did_smoke_effect =
            client.get("C_SmokeGrenadeProjectile", "m_bDidSmokeEffect").unwrap_or_else(|| { utils::warn!("missing C_SmokeGrenadeProjectile::m_bDidSmokeEffect"); 0 });
        offsets.smoke.smoke_color = client.get("C_SmokeGrenadeProjectile", "m_vSmokeColor").unwrap_or_else(|| { utils::warn!("missing C_SmokeGrenadeProjectile::m_vSmokeColor"); 0 });

        offsets.molotov.is_incendiary = client.get("C_MolotovProjectile", "m_bIsIncGrenade").unwrap_or_else(|| { utils::warn!("missing C_MolotovProjectile::m_bIsIncGrenade"); 0 });

        offsets.inferno.is_burning = client.get("C_Inferno", "m_bFireIsBurning").unwrap_or_else(|| { utils::warn!("missing C_Inferno::m_bFireIsBurning"); 0 });
        offsets.inferno.fire_count = client.get("C_Inferno", "m_fireCount").unwrap_or_else(|| { utils::warn!("missing C_Inferno::m_fireCount"); 0 });
        offsets.inferno.fire_positions = client.get("C_Inferno", "m_firePositions").unwrap_or_else(|| { utils::warn!("missing C_Inferno::m_firePositions"); 0 });

        offsets.spotted_state.mask = client.get("EntitySpottedState_t", "m_bSpottedByMask").unwrap_or_else(|| { utils::warn!("missing EntitySpottedState_t::m_bSpottedByMask"); 0 });

        offsets.action_tracking.round_kills = client.get(
            "CCSPlayerController_ActionTrackingServices",
            "m_iNumRoundKills",
        )?;
        offsets.action_tracking.round_damage = client.get(
            "CCSPlayerController_ActionTrackingServices",
            "m_flTotalRoundDamageDealt",
        )?;

        offsets.camera_services.fov = client.get("CCSPlayerBase_CameraServices", "m_iFOV").unwrap_or_else(|| { utils::warn!("missing CCSPlayerBase_CameraServices::m_iFOV"); 0 });

        offsets.item_services.has_defuser =
            client.get("CCSPlayer_ItemServices", "m_bHasDefuser").unwrap_or_else(|| { utils::warn!("missing CCSPlayer_ItemServices::m_bHasDefuser"); 0 });
        offsets.item_services.has_helmet = client.get("CCSPlayer_ItemServices", "m_bHasHelmet").unwrap_or_else(|| { utils::warn!("missing CCSPlayer_ItemServices::m_bHasHelmet"); 0 });

        offsets.weapon_services.active_weapon =
            client.get("CPlayer_WeaponServices", "m_hActiveWeapon").unwrap_or_else(|| { utils::warn!("missing CPlayer_WeaponServices::m_hActiveWeapon"); 0 });
        offsets.weapon_services.weapons = client.get("CPlayer_WeaponServices", "m_hMyWeapons").unwrap_or_else(|| { utils::warn!("missing CPlayer_WeaponServices::m_hMyWeapons"); 0 });

        offsets.observer_services.target =
            client.get("CPlayer_ObserverServices", "m_hObserverTarget").unwrap_or_else(|| { utils::warn!("missing CPlayer_ObserverServices::m_hObserverTarget"); 0 });

        offsets.aim_punch_services.aim_punch_cache =
            client.get("CCSPlayer_AimPunchServices", "m_unpredictableBaseTick").unwrap_or_else(|| { utils::warn!("missing CCSPlayer_AimPunchServices::m_unpredictableBaseTick"); 0 }) - 0x18;

        offsets.weapon.attribute_manager = client.get("C_EconEntity", "m_AttributeManager").unwrap_or_else(|| { utils::warn!("missing C_EconEntity::m_AttributeManager"); 0 });
        offsets.weapon.item = client.get("C_AttributeContainer", "m_Item").unwrap_or_else(|| { utils::warn!("missing C_AttributeContainer::m_Item"); 0 });
        offsets.weapon.clip_primary = client.get("C_BasePlayerWeapon", "m_iClip1").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerWeapon::m_iClip1"); 0 });
        offsets.weapon.reserve_ammo = client.get("C_BasePlayerWeapon", "m_pReserveAmmo").unwrap_or_else(|| { utils::warn!("missing C_BasePlayerWeapon::m_pReserveAmmo"); 0 });
        offsets.weapon.inaccuracy = client.get("C_CSWeaponBase", "m_fAccuracyPenalty").unwrap_or_else(|| {
            client.get("C_BasePlayerWeapon", "m_fAccuracyPenalty").unwrap_or_else(|| {
                utils::warn!("missing m_fAccuracyPenalty");
                0
            })
        });

        offsets.econ_item_view.item_definition_index =
            client.get("C_EconItemView", "m_iItemDefinitionIndex").unwrap_or_else(|| { utils::warn!("missing C_EconItemView::m_iItemDefinitionIndex"); 0 });

        offsets.planted_c4.is_ticking = client.get("C_PlantedC4", "m_bBombTicking").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_bBombTicking"); 0 });
        offsets.planted_c4.blow_time = client.get("C_PlantedC4", "m_flC4Blow").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_flC4Blow"); 0 });
        offsets.planted_c4.being_defused = client.get("C_PlantedC4", "m_bBeingDefused").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_bBeingDefused"); 0 });
        offsets.planted_c4.is_defused = client.get("C_PlantedC4", "m_bBombDefused").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_bBombDefused"); 0 });
        offsets.planted_c4.has_exploded = client.get("C_PlantedC4", "m_bHasExploded").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_bHasExploded"); 0 });
        offsets.planted_c4.defuse_time_left = client.get("C_PlantedC4", "m_flDefuseCountDown").unwrap_or_else(|| { utils::warn!("missing C_PlantedC4::m_flDefuseCountDown"); 0 });

        offsets.entity_identity.size = client.get_class("CEntityIdentity").map(|c| c.size()).unwrap_or_else(|| { utils::warn!("missing class CEntityIdentity"); 0 });

        utils::debug!("offsets: {:?} ({:?})", offsets, Instant::now() - start);
        Some(offsets)
    }
}
