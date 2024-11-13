use geo::{coord, line_string, Line};
use h3o::{geom::PlotterBuilder, Resolution};

fn line_rads() -> Line {
    Line::new(
        coord! { x: -0.009526982062241713, y: 0.8285232894553574 },
        coord! { x: 0.04142734140306332, y: 0.8525145186317127 },
    )
}

fn line_degs() -> Line {
    Line::new(
        coord! { x: -0.5458558636632915, y: 47.47088771408784 },
        coord! { x: 2.373611818843102,   y: 48.84548389122412 },
    )
}

#[test]
fn add_rads() {
    let mut plotter = PlotterBuilder::new(Resolution::Two)
        .disable_radians_conversion()
        .build();
    let result = plotter.add(line_rads());

    assert!(result.is_ok());
}

#[test]
fn add_degs() {
    let mut plotter = PlotterBuilder::new(Resolution::Two).build();
    let result = plotter.add(line_degs());

    assert!(result.is_ok());
}

#[test]
fn add_batch() {
    let mut plotter = PlotterBuilder::new(Resolution::Two).build();
    let result = plotter.add_batch(
        line_string![
            (x: 2.363503198417334,  y: 48.8203086545891),
            (x: 2.3730684893043588, y: 48.85398407690437),
            (x: 2.334964762310932,  y: 48.870861968772914),
        ]
        .lines(),
    );

    assert!(result.is_ok());
}

#[test]
fn invalid() {
    let mut plotter = PlotterBuilder::new(Resolution::Two).build();
    let result = plotter.add(Line::new(
        coord! { x: 0., y: 0. },
        coord! { x: f64::NAN, y: 0. },
    ));

    assert!(result.is_err());
}

#[test]
fn plot() {
    let mut plotter = PlotterBuilder::new(Resolution::Ten).build();
    plotter.add(line_degs()).expect("failed to add line");

    let result = plotter
        .plot()
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to plot")
        .len();

    assert_eq!(result, 2423);
}
