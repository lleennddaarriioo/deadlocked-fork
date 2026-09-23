use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::WeaponClass;

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum Weapon {
    #[default]
    #[serde(alias = "unknown", alias = "Unknown", alias = "none")]
    None,

    // pistols
    #[serde(alias = "c_z75", alias = "cz75", alias = "cz_75")]
    CZ75,
    #[serde(alias = "desert_eagle", alias = "deagle")]
    DesertEagle,
    #[serde(alias = "dual_berettas", alias = "elite", alias = "dualies")]
    DualBerettas,
    #[serde(alias = "five_seven", alias = "fiveseven")]
    FiveSeven,
    #[serde(alias = "glock")]
    Glock,
    #[serde(alias = "p2000", alias = "hkp2000")]
    P2000,
    #[serde(alias = "p250")]
    P250,
    #[serde(alias = "revolver", alias = "r8_revolver", alias = "r8")]
    Revolver,
    #[serde(alias = "tec9", alias = "tec_9")]
    Tec9,
    #[serde(alias = "usp", alias = "usp_silencer", alias = "usp_s")]
    Usp,

    // smg
    #[serde(alias = "m_a_c10", alias = "mac10", alias = "mac_10")]
    MAC10,
    #[serde(alias = "m_p5", alias = "mp5", alias = "mp5sd", alias = "mp5_sd")]
    MP5,
    #[serde(alias = "m_p7", alias = "mp7")]
    MP7,
    #[serde(alias = "m_p9", alias = "mp9")]
    MP9,
    #[serde(alias = "p90", alias = "p_90")]
    P90,
    #[serde(alias = "bizon")]
    Bizon,
    #[serde(alias = "u_m_p45", alias = "ump45", alias = "ump_45")]
    UMP45,

    // shotguns
    #[serde(alias = "mag7", alias = "mag_7")]
    Mag7,
    #[serde(alias = "nova")]
    Nova,
    #[serde(alias = "sawed_off", alias = "sawedoff")]
    SawedOff,
    #[serde(alias = "x_m1014", alias = "xm1014")]
    XM1014,

    // lmg
    #[serde(alias = "m249")]
    M249,
    #[serde(alias = "negev")]
    Negev,

    // assault rifles
    #[serde(alias = "a_k47", alias = "ak47", alias = "ak_47")]
    AK47,
    #[serde(alias = "aug")]
    Aug,
    #[serde(alias = "famas")]
    Famas,
    #[serde(alias = "galil", alias = "galilar", alias = "galil_ar")]
    Galil,
    #[serde(alias = "m4_a1_s", alias = "m4a1s", alias = "m4a1_s", alias = "m4a1_silencer")]
    M4A1S,
    #[serde(alias = "m4_a4", alias = "m4a4")]
    M4A4,
    #[serde(alias = "s_g553", alias = "sg553", alias = "sg556", alias = "sg_553")]
    SG553,

    // snipers
    #[serde(alias = "awp")]
    Awp,
    #[serde(alias = "g3_s_g1", alias = "g3sg1")]
    G3SG1,
    #[serde(alias = "s_c_a_r20", alias = "scar20", alias = "scar_20")]
    SCAR20,
    #[serde(alias = "s_s_g08", alias = "ssg08", alias = "ssg_08")]
    SSG08,

    // knives
    #[serde(alias = "knife_c_t", alias = "knifect", alias = "knife_ct")]
    KnifeCT,
    #[serde(alias = "knife_t", alias = "knifet")]
    KnifeT,
    #[serde(alias = "knife_bayonet", alias = "bayonet")]
    KnifeBayonet,
    #[serde(alias = "knife_bowie", alias = "bowie")]
    KnifeBowie,
    #[serde(alias = "knife_butterfly", alias = "butterfly")]
    KnifeButterfly,
    #[serde(alias = "knife_classic", alias = "classic")]
    KnifeClassic,
    #[serde(alias = "knife_falchion", alias = "falchion")]
    KnifeFalchion,
    #[serde(alias = "knife_flip", alias = "flip")]
    KnifeFlip,
    #[serde(alias = "knife_gut", alias = "gut")]
    KnifeGut,
    #[serde(alias = "knife_huntsman", alias = "huntsman")]
    KnifeHuntsman,
    #[serde(alias = "knife_karambit", alias = "karambit")]
    KnifeKarambit,
    #[serde(alias = "knife_kukri", alias = "kukri")]
    KnifeKukri,
    #[serde(alias = "knife_m9_bayonet", alias = "m9_bayonet")]
    KnifeM9Bayonet,
    #[serde(alias = "knife_navaja", alias = "navaja")]
    KnifeNavaja,
    #[serde(alias = "knife_nomad", alias = "nomad")]
    KnifeNomad,
    #[serde(alias = "knife_paracord", alias = "paracord")]
    KnifeParacord,
    #[serde(alias = "knife_shadow_daggers", alias = "shadow_daggers")]
    KnifeShadowDaggers,
    #[serde(alias = "knife_skeleton", alias = "skeleton")]
    KnifeSkeleton,
    #[serde(alias = "knife_stiletto", alias = "stiletto")]
    KnifeStiletto,
    #[serde(alias = "knife_survival", alias = "survival")]
    KnifeSurvival,
    #[serde(alias = "knife_talon", alias = "talon")]
    KnifeTalon,
    #[serde(alias = "knife_ursus", alias = "ursus")]
    KnifeUrsus,
    #[serde(alias = "knife_gold", alias = "gold")]
    KnifeGold,

    // grenades
    Flashbang,
    HE,
    Smoke,
    Molotov,
    Decoy,
    Incendiary,

    // misc
    Taser,
    C4,
    Healthshot,
}

