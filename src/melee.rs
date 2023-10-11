use std::fmt::Display;

use num_enum::TryFromPrimitive;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::melee::{stage::MeleeStage, character::MeleeCharacter};

use self::{dolphin_mem::{DolphinMemory, util::R13}, msrb::MSRBOffset, multiman::MultiManVariant};

mod dolphin_mem;
mod msrb;
mod multiman;
pub mod stage;
pub mod character;

// reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L11-L14
#[derive(PartialEq, EnumIter, Clone, Copy)]
enum TimerMode {
    Countup = 3,
    Countdown = 2,
    Hidden = 1,
    Frozen = 0,
}

#[derive(TryFromPrimitive, Display, Debug)]
#[repr(u8)]
enum MatchmakingMode {
    Idle = 0,
    Initializing = 1,
    Matchmaking = 2,
    OpponentConnecting = 3,
    ConnectionSuccess = 4,
    ErrorEncountered = 5
}

#[derive(Debug, TryFromPrimitive, Display, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum SlippiMenuScene {
    Ranked = 0,
    Unranked = 1,
    Direct = 2,
    Teams = 3
}

pub struct MeleeClient {
    mem: DolphinMemory,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MeleeScene {
    VsMode,
    UnclePunch,
    TrainingMode,
    SlippiOnline(Option<SlippiMenuScene>),
    SlippiCss(Option<SlippiMenuScene>),
    HomeRunContest,
    TargetTest(Option<MeleeStage>),
    MultiManMelee(MultiManVariant)
}

impl Display for MeleeScene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::VsMode => write!(f, "Vs. Mode"),
            Self::UnclePunch => write!(f, "UnclePunch Training Mode"),
            Self::TrainingMode => write!(f, "Training Mode"),
            Self::SlippiOnline(Some(scene)) => write!(f, "{}", scene),
            Self::SlippiOnline(None) => write!(f, "Slippi Online"),
            Self::HomeRunContest => write!(f, "Home-Run Contest"),
            Self::TargetTest(stage_opt) => {
                if stage_opt.is_some() {
                    write!(f, "{}", stage_opt.unwrap())
                } else {
                    write!(f, "Target Test")
                }
            },
            Self::MultiManMelee(variant) => write!(f, "Multi-Man Melee ({})", match variant {
                MultiManVariant::TenMan => "10 man",
                MultiManVariant::HundredMan => "100 man",
                MultiManVariant::ThreeMinute => "3 min",
                MultiManVariant::FifteenMinute => "15 min",
                MultiManVariant::Endless => "Endless",
                MultiManVariant::Cruel => "Cruel",
            }),
            Self::SlippiCss(_) => unimplemented!(),
        }
    }
}


impl MeleeClient {
    pub fn new() -> Self {
        MeleeClient { mem: DolphinMemory::new() }
    }

