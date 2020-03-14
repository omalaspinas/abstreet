use abstutil::CmdArgs;
use geom::GPSBounds;

fn main() {
    let mut args = CmdArgs::new();
    let input = args.required("--input");
    let output = args.required("--output");
    args.done();

    let mut shapes = kml::load(
        &input,
        &GPSBounds::seattle_bounds(),
        &mut abstutil::Timer::new("extracting shapes from KML"),
    )
    .unwrap();

    // TODO Bit of a hack to do filtering in here...
    if input == "../data/input/collisions.kml" {
        shapes.shapes.retain(|es| {
            es.attributes.get("PEDCOUNT") != Some(&"0".to_string())
                || es.attributes.get("PEDCYLCOUNT") != Some(&"0".to_string())
        });
    }

    abstutil::write_binary(output, &shapes);
}