impl Weapon {
    pub fn from_index(index: u16) -> Self {
        match index {
            1 => Self::DesertEagle,
            2 => Self::DualBerettas,
            3 => Self::FiveSeven,
            4 => Self::Glock,
            7 => Self::AK47,
            8 => Self::Aug,
            9 => Self::Awp,
            10 => Self::Famas,
            11 => Self::G3SG1,
            13 => Self::Galil,
            14 => Self::M249,
            16 => Self::M4A4,
            17 => Self::MAC10,
            19 => Self::P90,
            23 => Self::MP5,
            24 => Self::UMP45,
            25 => Self::XM1014,
            26 => Self::Bizon,
            27 => Self::Mag7,
            28 => Self::Negev,
            29 => Self::SawedOff,
            30 => Self::Tec9,
            31 => Self::Taser,
            32 => Self::P2000,
            33 => Self::MP7,
            34 => Self::MP9,
            35 => Self::Nova,
            36 => Self::P250,
            38 => Self::SCAR20,
            39 => Self::SG553,
            40 => Self::SSG08,
            41 => Self::KnifeGold,
            42 => Self::KnifeCT,
            43 => Self::Flashbang,
            44 => Self::HE,
            45 => Self::Smoke,
            46 => Self::Molotov,
            47 => Self::Decoy,
            48 => Self::Incendiary,
            49 => Self::C4,
            57 => Self::Healthshot,
            59 => Self::KnifeT,
            60 => Self::M4A1S,
            61 => Self::Usp,
            63 => Self::CZ75,
            64 => Self::Revolver,
            503 => Self::KnifeClassic,
            505 => Self::KnifeFlip,
            506 => Self::KnifeGut,
            507 => Self::KnifeKarambit,
            508 => Self::KnifeM9Bayonet,
            509 => Self::KnifeHuntsman,
            512 => Self::KnifeFalchion,
            514 => Self::KnifeBowie,
            515 => Self::KnifeButterfly,
            516 => Self::KnifeShadowDaggers,
            517 => Self::KnifeParacord,
            518 => Self::KnifeSurvival,
            519 => Self::KnifeUrsus,
            520 => Self::KnifeNavaja,
            521 => Self::KnifeNomad,
            522 => Self::KnifeStiletto,
            523 => Self::KnifeTalon,
            524 => Self::KnifeSkeleton,
            _ => Self::None,
        }
    }