    fn get_player_port(&mut self) -> Option<u8> { self.mem.read::<u8>(R13!(0x5108)) }
    fn get_slippi_player_port(&mut self) -> Option<u8> { self.mem.read_msrb(MSRBOffset::MsrbLocalPlayerIndex) }
    fn get_opp_name(&mut self) -> Option<String> { self.mem.read_msrb_string::<31>(MSRBOffset::MsrbOppName) }
    fn get_player_connect_code(&mut self, port: u8) -> Option<String> {
        const PLAYER_CONNECTCODE_OFFSETS: [MSRBOffset; 4] = [MSRBOffset::MsrbP1ConnectCode, MSRBOffset::MsrbP2ConnectCode, MSRBOffset::MsrbP3ConnectCode, MSRBOffset::MsrbP4ConnectCode];
        self.mem.read_msrb_string_shift_jis::<10>(PLAYER_CONNECTCODE_OFFSETS[port as usize])
    }
    fn get_character_selection(&mut self, port: u8) -> Option<MeleeCharacter> {
        // 0x04 = character, 0x05 = skin (reference: https://github.com/bkacjios/m-overlay/blob/master/source/modules/games/GALE01-2.lua#L199-L202)
        const PLAYER_SELECTION_BLOCKS: [u32; 4] = [0x8043208B, 0x80432093, 0x8043209B, 0x804320A3];
        self.mem.read::<u8>(PLAYER_SELECTION_BLOCKS[port as usize] + 0x04).and_then(|v| MeleeCharacter::try_from(v).ok())
    }
    fn timer_mode(&mut self) -> TimerMode {
        const MATCH_INIT: u32 = 0x8046DB68; // first byte, reference: https://github.com/akaneia/m-ex/blob/master/MexTK/include/match.h#L136
        self.mem.read::<u8>(MATCH_INIT).and_then(|v| {
            for timer_mode in TimerMode::iter() {
                let val = timer_mode as u8;
                if v & val == val {
                    return Some(timer_mode);
                }
            }
            None
        }).unwrap_or(TimerMode::Countup)
    }
    fn game_time(&mut self) -> i64 { self.mem.read::<u32>(0x8046B6C8).and_then(|v| Some(v)).unwrap_or(0) as i64 }
    fn matchmaking_type(&mut self) -> Option<MatchmakingMode> {
        self.mem.read_msrb::<u8>(MSRBOffset::MsrbConnectionState).and_then(|v| MatchmakingMode::try_from(v).ok())
    }
    fn slippi_online_scene(&mut self) -> Option<SlippiMenuScene> { self.mem.read::<u8>(R13!(0x5060)).and_then(|v| SlippiMenuScene::try_from(v).ok()) }
    /*fn game_variant(&mut self) -> Option<MeleeGameVariant> {
        const GAME_ID_ADDR: u32 = 0x80000000;
        const GAME_ID_LEN: usize = 0x06;

        let game_id = self.mem.read_string::<GAME_ID_LEN>(GAME_ID_ADDR);
        if game_id.is_none() {
            return None;
        }
        return match game_id.unwrap().as_str() {
            "GALE01" => Some(MeleeGameVariant::Vanilla),
            "GTME01" => Some(MeleeGameVariant::UnclePunch),
            _ => None
        }
    }*/
    fn get_melee_scene(&mut self) -> Option<MeleeScene> {
        const MAJOR_SCENE: u32 = 0x80479D30;
        const MINOR_SCENE: u32 = MAJOR_SCENE + 0x03;
        let scene_tuple = (self.mem.read::<u8>(MAJOR_SCENE).unwrap_or(0), self.mem.read::<u8>(MINOR_SCENE).unwrap_or(0));

        match scene_tuple {
            (2, 2) => Some(MeleeScene::VsMode),
            (43, 1) => Some(MeleeScene::UnclePunch),
            (28, 2) => Some(MeleeScene::TrainingMode),
            (8, 2) => Some(MeleeScene::SlippiOnline(self.slippi_online_scene())),
            (8, 0) => Some(MeleeScene::SlippiCss(self.slippi_online_scene())),
            (32, 1) => Some(MeleeScene::HomeRunContest),
            (15, 1) => Some(MeleeScene::TargetTest(self.get_stage())),
            (33, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::TenMan)),
            (34, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::HundredMan)),
            (35, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::ThreeMinute)),
            (36, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::FifteenMinute)),
            (37, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::Endless)),
            (38, 1) => Some(MeleeScene::MultiManMelee(MultiManVariant::Cruel)),
            _ => None
        }
    }
    fn get_stage(&mut self) -> Option<MeleeStage> {
        self.mem.read::<u8>(0x8049E6C8 + 0x88 + 0x03).and_then(|v| MeleeStage::try_from(v).ok())
    }
    fn get_character(&mut self, player_id: u8) -> Option<MeleeCharacter> {
        const PLAYER_BLOCKS: [u32; 4] = [0x80453080, 0x80453F10, 0x80454DA0, 0x80455C30];
        self.mem.read::<u8>(PLAYER_BLOCKS[player_id as usize] + 0x07).and_then(|v| MeleeCharacter::try_from(v).ok())
    }
}