use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(
    Debug, Default, Clone, PartialEq, Eq, Hash, EnumIter, AsRefStr, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Weapon {
    #[default]
    Unknown,

    Knife,

    // Pistols
    Cz75A,
    Deagle,
    DualBerettas,
    FiveSeven,
    Glock,
    P2000,
    P250,
    Revolver,
    Tec9,
    Usp,

    // SMGs
    Bizon,
    Mac10,
    Mp5Sd,
    Mp7,
    Mp9,
    P90,
    Ump45,

    // LMGs
    M249,
    Negev,

    // Shotguns
    Mag7,
    Nova,
    Sawedoff,
    Xm1014,

    // Rifles
    Ak47,
    Aug,
    Famas,
    Galilar,
    M4A4,
    M4A1,
    Sg556,

    // Snipers
    Awp,
    G3SG1,
    Scar20,
    Ssg08,

    // Utility
    Taser,

    // Grenades
    Flashbang,
    HeGrenade,
    Smoke,
    Molotov,
    Decoy,
    Incendiary,

    // Bomb
    C4,
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Weapon::Unknown => "Unknown",
            Weapon::Knife => "Knife",
            Weapon::Cz75A => "CZ75-Auto",
            Weapon::Deagle => "Desert Eagle",
            Weapon::DualBerettas => "Dual Berettas",
            Weapon::FiveSeven => "Five-SeveN",
            Weapon::Glock => "Glock-18",
            Weapon::P2000 => "P2000",
            Weapon::P250 => "P250",
            Weapon::Revolver => "R8 Revolver",
            Weapon::Tec9 => "Tec-9",
            Weapon::Usp => "USP-S",
            Weapon::Bizon => "PP-Bizon",
            Weapon::Mac10 => "MAC-10",
            Weapon::Mp5Sd => "MP5-SD",
            Weapon::Mp7 => "MP7",
            Weapon::Mp9 => "MP9",
            Weapon::P90 => "P90",
            Weapon::Ump45 => "UMP-45",
            Weapon::M249 => "M249",
            Weapon::Negev => "Negev",
            Weapon::Mag7 => "MAG-7",
            Weapon::Nova => "Nova",
            Weapon::Sawedoff => "Sawed-Off",
            Weapon::Xm1014 => "XM1014",
            Weapon::Ak47 => "AK-47",
            Weapon::Aug => "AUG",
            Weapon::Famas => "FAMAS",
            Weapon::Galilar => "Galil AR",
            Weapon::M4A4 => "M4A4",
            Weapon::M4A1 => "M4A1-S",
            Weapon::Sg556 => "SG 553",
            Weapon::Awp => "AWP",
            Weapon::G3SG1 => "G3SG1",
            Weapon::Scar20 => "SCAR-20",
            Weapon::Ssg08 => "SSG 08",
            Weapon::Taser => "Zeus x27",
            Weapon::Flashbang => "Flashbang",
            Weapon::HeGrenade => "HE Grenade",
            Weapon::Smoke => "Smoke Grenade",
            Weapon::Molotov => "Molotov Cocktail",
            Weapon::Decoy => "Decoy Grenade",
            Weapon::Incendiary => "Incendiary Grenade",
            Weapon::C4 => "C4 Explosive",
        };
        write!(f, "{}", s)
    }
}

impl Weapon {
    pub fn base_damage(&self) -> u32 {
        match self {
            Weapon::Knife => 40,
            Weapon::Cz75A => 31,
            Weapon::Deagle => 53,
            Weapon::DualBerettas => 38,
            Weapon::FiveSeven => 32,
            Weapon::Glock => 28,
            Weapon::P2000 => 26,
            Weapon::P250 => 38,
            Weapon::Revolver => 86,
            Weapon::Tec9 => 33,
            Weapon::Usp => 26,
            Weapon::Bizon => 26,
            Weapon::Mac10 => 29,
            Weapon::Mp5Sd => 27,
            Weapon::Mp7 => 29,
            Weapon::Mp9 => 26,
            Weapon::P90 => 26,
            Weapon::Ump45 => 35,
            Weapon::M249 => 32,
            Weapon::Negev => 26,
            Weapon::Mag7 => 30,
            Weapon::Nova => 26,
            Weapon::Sawedoff => 32,
            Weapon::Xm1014 => 20,
            Weapon::Ak47 => 36,
            Weapon::Aug => 28,
            Weapon::Famas => 30,
            Weapon::Galilar => 30,
            Weapon::M4A4 => 33,
            Weapon::M4A1 => 38,
            Weapon::Sg556 => 30,
            Weapon::Awp => 115,
            Weapon::G3SG1 => 80,
            Weapon::Scar20 => 80,
            Weapon::Ssg08 => 88,
            Weapon::Taser => 100,
            Weapon::HeGrenade => 98,
            _ => 0,
        }
    }

    pub fn damage_description(&self) -> String {
        match self {
            Weapon::Knife => "40 (stab: 55, backstab: 90)".to_string(),
            Weapon::Mag7 => "30 x 8 pellets (240 max)".to_string(),
            Weapon::Nova => "26 x 9 pellets (234 max)".to_string(),
            Weapon::Sawedoff => "32 x 8 pellets (256 max)".to_string(),
            Weapon::Xm1014 => "20 x 6 pellets (120 max)".to_string(),
            Weapon::Taser => "100 (instant kill)".to_string(),
            Weapon::HeGrenade => "Up to 98".to_string(),
            Weapon::Unknown => "N/A".to_string(),
            Weapon::Flashbang | Weapon::Smoke | Weapon::Decoy => "0".to_string(),
            Weapon::Molotov | Weapon::Incendiary => "40/sec".to_string(),
            w => w.base_damage().to_string(),
        }
    }

    pub fn penetration(&self) -> f32 {
        use Weapon::*;
        match self {
            Awp | G3SG1 | Scar20 => 2.5,
            Ak47 | Aug | Famas | Galilar | M4A4 | M4A1 | Sg556 | Ssg08 | M249 | Negev | Deagle | Revolver => 2.0,
            Cz75A | DualBerettas | FiveSeven | Glock | P2000 | P250 | Tec9 | Usp |
            Bizon | Mac10 | Mp5Sd | Mp7 | Mp9 | P90 | Ump45 |
            Mag7 | Nova | Sawedoff | Xm1014 => 1.0,
            _ => 0.0,
        }
    }
}
