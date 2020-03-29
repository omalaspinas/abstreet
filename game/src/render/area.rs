use crate::app::App;
use crate::helpers::{ColorScheme, ID};
use crate::render::{DrawOptions, Renderable};
use ezgui::{Color, GeomBatch, GfxCtx};
use geom::Polygon;
use map_model::{Area, AreaID, AreaType, Map};

pub struct DrawArea {
    pub id: AreaID,
}

impl DrawArea {
    pub fn new(area: &Area, cs: &ColorScheme, all_areas: &mut GeomBatch) -> DrawArea {
        let color = match area.area_type {
            AreaType::Park => cs.get_def("grass", Color::hex("#84BA3B")), //#2F8C2C")),
            AreaType::Water => cs.get_def("water", Color::rgb(164, 200, 234)),
            AreaType::PedestrianIsland => Color::grey(0.3),
        };
        all_areas.push(color, area.polygon.clone());
        DrawArea { id: area.id }
    }
}

impl Renderable for DrawArea {
    fn get_id(&self) -> ID {
        ID::Area(self.id)
    }

    fn draw(&self, _: &mut GfxCtx, _: &App, _: &DrawOptions) {}

    fn get_outline(&self, map: &Map) -> Polygon {
        // Since areas are so big, don't just draw the outline
        map.get_a(self.id).polygon.clone()
    }
}
