use crate::hgt::HgtFile;
use geom::{Distance, LonLat};

pub struct Elevation {
    hgt: HgtFile,
}

impl Elevation {
    pub fn load(path: &str) -> Result<Elevation, std::io::Error> {
        println!("Reading elevation data from {}", path);

        let hgt = HgtFile::from_path(47.0, -123.0, crate::hgt::HgtResolution::One, path)?;
        Ok(Elevation { hgt })
    }

    pub fn get(&self, pt: LonLat) -> Distance {
        if let Some(e) = self.hgt.interpolate(pt.latitude, pt.longitude) {
            Distance::meters(e)
        } else {
            println!("Can't get elevation at {}!", pt);
            Distance::ZERO
        }
    }
}
