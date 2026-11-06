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
fn initial_selection_and_menu_follow_recent_tracking() {
    let mut tracker = Tracker::new(Snapshot::default());
    let first = tracker.add_task("First").unwrap();
    let second = tracker.add_task("Second").unwrap();
    let unused = tracker.add_task("Unused").unwrap();
    let now = Utc::now();
    tracker.start(second, now).unwrap();
    tracker.pause(now + chrono::Duration::seconds(1)).unwrap();
    tracker
        .start(first, now + chrono::Duration::seconds(2))
        .unwrap();
    tracker.pause(now + chrono::Duration::seconds(3)).unwrap();

    let (mut app, _) = App::new();
    let _ = app.update(stored(tracker.into_snapshot()));
    assert_eq!(app.selected.as_ref().unwrap().id, first);
    app.settings.task_limit = 2;
    assert_eq!(
        ui::task_selector::recent_tasks(&app)
            .map(|task| task.id)
            .collect::<Vec<_>>(),
        [second, first]
    );
    let _ = app.update(Message::ToggleTaskMenu);
    let _ = app.update(Message::MenuSubmit);
    assert_eq!(app.selected.as_ref().unwrap().id, second);

    app.snapshot.active = Some(ActiveSession {
        task_id: unused,
        started_at: now + chrono::Duration::seconds(4),
    });
    assert_eq!(
        ui::task_selector::recent_tasks(&app)
            .map(|task| task.id)
            .collect::<Vec<_>>(),
        [first, unused]
    );
}

#[test]
fn initial_selection_falls_back_to_newest_unused_task() {
    let mut tracker = Tracker::new(Snapshot::default());
    tracker.add_task("First").unwrap();
    let newest = tracker.add_task("Newest").unwrap();
    let (mut app, _) = App::new();
    let _ = app.update(stored(tracker.into_snapshot()));
    assert_eq!(app.selected.as_ref().unwrap().id, newest);
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
    let _ = app.update(Message::Escape);
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
    let _ = app.update(Message::Escape);
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
    let _ = app.update(Message::Escape);
    assert!(!app.task_menu_open);
}

#[test]
fn selector_search_filters_all_tasks_before_applying_the_limit() {
    fn result_count(app: &App) -> usize {
        ui::task_selector::matching_tasks(app).count()
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
    let _ = app.update(Message::TaskName("NEEDLE".into()));
    assert_eq!(result_count(&app), 3);
    app.settings.task_limit = 20;
    assert_eq!(result_count(&app), 4);
    let _ = app.update(Message::ToggleTaskMenu);
    assert!(app.new_task_name.is_empty());
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

#[test]
fn search_and_menu_keyboard_selection_and_dismissal() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    tracker.add_task("Alpha").unwrap();
    tracker.add_task("Alpine").unwrap();
    app.snapshot = tracker.into_snapshot();
    let _ = app.update(Message::TaskName("Al".into()));
    let _ = app.update(Message::SearchMove(1));
    assert_eq!(app.search_index, 1);
    let _ = app.update(Message::SearchSubmit);
    assert_eq!(app.selected.as_ref().unwrap().name, "Alpine");
    assert!(app.new_task_name.is_empty());
    let _ = app.update(Message::ToggleTaskMenu);
    let _ = app.update(Message::SearchMove(1));
    let _ = app.update(Message::SearchMove(-1));
    let _ = app.update(Message::MenuSubmit);
    assert_eq!(app.selected.as_ref().unwrap().name, "Alpha");
    assert!(!app.task_menu_open);
    let _ = app.update(Message::TaskName("unknown".into()));
    let _ = app.update(Message::DismissSearch);
    assert!(app.new_task_name.is_empty());
    let _ = app.update(Message::ToggleTaskMenu);
    let _ = app.update(Message::DismissTaskMenu);
    assert!(!app.task_menu_open);
    let _ = app.update(Message::TaskName("test".into()));
    let _ = app.update(Message::Escape);
    assert!(app.new_task_name.is_empty());
}

#[test]
fn frame_selection_and_time_editor_follow_tracking_and_validate_input() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let start = Utc::now() - chrono::Duration::minutes(20);
    tracker.start(task, start).unwrap();
    let active = tracker.into_snapshot();
    let _ = app.update(stored(active.clone()));
    assert_eq!(
        app.selected_frame,
        Some(FrameSelection::Active(task, start))
    );
    let _ = app.update(Message::EditFrameTime(FrameField::Start));
    let _ = app.update(Message::FrameHourInput("23".into()));
    let _ = app.update(Message::FrameHourInput("24".into()));
    assert_eq!(app.frame_hour_input, "23");
    let _ = app.update(Message::FrameMinuteInput("59".into()));
    let _ = app.update(Message::FrameMinuteInput("60".into()));
    assert_eq!(app.frame_minute_input, "59");
    let _ = app.update(Message::Escape);
    assert!(app.editing_frame_field.is_none());
    let mut tracker = Tracker::new(active);
    tracker
        .pause(start + chrono::Duration::minutes(10))
        .unwrap();
    let completed = tracker.into_snapshot();
    let id = completed.frames[0].id;
    let _ = app.update(stored(completed));
    assert_eq!(app.selected_frame, Some(FrameSelection::Completed(id)));
}

