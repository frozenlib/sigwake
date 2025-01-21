use super::BTreeMultiMap;

#[test]
fn new_is_empty() {
    let map: BTreeMultiMap<i32, String> = BTreeMultiMap::new();
    assert!(map.is_empty());
}

#[test]
fn insert() {
    let mut map = BTreeMultiMap::new();

    // Insert first value for key 1
    let id1 = map.insert(1, "one".to_string());
    assert_eq!(id1, 0);
    assert!(!map.is_empty());

    // Insert second value for same key
    let id2 = map.insert(1, "another one".to_string());
    assert_eq!(id2, 1);

    let id3 = map.insert(2, "two".to_string());
    assert_eq!(id3, 0);
}

#[test]
fn remove() {
    let mut map = BTreeMultiMap::new();

    // Prepare test data
    let id1 = map.insert(1, "one".to_string());
    let id2 = map.insert(1, "another one".to_string());
    map.insert(2, "two".to_string());

    // Remove existing element
    let removed = map.remove(1, id1);
    assert_eq!(removed, Some("one".to_string()));

    // Remove another element with same key but different id
    let removed = map.remove(1, id2);
    assert_eq!(removed, Some("another one".to_string()));

    // Try to remove non-existent element
    let removed = map.remove(1, 999);
    assert_eq!(removed, None);
}

#[test]
fn first_key() {
    let mut map = BTreeMultiMap::new();

    assert_eq!(map.first_key(), None);

    map.insert(3, "three".to_string());
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());

    assert_eq!(map.first_key(), Some(&1));
}

#[test]
fn first_entry() {
    let mut map = BTreeMultiMap::new();

    assert!(map.first_entry().is_none());

    map.insert(3, "three".to_string());
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());

    let entry = map.first_entry().unwrap();
    assert_eq!(entry.key(), &(1, 0));
    assert_eq!(entry.get(), &"one".to_string());
}
