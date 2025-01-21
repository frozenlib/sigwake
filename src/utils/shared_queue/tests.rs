use super::*;

#[test]
fn basic_push_and_read() {
    let mut queue = SharedQueue::new();
    let mut cursor = queue.create_cursor();

    queue.push(1);
    queue.push(2);
    queue.push(3);

    let mut reader = queue.read(&mut cursor);
    assert_eq!(reader.pop(), Some(&1));
    assert_eq!(reader.pop(), Some(&2));
    assert_eq!(reader.pop(), Some(&3));
    assert_eq!(reader.pop(), None);
    drop(reader);

    queue.drop_cursor(cursor);
}

#[test]
fn multiple_cursors() {
    let mut queue = SharedQueue::new();
    let mut cursor1 = queue.create_cursor();
    let mut cursor2 = queue.create_cursor();

    queue.push(1);
    queue.push(2);

    let mut reader1 = queue.read(&mut cursor1);
    assert_eq!(reader1.pop(), Some(&1));
    assert_eq!(reader1.pop(), Some(&2));
    assert_eq!(reader1.pop(), None);
    drop(reader1);

    let mut reader2 = queue.read(&mut cursor2);
    assert_eq!(reader2.pop(), Some(&1));
    assert_eq!(reader2.pop(), Some(&2));
    assert_eq!(reader2.pop(), None);
    drop(reader2);

    queue.drop_cursor(cursor1);
    queue.drop_cursor(cursor2);
}

#[test]
fn memory_cleanup() {
    let mut queue = SharedQueue::new();
    queue.push(1);
    queue.push(2);

    let cursor = queue.create_cursor();
    queue.drop_cursor(cursor);

    assert_eq!(queue.values.len(), 0);
    assert_eq!(queue.ref_counts.len(), 1);
}

#[test]
fn memory_no_cursor() {
    let mut queue = SharedQueue::new();
    queue.push(1);
    queue.push(2);

    assert_eq!(queue.values.len(), 0);
    assert_eq!(queue.ref_counts.len(), 1);
}

#[test]
fn cursor_position_update() {
    let mut queue = SharedQueue::new();
    let mut cursor = queue.create_cursor();

    queue.push(1);
    queue.push(2);

    let mut reader = queue.read(&mut cursor);
    assert_eq!(reader.pop(), Some(&1));
    drop(reader);

    let mut reader = queue.read(&mut cursor);
    assert_eq!(reader.pop(), Some(&2));
    drop(reader);

    queue.drop_cursor(cursor);
}

#[test]
fn create_cursor_after_push() {
    let mut queue = SharedQueue::new();
    queue.push(1);
    let mut cursor = queue.create_cursor();
    queue.push(2);
    let mut reader = queue.read(&mut cursor);
    assert_eq!(reader.pop(), Some(&2));
    assert_eq!(reader.pop(), None);
    drop(reader);
    queue.drop_cursor(cursor);
}

#[test]
fn create_cursor_when_unread_value_exists() {
    let mut queue = SharedQueue::new();
    let mut cursor1 = queue.create_cursor();
    queue.push(1);
    let mut cursor2 = queue.create_cursor();
    queue.push(2);

    let mut reader2 = queue.read(&mut cursor2);
    assert_eq!(reader2.pop(), Some(&2));
    assert_eq!(reader2.pop(), None);
    drop(reader2);

    let mut reader1 = queue.read(&mut cursor1);
    assert_eq!(reader1.pop(), Some(&1));
    assert_eq!(reader1.pop(), Some(&2));
    assert_eq!(reader1.pop(), None);
    drop(reader1);
}

#[test]
#[should_panic]
fn ref_count_underflow() {
    let mut queue = SharedQueue::new();
    queue.push(1);
    queue.decrement_ref_count(0);
}
