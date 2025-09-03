use bincode::{enc::write, Decode, Encode};
use pcd_rs::{PcdDeserialize, PcdSerialize, Reader, WriterInit};
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, PcdDeserialize, PcdSerialize, Serialize, Deserialize, Encode, Decode)]
#[repr(C)]
pub struct PointXYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn load_pcd(path: &str) -> Result<Vec<PointXYZ>, Box<dyn std::error::Error>> {
    let reader = match Reader::open(path) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Failed to load the pcd!: {}", e);
            return Err(e.into());
        }
    };

    let points: Vec<PointXYZ> = match reader.collect() {
        Ok(points) => points,
        Err(e) => {
            eprintln!("Failed to read the pcd: {}", e);
            return Err(e.into());
        }
    };

    Ok(points)
}

pub fn save_pcd(path: &str, points: &[PointXYZ]) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = WriterInit {
        width: 1,
        height: points.len() as u64,
        viewpoint: Default::default(),
        data_kind: pcd_rs::DataKind::Ascii,
        schema: None,
    }
    .create(path)?;

    for p in points {
        writer.push(p)?;
    }

    writer.finish()?;
    Ok(())
}