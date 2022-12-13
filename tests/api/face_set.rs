use h3o::{CellIndex, Face};

#[test]
fn len() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("cell");
    let faces = index.icosahedron_faces();

    assert_eq!(faces.len(), 1);
}

#[test]
fn is_empty() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("cell");
    let faces = index.icosahedron_faces();

    assert!(!faces.is_empty());
}

#[test]
fn contains() {
    let index = CellIndex::try_from(0x89283470803ffff).expect("cell");
    let faces = index.icosahedron_faces();

    assert!(faces.contains(Face::try_from(7).expect("face")));
    assert!(!faces.contains(Face::try_from(2).expect("face")));
}

#[test]
fn display() {
    let index = CellIndex::try_from(0x8a1c00000007fff).expect("cell");
    let faces = index.icosahedron_faces();

    assert_eq!(faces.to_string(), "[1-2-6-7-11]".to_owned());
}
