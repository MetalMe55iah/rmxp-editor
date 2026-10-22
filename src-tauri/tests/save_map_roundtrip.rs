use essentials_studio_lib::commands::{load_map, save_map};

#[tokio::test]
async fn map_tile_edits_survive_two_save_and_reload_cycles() {
    let project = std::env::var("ESSENTIALS_STUDIO_TEST_PROJECT")
        .expect("ESSENTIALS_STUDIO_TEST_PROJECT must point to a disposable project copy");
    let map_id = std::env::var("ESSENTIALS_STUDIO_TEST_MAP_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1);

    let original = load_map(project.clone(), map_id)
        .await
        .expect("test map should load");
    assert!(!original.tiles.is_empty(), "test map must contain tile data");

    let mut first_edit = original.tiles.clone();
    first_edit[0] = if first_edit[0] == 384 { 385 } else { 384 };
    save_map(
        project.clone(),
        map_id,
        first_edit.clone(),
        original.width,
        original.height,
    )
    .await
    .expect("first save should succeed");
    let reopened_after_first = load_map(project.clone(), map_id)
        .await
        .expect("map should reopen after first save");
    assert_eq!(reopened_after_first.tiles, first_edit);

    let mut second_edit = first_edit.clone();
    second_edit[0] = if second_edit[0] == 386 { 387 } else { 386 };
    save_map(
        project.clone(),
        map_id,
        second_edit.clone(),
        original.width,
        original.height,
    )
    .await
    .expect("second save should succeed");
    let reopened_after_second = load_map(project.clone(), map_id)
        .await
        .expect("map should reopen after second save");
    assert_eq!(reopened_after_second.tiles, second_edit);

    save_map(
        project.clone(),
        map_id,
        original.tiles.clone(),
        original.width,
        original.height,
    )
    .await
    .expect("original tile data should be restored");
    let restored = load_map(project, map_id)
        .await
        .expect("restored map should reopen");
    assert_eq!(restored.tiles, original.tiles);
}

#[tokio::test]
async fn rejected_map_save_leaves_the_file_unchanged() {
    let project = std::env::var("ESSENTIALS_STUDIO_TEST_PROJECT")
        .expect("ESSENTIALS_STUDIO_TEST_PROJECT must point to a disposable project copy");
    let map_id = std::env::var("ESSENTIALS_STUDIO_TEST_MAP_ID")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1);
    let map_path = std::path::Path::new(&project)
        .join("Data")
        .join(format!("Map{map_id:03}.rxdata"));
    let bytes_before = std::fs::read(&map_path).expect("test map should be readable");
    let map = load_map(project.clone(), map_id)
        .await
        .expect("test map should load");
    let mut invalid_tiles = map.tiles.clone();
    invalid_tiles.pop();

    let error = save_map(
        project,
        map_id,
        invalid_tiles,
        map.width,
        map.height,
    )
    .await
    .expect_err("invalid tile data must be rejected");
    assert!(error.contains("Tile count"));
    assert_eq!(std::fs::read(map_path).unwrap(), bytes_before);
}