#[test]
fn initial_load_selects_latest_completed_frame() {
    let (mut app, _) = App::new();
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let start = Utc::now() - chrono::Duration::hours(1);
    tracker.start(task, start).unwrap();
    tracker.pause(start + chrono::Duration::minutes(5)).unwrap();
    tracker
        .start(task, start + chrono::Duration::minutes(10))
        .unwrap();
    tracker
        .pause(start + chrono::Duration::minutes(15))
        .unwrap();
    let snapshot = tracker.into_snapshot();
    let id = snapshot.frames[1].id;
    let _ = app.update(stored(snapshot));
    assert_eq!(app.selected_frame, Some(FrameSelection::Completed(id)));
}

#[test]
fn add_frame_success_moves_calendar_without_changing_selection() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let now = Utc::now();
    let old_start = now - chrono::Duration::days(3);
    tracker
        .add_frame(
            task,
            old_start,
            old_start + chrono::Duration::minutes(5),
            now,
        )
        .unwrap();
    app.snapshot = tracker.into_snapshot();
    let old_selection = FrameSelection::Completed(app.snapshot.frames[0].id);
    app.selected_frame = Some(old_selection);
    let _ = app.update(Message::OpenAddFrame);
    let new_start = now - chrono::Duration::days(1);
    app.pending_new_frame = Some(new_start);
    let mut tracker = Tracker::new(app.snapshot.clone());
    tracker
        .add_frame(
            task,
            new_start,
            new_start + chrono::Duration::minutes(10),
            now,
        )
        .unwrap();
    let _ = app.update(stored(tracker.into_snapshot()));
    assert_eq!(app.selected_frame, Some(old_selection));
    assert_eq!(app.date, new_start.with_timezone(&Local).date_naive());
    assert!(app.add_frame_window.is_none());
}

#[test]
fn popup_task_creation_preserves_main_selection_and_reports_failures() {
    let mut app = ready_app();
    let mut tracker = Tracker::new(Snapshot::default());
    let original = tracker.add_task("Original").unwrap();
    app.snapshot = tracker.into_snapshot();
    app.selected = Some(app.snapshot.tasks[0].clone());
    let _ = app.update(Message::OpenAddFrame);
    let _ = app.update(Message::AddFrameSearch("New".into()));
    let _ = app.update(Message::CreateAddFrameTask);
    let _ = app.update(Message::Stored(Err("Save failed".into())));
    assert_eq!(app.add_frame_error.as_deref(), Some("Save failed"));
    assert_eq!(app.add_frame_search, "New");
    let _ = app.update(Message::CreateAddFrameTask);
    let mut tracker = Tracker::new(app.snapshot.clone());
    let new = tracker.add_task("New").unwrap();
    let _ = app.update(stored(tracker.into_snapshot()));
    assert_eq!(app.selected.as_ref().unwrap().id, original);
    assert_eq!(app.add_frame_task.as_ref().unwrap().id, new);
}

#[test]
fn popup_escape_closes_open_controls_before_window() {
    let mut app = ready_app();
    let _ = app.update(Message::OpenAddFrame);
    let id = app.add_frame_window.unwrap();
    let _ = app.update(Message::ToggleAddFrameDate);
    let _ = app.update(Message::WindowKey(id, Box::new(Message::Escape)));
    assert!(!app.add_frame_date_open);
    assert!(app.add_frame_window.is_some());
    let _ = app.update(Message::ToggleAddFrameTasks);
    let _ = app.update(Message::WindowKey(id, Box::new(Message::Escape)));
    assert!(!app.add_frame_tasks_open);
    assert!(app.add_frame_window.is_some());
    let _ = app.update(Message::WindowKey(id, Box::new(Message::Escape)));
    assert!(app.add_frame_window.is_none());
}