    pub fn to_icon(&self) -> char {
        match self {
            Self::None => '?',

            // pistols
            Self::CZ75 => '\u{e02b}',
            Self::DesertEagle => '\u{e04a}',
            Self::DualBerettas => '\u{e056}',
            Self::FiveSeven => '\u{e03a}',
            Self::Glock => '\u{e023}',
            Self::P2000 => '\u{e05a}',
            Self::P250 => '\u{e04f}',
            Self::Revolver => '\u{e003}',
            Self::Tec9 => '\u{e00e}',
            Self::Usp => '\u{e037}',

            // smg
            Self::MAC10 => '\u{e049}',
            Self::MP5 => '\u{e039}',
            Self::MP7 => '\u{e047}',
            Self::MP9 => '\u{e040}',
            Self::P90 => '\u{e035}',
            Self::Bizon => '\u{e00c}',
            Self::UMP45 => '\u{e01f}',

            // shotguns
            Self::Mag7 => '\u{e032}',
            Self::Nova => '\u{e02d}',
            Self::SawedOff => '\u{e031}',
            Self::XM1014 => '\u{e01d}',

            // lmg
            Self::M249 => '\u{e055}',
            Self::Negev => '\u{e019}',

            // assault rifles
            Self::AK47 => '\u{e008}',
            Self::Aug => '\u{e060}',
            Self::Famas => '\u{e025}',
            Self::Galil => '\u{e000}',
            Self::M4A1S => '\u{e015}',
            Self::M4A4 => '\u{e042}',
            Self::SG553 => '\u{e00d}',

            // snipers
            Self::Awp => '\u{e007}',
            Self::G3SG1 => '\u{e04b}',
            Self::SCAR20 => '\u{e009}',
            Self::SSG08 => '\u{e03e}',

            // knives
            Self::KnifeCT => '\u{e050}',
            Self::KnifeT => '\u{e05b}',
            Self::KnifeBayonet => '\u{e014}',
            Self::KnifeBowie => '\u{e052}',
            Self::KnifeButterfly => '\u{e026}',
            Self::KnifeClassic => '\u{e04c}',
            Self::KnifeFalchion => '\u{e013}',
            Self::KnifeFlip => '\u{e01c}',
            Self::KnifeGut => '\u{e041}',
            Self::KnifeHuntsman => '\u{e010}',
            Self::KnifeKarambit => '\u{e048}',
            Self::KnifeKukri => '\u{e02f}',
            Self::KnifeM9Bayonet => '\u{e030}',
            Self::KnifeNavaja => '\u{e011}',
            Self::KnifeNomad => '\u{e058}',
            Self::KnifeParacord => '\u{e05d}',
            Self::KnifeShadowDaggers => '\u{e046}',
            Self::KnifeSkeleton => '\u{e005}',
            Self::KnifeStiletto => '\u{e054}',
            Self::KnifeSurvival => '\u{e002}',
            Self::KnifeTalon => '\u{e004}',
            Self::KnifeUrsus => '\u{e03c}',
            Self::KnifeGold => '\u{e044}',
            // grenades
            Self::Flashbang => '\u{e05e}',
            Self::HE => '\u{e00b}',
            Self::Smoke => '\u{e02e}',
            Self::Molotov => '\u{e024}',
            Self::Decoy => '\u{e028}',
            Self::Incendiary => '\u{e01a}',

            // misc
            Self::Taser => '\u{e021}',
            Self::C4 => '\u{e01e}',
            Self::Healthshot => '\u{e01b}',
        }
    }

