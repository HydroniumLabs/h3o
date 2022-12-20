use super::*;
use geo::{line_string, polygon};

#[test]
fn test_simple() {
    // ┏━┓
    // ┗━┛
    #[rustfmt::skip]
        let rings = vec![
            line_string![(x: 1., y: 1.), (x: 1., y: 3.), (x: 3., y: 3.), (x: 3., y: 1.), (x: 1., y: 1.)],
        ];
    // Expect a single polygon without hole.
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![
            polygon![(x: 1., y: 1.), (x: 1., y: 3.), (x: 3., y: 3.), (x: 3., y: 1.), (x: 1., y: 1.)],
        ]);

    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}

#[test]
fn test_with_one_hole() {
    // ┏━━━┓
    // ┃┏━┓┃
    // ┃┗━┛┃
    // ┗━━━┛
    #[rustfmt::skip]
        let rings = vec![
            line_string![(x: 2., y: 2.), (x: 2., y: 4.), (x: 4., y: 4.), (x: 4., y: 2.), (x: 2., y: 2.)],
            line_string![(x: 1., y: 1.), (x: 1., y: 5.), (x: 5., y: 5.), (x: 5., y: 1.), (x: 1., y: 1.)],
        ];
    // Expect a single polygon with a hole.
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![polygon![
            exterior:   [(x: 1., y: 1.), (x: 1., y: 5.), (x: 5., y: 5.), (x: 5., y: 1.), (x: 1., y: 1.)],
            interiors: [[(x: 2., y: 2.), (x: 2., y: 4.), (x: 4., y: 4.), (x: 4., y: 2.), (x: 2., y: 2.)]]
        ]]);

    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}

#[test]
fn test_with_multiple_holes() {
    // ┏━━━━━━━┓
    // ┃┏━┓ ┏━┓┃
    // ┃┗━┛ ┗━┛┃
    // ┗━━━━━━━┛
    #[rustfmt::skip]
        let rings = vec![
            line_string![(x: 3., y: 4.), (x: 3., y: 6.), (x: 4., y: 6.), (x: 4., y: 4.), (x: 3., y: 4.)],
            line_string![(x: 1., y: 3.), (x: 1., y: 7.), (x: 9., y: 7.), (x: 9., y: 3.), (x: 1., y: 3.)],
            line_string![(x: 6., y: 4.), (x: 6., y: 6.), (x: 7., y: 6.), (x: 7., y: 4.), (x: 6., y: 4.)],
        ];
    // Expect a single polygon with two holes.
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![polygon![
            exterior: [(x: 1., y: 3.), (x: 1., y: 7.), (x: 9., y: 7.), (x: 9., y: 3.), (x: 1., y: 3.)],
            interiors: [
                [(x: 3., y: 4.), (x: 3., y: 6.), (x: 4., y: 6.), (x: 4., y: 4.), (x: 3., y: 4.)],
                [(x: 6., y: 4.), (x: 6., y: 6.), (x: 7., y: 6.), (x: 7., y: 4.), (x: 6., y: 4.)],
            ]
        ]]);
    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}

#[test]
fn test_with_multiple_outers() {
    // ┏━┓ ┏━┓
    // ┗━┛ ┗━┛
    #[rustfmt::skip]
        let rings = vec![
            line_string![(x: 1., y: 2.), (x: 1., y: 4.), (x: 2., y: 4.), (x: 2., y: 2.), (x: 1., y: 2.)],
            line_string![(x: 4., y: 2.), (x: 4., y: 4.), (x: 5., y: 4.), (x: 5., y: 2.), (x: 4., y: 2.)],
        ];
    // Expect a two polygons without holes.
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![
            polygon![(x: 1., y: 2.), (x: 1., y: 4.), (x: 2., y: 4.), (x: 2., y: 2.), (x: 1., y: 2.)],
            polygon![(x: 4., y: 2.), (x: 4., y: 4.), (x: 5., y: 4.), (x: 5., y: 2.), (x: 4., y: 2.)],
        ]);
    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}

