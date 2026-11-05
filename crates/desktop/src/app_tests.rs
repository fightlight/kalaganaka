use super::*;
use iced::advanced::widget::{Tree, tree};
use tracking_core::{ActiveSession, Tracker};

fn ready_app() -> App {
    let (mut app, _) = App::new();
    app.loaded = true;
    app.busy = false;
    app
}

fn stored(snapshot: Snapshot) -> Message {
    Message::Stored(Ok(Data {
        snapshot,
        settings: Settings::default(),
    }))
}

fn calendar_state(tree: &Tree, tag: tree::Tag) -> Option<*const ()> {
    if tree.tag == tag {
        if let tree::State::Some(state) = &tree.state {
            return Some(state.as_ref() as *const dyn std::any::Any as *const ());
        }
    }
    tree.children
        .iter()
        .find_map(|child| calendar_state(child, tag))
}

#[test]
fn calendar_state_survives_status_messages_appearing_and_disappearing() {
    let mut app = ready_app();
    let calendar_tag = Tree::new(ui::calendar::view(&app).as_widget()).tag;
    let mut tree = Tree::new(ui::main_window::view(&app).as_widget());
    let original = calendar_state(&tree, calendar_tag).expect("calendar has scroll state");

    for (busy, error, recovered, confirm_quit) in [
        (true, None, false, false),
        (false, None, false, false),
        (false, Some("Storage failed"), false, false),
        (false, None, true, false),
        (false, None, false, true),
        (true, Some("Storage failed"), true, true),
        (false, None, false, false),
    ] {
        app.busy = busy;
        app.error = error.map(str::to_owned);
        app.recovered_session = recovered;
        app.confirm_quit = confirm_quit;
        tree.diff(ui::main_window::view(&app).as_widget());
        assert_eq!(calendar_state(&tree, calendar_tag), Some(original));
    }
}

#[test]
fn successful_start_switch_and_pause_request_scrolling() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let first = tracker.add_task("Development").unwrap();
    let second = tracker.add_task("Review").unwrap();
    let start = Utc::now();
    app.snapshot = tracker.into_snapshot();

    for active in [
        Some(ActiveSession {
            task_id: first,
            started_at: start,
        }),
        Some(ActiveSession {
            task_id: second,
            started_at: start,
        }),
        None,
    ] {
        let mut snapshot = app.snapshot.clone();
        snapshot.active = active.clone();
        app.busy = true;
        assert!(app.update(stored(snapshot)).units() > 0);
        assert_eq!(app.snapshot.active, active);
        assert!(!app.busy);
    }
}

#[test]
fn ordinary_updates_and_failed_saves_do_not_request_scrolling() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    tracker.start(task_id, Utc::now()).unwrap();
    app.snapshot = tracker.into_snapshot();

    assert_eq!(app.update(Message::Tick).units(), 0);
    assert_eq!(app.update(Message::TaskName("Review".into())).units(), 0);
    assert_eq!(app.update(stored(app.snapshot.clone())).units(), 0);
    assert_eq!(
        app.update(Message::Stored(Err("Storage failed".into())))
            .units(),
        0
    );
    assert_eq!(app.snapshot.active.as_ref().unwrap().task_id, task_id);
}

#[test]
fn today_view_changes_and_navigation_request_scrolling_when_today_is_visible() {
    let mut app = ready_app();
    app.date = app.now.date_naive() - chrono::Days::new(1);
    assert!(app.update(Message::Today).units() > 0);
    assert!(app.update(Message::ChangeView(ViewMode::Week)).units() > 0);
    assert!(app.update(Message::ChangeView(ViewMode::Day)).units() > 0);
    assert_eq!(app.update(Message::Navigate(-1)).units(), 0);
    assert!(app.update(Message::Navigate(1)).units() > 0);
}
