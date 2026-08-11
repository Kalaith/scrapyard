use super::*;

#[test]
fn all_variants_load_valid_geometry() {
    let base = ShipInterior::for_variant(0);
    assert!(!base.rooms.is_empty(), "base layout should have rooms");

    for variant in 0..ShipInterior::VARIANT_COUNT {
        let ship = ShipInterior::for_variant(variant);
        // Mirroring must preserve the room set and module wiring.
        assert_eq!(ship.rooms.len(), base.rooms.len());
        for room in &ship.rooms {
            assert!(room.x >= -0.5 && room.x + room.width <= ship.width + 0.5);
            assert!(room.y >= -0.5 && room.y + room.height <= ship.height + 0.5);
            // Repair points stay inside their room after mirroring.
            for rp in &room.repair_points {
                assert!(rp.x >= 0.0 && rp.x <= room.width);
                assert!(rp.y >= 0.0 && rp.y <= room.height);
            }
        }
    }
}

#[test]
fn variants_actually_differ() {
    let base = ShipInterior::for_variant(0);
    let mirrored = ShipInterior::for_variant(1);
    // At least one room should have moved under the horizontal mirror.
    let moved = base
        .rooms
        .iter()
        .zip(mirrored.rooms.iter())
        .any(|(a, b)| (a.x - b.x).abs() > 0.5);
    assert!(moved, "horizontal mirror should reposition rooms");
}