#[test]
fn test_nested() {
    // ┏━━━━━┓
    // ┃┏━━━┓┃
    // ┃┃┏━┓┃┃
    // ┃┃┗━┛┃┃
    // ┃┗━━━┛┃
    // ┗━━━━━┛
    #[rustfmt::skip]
        let rings = vec![
            line_string![(x: 3., y: 3.), (x: 3., y: 5.), (x: 5., y: 5.), (x: 5., y: 3.), (x: 3., y: 3.)],
            line_string![(x: 2., y: 2.), (x: 2., y: 6.), (x: 6., y: 6.), (x: 6., y: 2.), (x: 2., y: 2.)],
            line_string![(x: 1., y: 1.), (x: 1., y: 7.), (x: 7., y: 7.), (x: 7., y: 1.), (x: 1., y: 1.)],
        ];
    // Expect a polygon with a hole (outermost) + a polygon without hole
    // (innermost one)
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![
            polygon![
                exterior:   [(x: 1., y: 1.), (x: 1., y: 7.), (x: 7., y: 7.), (x: 7., y: 1.), (x: 1., y: 1.)],
                interiors: [[(x: 2., y: 2.), (x: 2., y: 6.), (x: 6., y: 6.), (x: 6., y: 2.), (x: 2., y: 2.)]]
            ],
            polygon![(x: 3., y: 3.), (x: 3., y: 5.), (x: 5., y: 5.), (x: 5., y: 3.), (x: 3., y: 3.)],
        ]);

    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}

#[test]
fn test_gamut() {
    // ┏━━━━━━━┓
    // ┃┏━━━━━┓┃  ┏━━━━━━━┓
    // ┃┃┏━━━┓┃┃  ┃┏━┓ ┏━┓┃
    // ┃┃┃┏━┓┃┃┃  ┃┗━┛ ┗━┛┃
    // ┃┃┃┗━┛┃┃┃  ┗━━━━━━━┛
    // ┃┃┗━━━┛┃┃
    // ┃┗━━━━━┛┃     ┏━┓
    // ┗━━━━━━━┛     ┗━┛
    #[rustfmt::skip]
        let rings = vec![
            // Nested.
            line_string![(x: 1., y: 1.), (x: 1., y: 9.), (x: 9., y: 9.), (x: 9., y: 1.), (x: 1., y: 1.)],
            line_string![(x: 2., y: 2.), (x: 2., y: 8.), (x: 8., y: 8.), (x: 8., y: 2.), (x: 2., y: 2.)],
            line_string![(x: 3., y: 3.), (x: 3., y: 7.), (x: 7., y: 7.), (x: 7., y: 3.), (x: 3., y: 3.)],
            line_string![(x: 4., y: 4.), (x: 4., y: 6.), (x: 6., y: 6.), (x: 6., y: 4.), (x: 4., y: 4.)],
            // Multiple holes.
            line_string![(x: 13., y: 14.), (x: 13., y: 16.), (x: 14., y: 16.), (x: 14., y: 14.), (x: 13., y: 14.)],
            line_string![(x: 11., y: 13.), (x: 11., y: 17.), (x: 19., y: 17.), (x: 19., y: 13.), (x: 11., y: 13.)],
            line_string![(x: 16., y: 14.), (x: 16., y: 16.), (x: 17., y: 16.), (x: 17., y: 14.), (x: 16., y: 14.)],
            // Simple
            line_string![(x: 19., y: 19.), (x: 19., y: 21.), (x: 21., y: 21.), (x: 21., y: 19.), (x: 19., y: 19.)],
        ];
    #[rustfmt::skip]
        let expected = MultiPolygon(vec![
            polygon![
                exterior:   [(x: 1., y: 1.), (x: 1., y: 9.), (x: 9., y: 9.), (x: 9., y: 1.), (x: 1., y: 1.)],
                interiors: [[(x: 2., y: 2.), (x: 2., y: 8.), (x: 8., y: 8.), (x: 8., y: 2.), (x: 2., y: 2.)]]
            ],
            polygon![
                exterior: [(x: 11., y: 13.), (x: 11., y: 17.), (x: 19., y: 17.), (x: 19., y: 13.), (x: 11., y: 13.)],
                interiors: [
                    [(x: 13., y: 14.), (x: 13., y: 16.), (x: 14., y: 16.), (x: 14., y: 14.), (x: 13., y: 14.)],
                    [(x: 16., y: 14.), (x: 16., y: 16.), (x: 17., y: 16.), (x: 17., y: 14.), (x: 16., y: 14.)],
                ]
            ],
            polygon![(x: 19., y: 19.), (x: 19., y: 21.), (x: 21., y: 21.), (x: 21., y: 19.), (x: 19., y: 19.)],
            polygon![
                exterior:   [(x: 3., y: 3.), (x: 3., y: 7.), (x: 7., y: 7.), (x: 7., y: 3.), (x: 3., y: 3.)],
                interiors: [[(x: 4., y: 4.), (x: 4., y: 6.), (x: 6., y: 6.), (x: 6., y: 4.), (x: 4., y: 4.)]]
            ],
        ]);

    let res = MultiPolygon::from(RingHierarchy::new(rings));

    assert_eq!(res, expected);
}
