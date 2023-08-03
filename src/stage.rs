use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    block::{Block, List, Variable},
    costume::Costume,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub current_costume: usize,
    pub volume: u8,
    pub variables: HashMap<String, Variable>,
    pub lists: HashMap<String, List>,
    pub blocks: HashMap<String, Block>,
    pub costumes: Vec<Costume>,
    pub tempo: u32,
    pub video_transparency: u8,
    pub video_state: VideoState,
    pub text_to_speech_language: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoState {
    On,
    Off,
    KillingTheRadioStar,
}
