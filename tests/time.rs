use assert_call::{CallRecorder, call};
use sigwake::time::spawn_at;
use sigwake::utils::Action;
use std::time::{Duration, Instant, SystemTime};
use tokio::{spawn, test, time::sleep};

async fn wait_sleep() {
    sleep(Duration::from_millis(200)).await;
}

#[test]
async fn spawn_at_instant() {
    let mut cr = CallRecorder::new();
    let at = Instant::now() + Duration::from_millis(50);
    let _task = spawn_at(Action::new(|| call!("called")), at);
    wait_sleep().await;
    cr.verify("called");
}

#[test]
async fn spawn_at_system_time() {
    let mut cr = CallRecorder::new();
    let at = SystemTime::now() + Duration::from_millis(50);
    let _task = spawn_at(Action::new(|| call!("called")), at);
    wait_sleep().await;
    cr.verify("called");
}

#[test]
async fn spawn_at_duration() {
    let mut cr = CallRecorder::new();
    let duration = Duration::from_millis(50);
    let _task = spawn_at(Action::new(|| call!("called")), duration);
    wait_sleep().await;
    cr.verify("called");
}

#[test]
async fn cancel_timer() {
    let mut cr = CallRecorder::new();
    let at = Instant::now() + Duration::from_millis(50);
    let task = spawn_at(Action::new(|| call!("called")), at);
    drop(task);
    wait_sleep().await;
    cr.verify(());
}

#[test]
async fn spawn_multiple_timers() {
    let mut cr = CallRecorder::new();
    let expected = 10;
    let mut tasks = Vec::new();

    for i in 0..expected {
        let task = spawn_at(
            Action::new(move || call!("timer{i}")),
            Duration::from_millis(50 + i * 10),
        );
        tasks.push(task);
    }
    wait_sleep().await;

    let expected = (0..expected)
        .map(|i| format!("timer{i}"))
        .collect::<Vec<_>>();
    cr.verify(expected);
}

#[test]
async fn timer_ordering() {
    let mut cr = CallRecorder::new();
    let _task1 = spawn_at(Action::new(|| call!("second")), Duration::from_millis(50));
    let _task2 = spawn_at(Action::new(|| call!("first")), Duration::from_millis(25));
    wait_sleep().await;
    cr.verify(vec!["first", "second"]);
}

#[test]
async fn sleep_duration() {
    let mut cr = CallRecorder::new();
    let duration = Duration::from_millis(50);
    let _task = spawn(async move {
        sigwake::time::sleep(duration).await;
        call!("woke up");
    });
    wait_sleep().await;
    cr.verify("woke up");
}

#[test]
async fn sleep_instant() {
    let mut cr = CallRecorder::new();
    let at = Instant::now() + Duration::from_millis(50);
    let _task = spawn(async move {
        sigwake::time::sleep(at).await;
        call!("woke up");
    });
    wait_sleep().await;
    cr.verify("woke up");
}

#[test]
async fn sleep_system_time() {
    let mut cr = CallRecorder::new();
    let at = SystemTime::now() + Duration::from_millis(50);
    let _task = spawn(async move {
        sigwake::time::sleep(at).await;
        call!("woke up");
    });
    wait_sleep().await;
    cr.verify("woke up");
}

#[test]
async fn sleep_multiple() {
    let mut cr = CallRecorder::new();
    let expected = 3;
    let mut tasks = Vec::new();

    for i in 0..expected {
        let task = spawn(async move {
            sigwake::time::sleep(Duration::from_millis(50 + i * 10)).await;
            call!("woke up {i}");
        });
        tasks.push(task);
    }
    wait_sleep().await;

    let expected = (0..expected)
        .map(|i| format!("woke up {i}"))
        .collect::<Vec<_>>();
    cr.verify(expected);
}

#[test]
async fn sleep_duration_elapsed() {
    let duration = Duration::from_millis(50);
    let start = Instant::now();
    sigwake::time::sleep(duration).await;
    let elapsed = start.elapsed();
    assert!(
        elapsed >= duration,
        "sleep should take at least the specified duration"
    );
}

#[test]
async fn sleep_instant_elapsed() {
    let target_duration = Duration::from_millis(50);
    let target_time = Instant::now() + target_duration;
    sigwake::time::sleep(target_time).await;
    assert!(
        Instant::now() >= target_time,
        "sleep should wait until the target time"
    );
}

#[test]
async fn sleep_system_time_elapsed() {
    let target_duration = Duration::from_millis(50);
    let target_time = SystemTime::now() + target_duration;
    sigwake::time::sleep(target_time).await;
    assert!(
        SystemTime::now() >= target_time,
        "sleep should wait until the target time"
    );
}
