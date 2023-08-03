use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costume {
    pub name: String,
    pub bitmap_resolution: u32,
    pub md5ext: String,
    pub rotation_center_x: i32,
    pub rotation_center_y: i32,
}
