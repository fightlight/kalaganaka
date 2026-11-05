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

#[test]
fn adding_task_selects_it_only_when_paused_and_save_succeeds() {
    for running in [false, true] {
        let mut app = ready_app();
        let mut tracker = Tracker::new(Snapshot::default());
        let original_id = tracker.add_task("Original").unwrap();
        if running {
            tracker.start(original_id, Utc::now()).unwrap();
        }
        app.snapshot = tracker.into_snapshot();
        app.selected = Some(app.snapshot.tasks[0].clone());
        app.new_task_name = "New".into();
        let _ = app.update(Message::AddTask);
        let _ = app.update(Message::Stored(Err("Save failed".into())));
        assert_eq!(app.selected.as_ref().unwrap().id, original_id);
        assert_eq!(app.new_task_name, "New");

        let _ = app.update(Message::AddTask);
        let mut tracker = Tracker::new(app.snapshot.clone());
        let new_id = tracker.add_task("New").unwrap();
        let _ = app.update(stored(tracker.into_snapshot()));
        assert_eq!(
            app.selected.as_ref().unwrap().id,
            if running { original_id } else { new_id }
        );
        assert!(app.new_task_name.is_empty());
    }
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

#[test]
fn navigation_uses_selected_view_and_today_returns_to_local_date() {
    let mut app = ready_app();
    let original = chrono::NaiveDate::from_ymd_opt(2026, 11, 5).unwrap();
    app.date = original;

    let _ = app.update(Message::ChangeView(ViewMode::Week));
    assert_eq!(app.mode, ViewMode::Week);
    let _ = app.update(Message::Navigate(1));
    assert_eq!(app.date, original + chrono::Days::new(7));
    let _ = app.update(Message::Navigate(-1));
    assert_eq!(app.date, original);

    let _ = app.update(Message::ChangeView(ViewMode::Day));
    assert_eq!(app.mode, ViewMode::Day);
    let _ = app.update(Message::Navigate(1));
    assert_eq!(app.date, original + chrono::Days::new(1));

    let _ = app.update(Message::Today);
    assert_eq!(app.date, app.now.date_naive());
}

#[test]
fn editing_can_be_cancelled_and_failed_saves_keep_the_draft() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let id = tracker.add_task("Development").unwrap();
    app.snapshot = tracker.into_snapshot();
    let task = app.snapshot.tasks[0].clone();
    let _ = app.update(Message::ToggleTaskMenu);
    let _ = app.update(Message::EditTask(task.clone()));
    let _ = app.update(Message::EditTaskName("Draft".into()));
    let _ = app.update(Message::CancelTaskEdit);
    assert!(app.editing_task.is_none());
    assert_eq!(app.snapshot.tasks[0].name, "Development");

    let _ = app.update(Message::EditTask(task));
    let _ = app.update(Message::EditTaskName("Draft".into()));
    let _ = app.update(Message::SaveTaskName);
    let _ = app.update(Message::Stored(Err("Storage failed".into())));
    assert_eq!(app.editing_task, Some((id, "Draft".into())));
    assert_eq!(app.error.as_deref(), Some("Storage failed"));
    let mut renamed = app.snapshot.clone();
    renamed.tasks[0].name = "Draft".into();
    let _ = app.update(Message::SaveTaskName);
    let _ = app.update(stored(renamed));
    assert!(app.editing_task.is_none());
    assert_eq!(app.snapshot.tasks[0].name, "Draft");
}

#[test]
fn calendar_state_survives_task_menu_and_editing() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    tracker.add_task("Development").unwrap();
    app.snapshot = tracker.into_snapshot();
    let tag = Tree::new(ui::calendar::view(&app).as_widget()).tag;
    let mut tree = Tree::new(ui::main_window::view(&app).as_widget());
    let original = calendar_state(&tree, tag).unwrap();
    let _ = app.update(Message::ToggleTaskMenu);
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(calendar_state(&tree, tag), Some(original));
    let _ = app.update(Message::EditTask(app.snapshot.tasks[0].clone()));
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(calendar_state(&tree, tag), Some(original));
    let _ = app.update(Message::CancelTaskEdit);
    let _ = app.update(Message::ToggleTaskMenu);
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(calendar_state(&tree, tag), Some(original));
}