    pub fn weapon_class(&self) -> WeaponClass {
        match self {
            Self::None => WeaponClass::Unknown,

            // pistols
            Self::CZ75 => WeaponClass::Pistol,
            Self::DesertEagle => WeaponClass::Pistol,
            Self::DualBerettas => WeaponClass::Pistol,
            Self::FiveSeven => WeaponClass::Pistol,
            Self::Glock => WeaponClass::Pistol,
            Self::P2000 => WeaponClass::Pistol,
            Self::P250 => WeaponClass::Pistol,
            Self::Revolver => WeaponClass::Pistol,
            Self::Tec9 => WeaponClass::Pistol,
            Self::Usp => WeaponClass::Pistol,

            // smg
            Self::MAC10 => WeaponClass::Smg,
            Self::MP5 => WeaponClass::Smg,
            Self::MP7 => WeaponClass::Smg,
            Self::MP9 => WeaponClass::Smg,
            Self::P90 => WeaponClass::Smg,
            Self::Bizon => WeaponClass::Smg,
            Self::UMP45 => WeaponClass::Smg,

            // shotguns
            Self::Mag7 => WeaponClass::Shotgun,
            Self::Nova => WeaponClass::Shotgun,
            Self::SawedOff => WeaponClass::Shotgun,
            Self::XM1014 => WeaponClass::Shotgun,

            // lmg
            Self::M249 => WeaponClass::Heavy,
            Self::Negev => WeaponClass::Heavy,

            // assault rifles
            Self::AK47 => WeaponClass::Rifle,
            Self::Aug => WeaponClass::Rifle,
            Self::Famas => WeaponClass::Rifle,
            Self::Galil => WeaponClass::Rifle,
            Self::M4A1S => WeaponClass::Rifle,
            Self::M4A4 => WeaponClass::Rifle,
            Self::SG553 => WeaponClass::Rifle,

            // snipers
            Self::Awp => WeaponClass::Sniper,
            Self::G3SG1 => WeaponClass::Sniper,
            Self::SCAR20 => WeaponClass::Sniper,
            Self::SSG08 => WeaponClass::Sniper,

            // knives
            Self::KnifeCT => WeaponClass::Knife,
            Self::KnifeT => WeaponClass::Knife,
            Self::KnifeBayonet => WeaponClass::Knife,
            Self::KnifeBowie => WeaponClass::Knife,
            Self::KnifeButterfly => WeaponClass::Knife,
            Self::KnifeClassic => WeaponClass::Knife,
            Self::KnifeFalchion => WeaponClass::Knife,
            Self::KnifeFlip => WeaponClass::Knife,
            Self::KnifeGut => WeaponClass::Knife,
            Self::KnifeHuntsman => WeaponClass::Knife,
            Self::KnifeKarambit => WeaponClass::Knife,
            Self::KnifeKukri => WeaponClass::Knife,
            Self::KnifeM9Bayonet => WeaponClass::Knife,
            Self::KnifeNavaja => WeaponClass::Knife,
            Self::KnifeNomad => WeaponClass::Knife,
            Self::KnifeParacord => WeaponClass::Knife,
            Self::KnifeShadowDaggers => WeaponClass::Knife,
            Self::KnifeSkeleton => WeaponClass::Knife,
            Self::KnifeStiletto => WeaponClass::Knife,
            Self::KnifeSurvival => WeaponClass::Knife,
            Self::KnifeTalon => WeaponClass::Knife,
            Self::KnifeUrsus => WeaponClass::Knife,
            Self::KnifeGold => WeaponClass::Knife,
            // grenades
            Self::Flashbang => WeaponClass::Grenade,
            Self::HE => WeaponClass::Grenade,
            Self::Smoke => WeaponClass::Grenade,
            Self::Molotov => WeaponClass::Grenade,
            Self::Decoy => WeaponClass::Grenade,
            Self::Incendiary => WeaponClass::Grenade,

            // misc
            Self::Taser => WeaponClass::Utility,
            Self::C4 => WeaponClass::Utility,
            Self::Healthshot => WeaponClass::Utility,
        }
    }
}

impl std::fmt::Display for Weapon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => "?",

            // pistols
            Self::CZ75 => "CZ75-Auto",
            Self::DesertEagle => "Desert Eagle",
            Self::DualBerettas => "Dual Berettas",
            Self::FiveSeven => "Five-SeveN",
            Self::Glock => "Glock-18",
            Self::P2000 => "P2000",
            Self::P250 => "P250",
            Self::Revolver => "R8 Revolver",
            Self::Tec9 => "Tec-9",
            Self::Usp => "USP-S",

            // smg
            Self::MAC10 => "MAC-10",
            Self::MP5 => "MP5-SD",
            Self::MP7 => "MP7",
            Self::MP9 => "MP9",
            Self::P90 => "P90",
            Self::Bizon => "PP-Bizon",
            Self::UMP45 => "UMP-45",

            // shotguns
            Self::Mag7 => "Mag-7",
            Self::Nova => "Nova",
            Self::SawedOff => "Sawed-Off",
            Self::XM1014 => "XM1014",

            // lmg
            Self::M249 => "M249",
            Self::Negev => "Negev",

            // assault rifles
            Self::AK47 => "AK-47",
            Self::Aug => "AUG",
            Self::Famas => "FAMAS",
            Self::Galil => "Galil AR",
            Self::M4A1S => "M4A1-S",
            Self::M4A4 => "M4A4",
            Self::SG553 => "SG 553",

            // snipers
            Self::Awp => "AWP",
            Self::G3SG1 => "G3SG1",
            Self::SCAR20 => "SCAR-20",
            Self::SSG08 => "SSG 08",

            // knives
            Self::KnifeCT => "Knife",
            Self::KnifeT => "Knife",
            Self::KnifeBayonet => "Bayonet",
            Self::KnifeBowie => "Bowie Knife",
            Self::KnifeButterfly => "Butterfly Knife",
            Self::KnifeClassic => "Classic Knife",
            Self::KnifeFalchion => "Falchion Knife",
            Self::KnifeFlip => "Flip Knife",
            Self::KnifeGut => "Gut Knife",
            Self::KnifeHuntsman => "Huntsman Knife",
            Self::KnifeKarambit => "Karambit",
            Self::KnifeKukri => "Kukri Knife",
            Self::KnifeM9Bayonet => "M9 Bayonet",
            Self::KnifeNavaja => "Navaja Knife",
            Self::KnifeNomad => "Nomad Knife",
            Self::KnifeParacord => "Paracord Knife",
            Self::KnifeShadowDaggers => "Shadow Daggers",
            Self::KnifeSkeleton => "Skeleton Knife",
            Self::KnifeStiletto => "Stiletto Knife",
            Self::KnifeSurvival => "Survival Knife",
            Self::KnifeTalon => "Talon Knife",
            Self::KnifeUrsus => "Ursus Knife",
            Self::KnifeGold => "Gold Knife",
            // grenades
            Self::Flashbang => "Flashbang",
            Self::HE => "HE Grenade",
            Self::Smoke => "Smoke Grenade",
            Self::Molotov => "Molotov",
            Self::Decoy => "Decoy Grenade",
            Self::Incendiary => "Incendiary Grenade",

            // misc
            Self::Taser => "Zeus x27",
            Self::C4 => "C4",
            Self::Healthshot => "Healthshot",
        }
        .fmt(f)
    }
}

