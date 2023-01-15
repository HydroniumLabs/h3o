use super::h3api;
use float_eq::assert_float_eq;
use h3o::{CellIndex, LatLng, Resolution, Vertex};

macro_rules! exhaustive_test {
    ($name:ident, $resolution:literal) => {
        #[test]
        fn $name() {
            let resolution =
                Resolution::try_from($resolution).expect("index resolution");
            for index in CellIndex::base_cells()
                .flat_map(|index| index.children(resolution))
            {
                for vertex in 0..6 {
                    let vertex = Vertex::try_from(vertex).expect("cell vertex");
                    if let Some(index) = index.vertex(vertex) {
                        let result = LatLng::from(index);
                        let reference = h3api::vertex_to_latlng(index);

                        assert_float_eq!(
                            result.lat_radians(),
                            reference.lat_radians(),
                            abs <= f64::from(f32::EPSILON),
                            "latitude (vertex {index})"
                        );
                        assert_float_eq!(
                            result.lng_radians(),
                            reference.lng_radians(),
                            abs <= f64::from(f32::EPSILON),
                            "longitude (vertex {index})"
                        );
                    }
                }
            }
        }
    };
}

exhaustive_test!(exhaustive_res0, 0);
exhaustive_test!(exhaustive_res1, 1);
exhaustive_test!(exhaustive_res2, 2);