#[test]
fn selector_scroll_state_survives_edit_save_and_cancel() {
    let mut app = ready_app();
    app.settings.task_limit = 20;
    let mut tracker = Tracker::new(Snapshot::default());
    for index in 0..20 {
        tracker.add_task(&format!("Task {index}")).unwrap();
    }
    app.snapshot = tracker.into_snapshot();
    let task = app.snapshot.tasks[15].clone();
    let _ = app.update(Message::ToggleTaskMenu);
    let scrollable: Element<'_, Message> = iced::widget::scrollable(iced::widget::text("")).into();
    let scroll_tag = Tree::new(scrollable.as_widget()).tag;
    let mut tree = Tree::new(ui::main_window::view(&app).as_widget());
    let scroll_state = |tree: &Tree| calendar_state(tree.children.last().unwrap(), scroll_tag);
    let original = scroll_state(&tree).expect("selector has scroll state");

    let _ = app.update(Message::EditTask(task.clone()));
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(scroll_state(&tree), Some(original));
    let _ = app.update(Message::EditTaskName("Renamed".into()));
    let _ = app.update(Message::SaveTaskName);
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(scroll_state(&tree), Some(original));
    let mut renamed = app.snapshot.clone();
    renamed.tasks[15].name = "Renamed".into();
    let _ = app.update(stored(renamed));
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(scroll_state(&tree), Some(original));
    let _ = app.update(Message::EditTask(app.snapshot.tasks[15].clone()));
    let _ = app.update(Message::CancelTaskEdit);
    tree.diff(ui::main_window::view(&app).as_widget());
    assert_eq!(scroll_state(&tree), Some(original));
}

#[test]
fn selector_search_filters_all_tasks_before_applying_the_limit() {
    fn result_count(app: &App) -> usize {
        fn find(tree: &Tree, tag: tree::Tag) -> Option<&Tree> {
            if tree.tag == tag {
                return Some(tree);
            }
            tree.children.iter().find_map(|child| find(child, tag))
        }
        let scrollable: Element<'_, Message> =
            iced::widget::scrollable(iced::widget::text("")).into();
        let tag = Tree::new(scrollable.as_widget()).tag;
        let menu = Tree::new(ui::task_selector::menu(app).as_widget());
        find(&menu, tag).unwrap().children[0].children.len()
    }

    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    for index in 0..12 {
        tracker.add_task(&format!("Other {index}")).unwrap();
    }
    for index in 0..4 {
        tracker.add_task(&format!("Needle {index}")).unwrap();
    }
    app.snapshot = tracker.into_snapshot();
    app.settings.task_limit = 3;
    assert_eq!(result_count(&app), 3);
    let _ = app.update(Message::SearchTasks("NEEDLE".into()));
    assert_eq!(result_count(&app), 3);
    app.settings.task_limit = 20;
    assert_eq!(result_count(&app), 4);
    let _ = app.update(Message::ToggleTaskMenu);
    assert!(app.task_search.is_empty());
}

#[test]
fn task_limit_input_rejects_invalid_characters_and_values() {
    let mut app = ready_app();
    for input in ["0", "31", "-1", "2a", " 2", "３"] {
        let _ = app.update(Message::TaskLimitInput(input.into()));
        assert_eq!(app.task_limit_input, "5");
    }
    for input in ["", "1", "30"] {
        let _ = app.update(Message::TaskLimitInput(input.into()));
        assert_eq!(app.task_limit_input, input);
    }
    let _ = app.update(Message::SaveTaskLimit);
    assert!(app.busy);
    let _ = app.update(Message::Stored(Ok(Data {
        snapshot: Snapshot::default(),
        settings: Settings {
            task_limit: 30,
            ..Settings::default()
        },
    })));
    let _ = app.update(Message::AdjustTaskLimit(1));
    assert_eq!(app.task_limit_input, "30");
    assert!(!app.busy);
}