impl Weapon {
    pub fn base_damage(&self) -> u32 {
        match self {
            Weapon::KnifeCT | Weapon::KnifeT => 40,
            Weapon::CZ75 => 31,
            Weapon::DesertEagle => 53,
            Weapon::DualBerettas => 38,
            Weapon::FiveSeven => 32,
            Weapon::Glock => 28,
            Weapon::P2000 => 26,
            Weapon::P250 => 38,
            Weapon::Revolver => 86,
            Weapon::Tec9 => 33,
            Weapon::Usp => 26,
            Weapon::Bizon => 26,
            Weapon::MAC10 => 29,
            Weapon::MP5 => 27,
            Weapon::MP7 => 29,
            Weapon::MP9 => 26,
            Weapon::P90 => 26,
            Weapon::UMP45 => 35,
            Weapon::M249 => 32,
            Weapon::Negev => 26,
            Weapon::Mag7 => 30,
            Weapon::Nova => 26,
            Weapon::SawedOff => 32,
            Weapon::XM1014 => 20,
            Weapon::AK47 => 36,
            Weapon::Aug => 28,
            Weapon::Famas => 30,
            Weapon::Galil => 30,
            Weapon::M4A4 => 33,
            Weapon::M4A1S => 38,
            Weapon::SG553 => 30,
            Weapon::Awp => 115,
            Weapon::G3SG1 => 80,
            Weapon::SCAR20 => 80,
            Weapon::SSG08 => 88,
            Weapon::Taser => 100,
            Weapon::HE => 98,
            _ => 0,
        }
    }

    pub fn damage_description(&self) -> String {
        match self {
            Weapon::KnifeCT | Weapon::KnifeT => "40 (stab: 55, backstab: 90)".to_string(),
            Weapon::Mag7 => "30 x 8 pellets (240 max)".to_string(),
            Weapon::Nova => "26 x 9 pellets (234 max)".to_string(),
            Weapon::SawedOff => "32 x 8 pellets (256 max)".to_string(),
            Weapon::XM1014 => "20 x 6 pellets (120 max)".to_string(),
            Weapon::Taser => "100 (instant kill)".to_string(),
            Weapon::HE => "Up to 98".to_string(),
            Weapon::Flashbang | Weapon::Smoke | Weapon::Decoy => "0".to_string(),
            Weapon::Molotov | Weapon::Incendiary => "40/sec".to_string(),
            w => w.base_damage().to_string(),
        }
    }

    pub fn penetration(&self) -> f32 {
        use Weapon::*;
        match self {
            Awp | G3SG1 | SCAR20 => 2.5,
            AK47 | Aug | Famas | Galil | M4A4 | M4A1S | SG553 | SSG08 | M249 | Negev | DesertEagle | Revolver => 2.0,
            CZ75 | DualBerettas | FiveSeven | Glock | P2000 | P250 | Tec9 | Usp |
            Bizon | MAC10 | MP5 | MP7 | MP9 | P90 | UMP45 |
            Mag7 | Nova | SawedOff | XM1014 => 1.0,
            _ => 0.0,
        }
    }
}
