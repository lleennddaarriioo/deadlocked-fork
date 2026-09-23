use std::ops::Deref;

use crate::cs2::{
    CS2,
    class::{life_state::LifeState, net_vec::NetworkVelocityVector},
};
use glam::{Mat4, Quat, Vec3};
use shared::Team;

/// Common handle-backed functionality shared by all client entities.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BaseEntity {
    pub handle: usize,
}

impl std::ops::Add<usize> for BaseEntity {
    type Output = usize;

    fn add(self, rhs: usize) -> Self::Output {
        self.handle + rhs
    }
}

impl BaseEntity {
    pub(crate) fn new(handle: usize) -> Self {
        Self { handle }
    }

    pub fn health(&self, cs2: &CS2) -> i32 {
        cs2.process.read(self.handle + cs2.offsets.entity.health)
    }

    pub fn max_health(&self, cs2: &CS2) -> i32 {
        cs2.process
            .read(self.handle + cs2.offsets.entity.max_health)
    }

    pub fn team(&self, cs2: &CS2) -> Team {
        cs2.process
            .read_as::<u8, Team>(self.handle + cs2.offsets.entity.team)
    }

    pub fn life_state(&self, cs2: &CS2) -> LifeState {
        cs2.process
            .read_as::<u8, LifeState>(self.handle + cs2.offsets.entity.life_state)
    }

    pub fn game_scene_node(&self, cs2: &CS2) -> usize {
        cs2.process
            .read(self.handle + cs2.offsets.entity.game_scene_node)
    }

    pub fn position(&self, cs2: &CS2) -> Vec3 {
        cs2.process
            .read(self.game_scene_node(cs2) + cs2.offsets.game_scene_node.origin)
    }

    pub fn collision_bounds(&self, cs2: &CS2) -> (Vec3, Vec3) {
        let collision: usize = cs2.process.read(self.handle + cs2.offsets.entity.collision);
        if collision == 0 {
            return (Vec3::ZERO, Vec3::ZERO);
        }
        (
            cs2.process.read(collision + cs2.offsets.collision.mins),
            cs2.process.read(collision + cs2.offsets.collision.maxs),
        )
    }

    #[allow(dead_code)]
    pub fn velocity(&self, cs2: &CS2) -> Vec3 {
        NetworkVelocityVector::read(cs2, self.handle + cs2.offsets.entity.velocity).to_vec()
    }
}

impl Deref for BaseEntity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
