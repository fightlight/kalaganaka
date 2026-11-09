use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use iced::{Element, Subscription, Task, window};
pub use tracking_core::FrameSelection;
use tracking_core::Snapshot;

use crate::{
    platform,
    settings::{MAX_TASK_LIMIT, MIN_TASK_LIMIT, Settings, ThemePreference},
    storage::{self, Command, Data},
    ui,
};

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;

#[derive(Debug, Clone)]
pub enum Message {
    Stored(Result<Data, String>),
    SelectFrame(FrameSelection),
    EditFrameTime(FrameField),
    FrameHourInput(String),
    FrameMinuteInput(String),
    FrameTab,
    FrameHourFocused(bool),
    FrameMinuteFocused(bool),
    SaveFrameTime,
    EditMetadata(bool),
    CancelMetadata,
    MetadataAreaPressed,
    FrameTimeHovered(bool),
    TagAreaHovered(bool),
    TagSuggestionsHovered(bool),
    DescriptionAreaHovered(bool),
    MetadataOutsideClick,
    SaveMetadata(bool),
    DescriptionInput(iced::widget::text_editor::Action),
    TagInput(String),
    TagSubmit,
    TagChoose(String),
    TagRemove(usize),
    TagKey(i32),
    TagKeyFocused(i32, bool),
    OpenAddFrame,
    AddFrameSearch(String),
    AddFrameSearchMove(i32),
    AddFrameSearchSubmit,
    DismissAddFrameTasks,
    CreateAddFrameTask,
    WindowKey(window::Id, Box<Message>),
    AddFrameTab(bool),
    AddFrameCheckFocus(bool, usize, bool),
    AddFrameActivateCheck(usize, bool),
    AddFrameTask(tracking_core::Task),
    ToggleAddFrameTasks,
    ToggleAddFrameDate,
    AddFrameMonth(i32),
    AddFrameDate(NaiveDate),
    AddFrameTime(usize, String),
    SaveNewFrame,
    Tick,
    Toggle,
    SelectTask(tracking_core::Task),
    ToggleTaskMenu,
    SearchMove(i32),
    SearchSubmit,
    MenuSubmit,
    DismissSearch,
    DismissTaskMenu,
    Escape,
    EditTask(tracking_core::Task),
    EditTaskName(String),
    SaveTaskName,
    TaskName(String),
    AddTask,
    ChangeView(ViewMode),
    Navigate(i64),
    Today,
    OpenStatistics,
    OpenSettings,
    SetTheme(ThemePreference),
    TaskLimitInput(String),
    SaveTaskLimit,
    AdjustTaskLimit(i32),
    SystemThemeChanged(iced::theme::Mode),
    WindowOpened(window::Id),
    WindowResized(window::Id, iced::Size),
    CloseRequested(window::Id),
    ConfirmQuit,
    CancelQuit,
    Retry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Day,
    Week,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameField {
    Start,
    End,
}

pub struct App {
    pub snapshot: Snapshot,
    pub selected_frame: Option<FrameSelection>,
    pub editing_frame_field: Option<FrameField>,
    frame_time_hovered: bool,
    pub frame_hour_input: String,
    pub frame_minute_input: String,
    pub editing_tags: bool,
    tag_area_hovered: bool,
    tag_suggestions_hovered: bool,
    description_area_hovered: bool,
    pub editing_description: bool,
    pub tag_input: String,
    pub tag_draft: Vec<String>,
    pub tag_index: usize,
    pub description_draft: iced::widget::text_editor::Content,
    saving_metadata: Option<bool>,
    saving_frame: bool,
    pub settings: Settings,
    pub selected: Option<tracking_core::Task>,
    pub new_task_name: String,
    pub task_menu_open: bool,
    pub search_index: usize,
    pub task_limit_input: String,
    pub task_limit_error: bool,
    pub editing_task: Option<(uuid::Uuid, String)>,
    pub date: NaiveDate,
    pub mode: ViewMode,
    pub now: chrono::DateTime<Local>,
    pub busy: bool,
    pub loaded: bool,
    pub error: Option<String>,
    pub confirm_quit: bool,
    pub recovered_session: bool,
    pub add_frame_window: Option<window::Id>,
    pub add_frame_date: NaiveDate,
    pub add_frame_search: String,
    pub add_frame_tasks_open: bool,
    pub add_frame_search_index: usize,
    pub add_frame_focus: usize,
    adding_frame_task: bool,
    pub add_frame_date_open: bool,
    pub add_frame_month: NaiveDate,
    pub add_frame_task: Option<tracking_core::Task>,
    pub add_frame_times: [String; 4],
    pub add_frame_error: Option<String>,
    pending_new_frame: Option<DateTime<Utc>>,
    main_window: window::Id,
    pub main_window_width: f32,
    statistics_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    system_theme: iced::theme::Mode,
    adding_task: bool,
    renaming_task: bool,
    saving_task_limit: bool,
    quitting: bool,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (main_window, open) = window::open(window::Settings {
            size: iced::Size::new(1050.0, 800.0),
            min_size: Some(iced::Size::new(384.0, 0.0)),
            exit_on_close_request: false,
            platform_specific: platform::window_settings(),
            ..Default::default()
        });
        (
            Self {
                snapshot: Snapshot::default(),
                selected_frame: None,
                editing_frame_field: None,
                frame_time_hovered: false,
                frame_hour_input: String::new(),
                frame_minute_input: String::new(),
                editing_tags: false,
                tag_area_hovered: false,
                tag_suggestions_hovered: false,
                description_area_hovered: false,
                editing_description: false,
                tag_input: String::new(),
                tag_draft: Vec::new(),
                tag_index: 0,
                description_draft: iced::widget::text_editor::Content::new(),
                saving_metadata: None,
                saving_frame: false,
                settings: Settings::default(),
                selected: None,
                new_task_name: String::new(),
                task_menu_open: false,
                search_index: 0,
                task_limit_input: Settings::default().task_limit.to_string(),
                task_limit_error: false,
                editing_task: None,
                date: Local::now().date_naive(),
                mode: ViewMode::Day,
                now: Local::now(),
                busy: true,
                loaded: false,
                error: None,
                confirm_quit: false,
                recovered_session: false,
                add_frame_window: None,
                add_frame_date: Local::now().date_naive(),
                add_frame_search: String::new(),
                add_frame_tasks_open: false,
                add_frame_search_index: 0,
                add_frame_focus: 0,
                adding_frame_task: false,
                add_frame_date_open: false,
                add_frame_month: Local::now().date_naive(),
                add_frame_task: None,
                add_frame_times: Default::default(),
                add_frame_error: None,
                pending_new_frame: None,
                main_window,
                main_window_width: 1050.0,
                statistics_window: None,
                settings_window: None,
                system_theme: iced::theme::Mode::None,
                adding_task: false,
                renaming_task: false,
                saving_task_limit: false,
                quitting: false,
            },
            Task::batch([
                open.map(Message::WindowOpened),
                Task::perform(storage::execute(Command::Load), Message::Stored),
            ]),
        )
    }

    pub fn title(&self, id: window::Id) -> String {
        if Some(id) == self.add_frame_window {
            "Kalaganaka — Add Time Frame".into()
        } else if Some(id) == self.statistics_window {
            "Kalaganaka — Statistics".into()
        } else if Some(id) == self.settings_window {
            "Kalaganaka — Settings".into()
        } else {
            "Kalaganaka".into()
        }
    }

    pub fn theme(&self, _id: window::Id) -> iced::Theme {
        self.settings.theme.theme(self.system_theme)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            window::close_requests().map(Message::CloseRequested),
            window::resize_events().map(|(id, size)| Message::WindowResized(id, size)),
            crate::platform::appearance::changes().map(Message::SystemThemeChanged),
            iced::event::listen_with(|event, _status, window| {
                let message = match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonPressed(
                        iced::mouse::Button::Left,
                    )) => Some(Message::MetadataOutsideClick),
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                        ..
                    }) => Some(Message::Escape),
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown),
                        ..
                    }) => Some(Message::SearchMove(1)),
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp),
                        ..
                    }) => Some(Message::SearchMove(-1)),
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter),
                        ..
                    }) => Some(Message::MenuSubmit),
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
                        modifiers,
                        ..
                    }) => Some(Message::AddFrameTab(modifiers.shift())),
                    _ => None,
                };
                message.map(|message| Message::WindowKey(window, Box::new(message)))
            }),
        ])
    }

    fn store(&mut self, command: Command) -> Task<Message> {
        self.busy = true;
        self.adding_task = matches!(&command, Command::AddTask(_));
        self.renaming_task = matches!(&command, Command::RenameTask(..));
        self.saving_frame = matches!(&command, Command::EditFrame { .. });
        self.saving_task_limit = matches!(&command, Command::SetTaskLimit(_));
        self.error = None;
        Task::perform(storage::execute(command), Message::Stored)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Stored(result) => {
                self.busy = false;
                match result {
                    Ok(Data { snapshot, settings }) => {
                        if let Some(tags) = self.saving_metadata.take() {
                            if tags {
                                self.editing_tags = false;
                                self.tag_input.clear();
                            } else {
                                self.editing_description = false;
                            }
                        }
                        let identity = |active: &Option<tracking_core::ActiveSession>| {
                            active.as_ref().map(|a| (a.task_id, a.started_at))
                        };
                        let tracking_changed =
                            identity(&self.snapshot.active) != identity(&snapshot.active);
                        if !self.loaded {
                            self.selected_frame = snapshot
                                .active
                                .as_ref()
                                .map(|active| {
                                    FrameSelection::Active(active.task_id, active.started_at)
                                })
                                .or_else(|| {
                                    snapshot
                                        .frames
                                        .iter()
                                        .max_by_key(|frame| frame.ended_at)
                                        .map(|frame| FrameSelection::Completed(frame.id))
                                });
                        }
                        if !self.loaded || self.saving_task_limit {
                            self.task_limit_input = settings.task_limit.to_string();
                            self.task_limit_error = false;
                        }
                        self.settings = settings;
                        self.recovered_session = !self.loaded && snapshot.active.is_some();
                        self.loaded = true;
                        self.selected = snapshot
                            .active
                            .as_ref()
                            .and_then(|active| {
                                snapshot.tasks.iter().find(|task| task.id == active.task_id)
                            })
                            .or_else(|| {
                                if self.adding_task
                                    && !self.adding_frame_task
                                    && snapshot.active.is_none()
                                {
                                    snapshot.tasks.last()
                                } else {
                                    None
                                }
                            })
                            .or_else(|| {
                                self.selected.as_ref().and_then(|selected| {
                                    snapshot.tasks.iter().find(|task| task.id == selected.id)
                                })
                            })
                            .or_else(|| {
                                snapshot.tasks.iter().max_by_key(|task| {
                                    ui::task_selector::last_used(&snapshot, task.id)
                                })
                            })
                            .cloned();
                        if self.saving_frame {
                            if matches!(self.selected_frame, Some(FrameSelection::Active(..))) {
                                self.selected_frame = snapshot.active.as_ref().map(|active| {
                                    FrameSelection::Active(active.task_id, active.started_at)
                                });
                            }
                            self.editing_frame_field = None;
                        } else if let Some(FrameSelection::Active(task_id, started_at)) =
                            self.selected_frame
                        {
                            if !snapshot
                                .active
                                .as_ref()
                                .is_some_and(|a| a.task_id == task_id && a.started_at == started_at)
                            {
                                self.selected_frame = snapshot
                                    .frames
                                    .iter()
                                    .find(|frame| {
                                        frame.task_id == task_id && frame.started_at == started_at
                                    })
                                    .map(|frame| FrameSelection::Completed(frame.id));
                                self.editing_frame_field = None;
                            }
                        }
                        if tracking_changed && !self.saving_frame {
                            self.editing_tags = false;
                            self.editing_description = false;
                            self.tag_input.clear();
                            if let Some(active) = &snapshot.active {
                                self.selected_frame =
                                    Some(FrameSelection::Active(active.task_id, active.started_at));
                                self.editing_frame_field = None;
                            }
                        }
                        if self.adding_frame_task {
                            self.add_frame_task = snapshot.tasks.last().cloned();
                            self.add_frame_tasks_open = false;
                            self.add_frame_search.clear();
                        }
                        self.snapshot = snapshot;
                        if self.adding_task {
                            self.new_task_name.clear();
                        }
                        if self.renaming_task {
                            self.editing_task = None;
                        }
                        if self.quitting {
                            return iced::exit();
                        }
                        if let Some(start) = self.pending_new_frame.take() {
                            self.date = start.with_timezone(&Local).date_naive();
                            let close = self
                                .add_frame_window
                                .take()
                                .map(window::close)
                                .unwrap_or_else(Task::none);
                            return Task::batch([
                                close,
                                window::gain_focus(self.main_window),
                                ui::calendar::scroll_to_time(start),
                            ]);
                        }
                        if tracking_changed && !self.saving_frame {
                            self.now = Local::now();
                            return ui::calendar::scroll_to_now(self);
                        }
                    }
                    Err(error) => {
                        self.saving_metadata = None;
                        if self.pending_new_frame.take().is_some() || self.adding_frame_task {
                            self.add_frame_error = Some(error);
                        } else {
                            self.error = Some(error);
                        }
                        self.quitting = false;
                    }
                }
                self.adding_frame_task = false;
            }
            Message::SelectFrame(frame) if self.ready() => {
                self.selected_frame = Some(frame);
                self.editing_tags = false;
                self.editing_description = false;
                self.tag_input.clear();
                self.editing_frame_field = None;
                self.error = None;
            }
            Message::OpenAddFrame if self.ready() => {
                if let Some(id) = self.add_frame_window {
                    return window::gain_focus(id);
                }
                self.add_frame_date = self.date;
                self.add_frame_month = self.date;
                self.add_frame_date_open = false;
                self.add_frame_tasks_open = false;
                self.add_frame_search.clear();
                self.add_frame_search_index = 0;
                self.add_frame_focus = 0;
                self.add_frame_task = self.selected.clone();
                self.add_frame_times = Default::default();
                self.add_frame_error = None;
                let (id, open) = window::open(window::Settings {
                    size: iced::Size::new(440.0, 620.0),
                    exit_on_close_request: false,
                    platform_specific: platform::window_settings(),
                    ..Default::default()
                });
                self.add_frame_window = Some(id);
                return open.map(Message::WindowOpened);
            }
            Message::WindowKey(id, message) => {
                if id == self.main_window {
                    if self.editing_tags {
                        if let Message::SearchMove(direction) = *message {
                            return self.update(Message::TagKey(direction));
                        }
                    }
                    return self.update(match *message {
                        Message::AddFrameTab(_) => Message::FrameTab,
                        other => other,
                    });
                }
                if Some(id) == self.add_frame_window {
                    match *message {
                        Message::Escape if self.add_frame_tasks_open => {
                            self.add_frame_tasks_open = false
                        }
                        Message::Escape if self.add_frame_date_open => {
                            self.add_frame_date_open = false
                        }
                        Message::Escape if !self.busy => {
                            return self.update(Message::CloseRequested(id));
                        }
                        Message::SearchMove(direction) => {
                            return self.update(Message::AddFrameSearchMove(direction));
                        }
                        Message::MenuSubmit if self.add_frame_tasks_open => {
                            return self.update(Message::AddFrameSearchSubmit);
                        }
                        Message::MenuSubmit => {
                            return iced::widget::operation::is_focused("add-frame-time-0")
                                .map(|focused| Message::AddFrameActivateCheck(0, focused));
                        }
                        Message::AddFrameTab(reverse) => {
                            return self.update(Message::AddFrameTab(reverse));
                        }
                        _ => {}
                    }
                }
            }
            Message::AddFrameTab(reverse) if !self.busy => {
                if self.add_frame_tasks_open {
                    self.add_frame_tasks_open = false;
                    self.add_frame_focus = 0;
                }
                return iced::widget::operation::is_focused("add-frame-time-0")
                    .map(move |focused| Message::AddFrameCheckFocus(reverse, 0, focused));
            }
            Message::AddFrameCheckFocus(reverse, index, focused) => {
                if focused {
                    self.add_frame_focus = index + 1;
                }
                if !focused && index < 3 {
                    let next = index + 1;
                    return iced::widget::operation::is_focused(format!("add-frame-time-{next}"))
                        .map(move |focused| Message::AddFrameCheckFocus(reverse, next, focused));
                }
                self.add_frame_focus = if reverse {
                    (self.add_frame_focus + 6) % 7
                } else {
                    (self.add_frame_focus + 1) % 7
                };
                let id = if (1..=4).contains(&self.add_frame_focus) {
                    format!("add-frame-time-{}", self.add_frame_focus - 1)
                } else {
                    "add-frame-button-focus".to_owned()
                };
                return iced::widget::operation::focus(id);
            }
            Message::AddFrameActivateCheck(index, focused) => {
                if !focused && index < 3 {
                    let next = index + 1;
                    return iced::widget::operation::is_focused(format!("add-frame-time-{next}"))
                        .map(move |focused| Message::AddFrameActivateCheck(next, focused));
                }
                if !focused {
                    match self.add_frame_focus {
                        0 => return self.update(Message::ToggleAddFrameTasks),
                        5 => return self.update(Message::SaveNewFrame),
                        6 => {
                            if let Some(id) = self.add_frame_window {
                                return self.update(Message::CloseRequested(id));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::AddFrameSearch(value) => {
                self.add_frame_search = value;
                self.add_frame_search_index = 0;
            }
            Message::AddFrameSearchMove(direction) if self.add_frame_tasks_open => {
                let count = ui::add_frame_window::matching_tasks(self).count();
                if count > 0 {
                    self.add_frame_search_index = (self.add_frame_search_index as i32 + direction)
                        .clamp(0, count as i32 - 1)
                        as usize;
                    return iced::widget::operation::snap_to(
                        "add-frame-results",
                        iced::widget::scrollable::RelativeOffset {
                            x: 0.0,
                            y: self.add_frame_search_index as f32
                                / count.saturating_sub(1).max(1) as f32,
                        },
                    );
                }
            }
            Message::AddFrameSearchSubmit if self.add_frame_tasks_open && self.ready() => {
                let selected = ui::add_frame_window::matching_tasks(self)
                    .nth(self.add_frame_search_index)
                    .cloned();
                if let Some(task) = selected {
                    return self.update(Message::AddFrameTask(task));
                }
                return self.update(Message::CreateAddFrameTask);
            }
            Message::DismissAddFrameTasks => self.add_frame_tasks_open = false,
            Message::CreateAddFrameTask
                if self.ready()
                    && !self.add_frame_search.trim().is_empty()
                    && ui::add_frame_window::matching_tasks(self).next().is_none() =>
            {
                let name = self.add_frame_search.trim().to_owned();
                self.add_frame_error = None;
                self.adding_frame_task = true;
                return self.store(Command::AddTask(name));
            }
            Message::AddFrameTask(task) => {
                self.add_frame_task = Some(task);
                self.add_frame_tasks_open = false;
                self.add_frame_search.clear();
            }
            Message::ToggleAddFrameTasks => {
                self.add_frame_focus = 0;
                self.add_frame_tasks_open = !self.add_frame_tasks_open;
                self.add_frame_search_index = 0;
                self.add_frame_date_open = false;
                if self.add_frame_tasks_open {
                    return iced::widget::operation::focus("add-frame-search");
                }
            }
            Message::ToggleAddFrameDate => {
                self.add_frame_date_open = !self.add_frame_date_open;
                self.add_frame_tasks_open = false;
            }
            Message::AddFrameMonth(direction) => {
                let months = chrono::Months::new(direction.unsigned_abs());
                let next = if direction < 0 {
                    self.add_frame_month.checked_sub_months(months)
                } else {
                    self.add_frame_month.checked_add_months(months)
                };
                if let Some(date) = next {
                    self.add_frame_month = date;
                }
            }
            Message::AddFrameDate(date) => {
                self.add_frame_date = date;
                self.add_frame_month = date;
                self.add_frame_date_open = false;
            }
            Message::AddFrameTime(index, value) if index < 4 => {
                let max = if index % 2 == 0 { 23 } else { 59 };
                if value.is_empty()
                    || (value.len() <= 2
                        && value.bytes().all(|b| b.is_ascii_digit())
                        && value.parse::<u8>().is_ok_and(|n| n <= max))
                {
                    self.add_frame_times[index] = value;
                }
            }
            Message::SaveNewFrame if self.ready() && self.add_frame_window.is_some() => {
                let Some(task_id) = self.add_frame_task.as_ref().map(|task| task.id) else {
                    self.add_frame_error = Some("Select a task.".into());
                    return Task::none();
                };
                let parse = |hour: &str, minute: &str| {
                    NaiveTime::from_hms_opt(hour.parse().ok()?, minute.parse().ok()?, 0)
                        .and_then(|time| {
                            Local
                                .from_local_datetime(&self.add_frame_date.and_time(time))
                                .single()
                        })
                        .map(|time| time.with_timezone(&Utc))
                };
                let Some((start, end)) = parse(&self.add_frame_times[0], &self.add_frame_times[1])
                    .zip(parse(&self.add_frame_times[2], &self.add_frame_times[3]))
                else {
                    self.add_frame_error =
                        Some("Enter valid, unambiguous local times in HH:MM format.".into());
                    return Task::none();
                };
                self.add_frame_error = None;
                self.pending_new_frame = Some(start);
                return self.store(Command::AddFrame {
                    task_id,
                    start,
                    end,
                    now: Utc::now(),
                });
            }
            Message::EditFrameTime(field) if self.ready() => {
                if let Some((_, start, end)) = self.frame_details() {
                    let value = match field {
                        FrameField::Start => Some(start),
                        FrameField::End => end,
                    };
                    if let Some(value) = value {
                        let local = value.with_timezone(&Local);
                        self.frame_hour_input = local.format("%H").to_string();
                        self.frame_minute_input = local.format("%M").to_string();
                        self.editing_frame_field = Some(field);
                        self.frame_time_hovered = true;
                        return iced::widget::operation::focus("frame-time");
                    }
                }
            }
            Message::FrameHourInput(value) => {
                if value.is_empty()
                    || (value.len() <= 2
                        && value.bytes().all(|ch| ch.is_ascii_digit())
                        && value.parse::<u8>().is_ok_and(|hour| hour <= 23))
                {
                    self.frame_hour_input = value;
                }
            }
            Message::FrameMinuteInput(value) => {
                if value.is_empty()
                    || (value.len() <= 2
                        && value.bytes().all(|ch| ch.is_ascii_digit())
                        && value.parse::<u8>().is_ok_and(|minute| minute <= 59))
                {
                    self.frame_minute_input = value;
                }
            }
            Message::FrameTab if self.ready() && self.editing_frame_field.is_some() => {
                return iced::widget::operation::is_focused("frame-time")
                    .map(Message::FrameHourFocused);
            }
            Message::FrameHourFocused(focused)
                if self.ready() && self.editing_frame_field.is_some() =>
            {
                if focused {
                    return iced::widget::operation::focus("frame-minute");
                }
                return iced::widget::operation::is_focused("frame-minute")
                    .map(Message::FrameMinuteFocused);
            }
            Message::FrameMinuteFocused(true)
                if self.ready() && self.editing_frame_field.is_some() =>
            {
                return iced::widget::operation::focus("frame-time");
            }
            Message::SaveFrameTime if self.ready() => {
                let Some(field) = self.editing_frame_field else {
                    return Task::none();
                };
                let Some((_, start, end)) = self.frame_details() else {
                    return Task::none();
                };
                let parsed = self
                    .frame_hour_input
                    .parse::<u32>()
                    .ok()
                    .zip(self.frame_minute_input.parse::<u32>().ok())
                    .and_then(|(hours, minutes)| NaiveTime::from_hms_opt(hours, minutes, 0));
                let original = match field {
                    FrameField::Start => start,
                    FrameField::End => match end {
                        Some(end) => end,
                        None => return Task::none(),
                    },
                };
                if parsed.is_some_and(|time| {
                    time.format("%H:%M").to_string()
                        == original.with_timezone(&Local).format("%H:%M").to_string()
                }) {
                    self.editing_frame_field = None;
                    self.error = None;
                    return Task::none();
                }
                let local_date = original.with_timezone(&Local).date_naive();
                let Some(updated) = parsed
                    .and_then(|time| time.with_second(original.second()))
                    .and_then(|time| time.with_nanosecond(original.nanosecond()))
                    .and_then(|time| {
                        Local
                            .from_local_datetime(&local_date.and_time(time))
                            .single()
                    })
                    .map(|value| value.with_timezone(&Utc))
                else {
                    self.error =
                        Some("Enter a valid, unambiguous local time in HH:MM format.".into());
                    return Task::none();
                };
                let (new_start, new_end) = match field {
                    FrameField::Start => (updated, end),
                    FrameField::End => (start, Some(updated)),
                };
                let (id, expected_active) = match self.selected_frame {
                    Some(FrameSelection::Completed(id)) => (Some(id), None),
                    Some(FrameSelection::Active(task, started)) => (None, Some((task, started))),
                    None => return Task::none(),
                };
                return self.store(Command::EditFrame {
                    id,
                    expected_active,
                    start: new_start,
                    end: new_end,
                    now: Utc::now(),
                });
            }
            Message::EditMetadata(tags) if self.ready() => {
                let Some(metadata) = self.frame_metadata().cloned() else {
                    return Task::none();
                };
                if tags {
                    self.tag_draft = metadata.tags;
                    self.tag_input.clear();
                    self.tag_suggestions_hovered = false;
                    self.tag_index = 0;
                    self.editing_tags = true;
                    return iced::widget::operation::focus("frame-tags");
                }
                self.description_draft =
                    iced::widget::text_editor::Content::with_text(&metadata.description);
                self.editing_description = true;
                return iced::widget::operation::focus("frame-description");
            }
            Message::CancelMetadata if self.ready() => {
                self.editing_tags = false;
                self.editing_description = false;
                self.tag_input.clear();
                self.tag_suggestions_hovered = false;
            }
            Message::MetadataAreaPressed => {}
            Message::FrameTimeHovered(hovered) => self.frame_time_hovered = hovered,
            Message::TagAreaHovered(hovered) => self.tag_area_hovered = hovered,
            Message::TagSuggestionsHovered(hovered) => self.tag_suggestions_hovered = hovered,
            Message::DescriptionAreaHovered(hovered) => self.description_area_hovered = hovered,
            Message::MetadataOutsideClick if self.ready() => {
                if self.editing_frame_field.is_some() && !self.frame_time_hovered {
                    self.editing_frame_field = None;
                    self.frame_hour_input.clear();
                    self.frame_minute_input.clear();
                    self.error = None;
                }
                if self.editing_tags && !self.tag_area_hovered && !self.tag_suggestions_hovered {
                    self.editing_tags = false;
                    self.tag_input.clear();
                    self.tag_suggestions_hovered = false;
                }
                if self.editing_description && !self.description_area_hovered {
                    self.editing_description = false;
                }
            }
            Message::DescriptionInput(action) if self.editing_description && self.ready() => {
                self.description_draft.perform(action)
            }
            Message::TagInput(value) => {
                self.tag_input = value;
                if self.tag_input.trim().is_empty() {
                    self.tag_suggestions_hovered = false;
                }
                self.tag_index = 0;
            }
            Message::TagKey(direction) if self.editing_tags => {
                return iced::widget::operation::is_focused("frame-tags")
                    .map(move |focused| Message::TagKeyFocused(direction, focused));
            }
            Message::TagKeyFocused(direction, true) if self.editing_tags => {
                let count = self.tag_suggestions().len().max(1);
                self.tag_index =
                    (self.tag_index as i32 + direction).clamp(0, count as i32 - 1) as usize;
            }
            Message::TagSubmit if self.editing_tags && self.ready() => {
                if !self.tag_input.trim().is_empty() {
                    let tag = self
                        .tag_suggestions()
                        .get(self.tag_index)
                        .cloned()
                        .unwrap_or_else(|| self.tag_input.trim().to_owned());
                    return self.update(Message::TagChoose(tag));
                }
            }
            Message::TagChoose(tag) if self.editing_tags && self.ready() => {
                let tag = tag.trim();
                if !tag.is_empty()
                    && !self
                        .tag_draft
                        .iter()
                        .any(|existing| existing.eq_ignore_ascii_case(tag))
                {
                    self.tag_draft.push(tag.to_owned());
                    return self.update(Message::SaveMetadata(true));
                }
                self.tag_input.clear();
                self.tag_index = 0;
                return iced::widget::operation::focus("frame-tags");
            }
            Message::TagRemove(index) if self.ready() => {
                let Some(selection) = self.selected_frame else {
                    return Task::none();
                };
                let Some(mut metadata) = self.frame_metadata().cloned() else {
                    return Task::none();
                };
                if index >= metadata.tags.len() {
                    return Task::none();
                }
                metadata.tags.remove(index);
                self.saving_metadata = Some(true);
                return self.store(Command::SetFrameMetadata(selection, metadata));
            }
            Message::SaveMetadata(tags) if self.ready() => {
                if !(if tags {
                    self.editing_tags
                } else {
                    self.editing_description
                }) {
                    return Task::none();
                }
                let Some(selection) = self.selected_frame else {
                    return Task::none();
                };
                let Some(mut metadata) = self.frame_metadata().cloned() else {
                    return Task::none();
                };
                if tags {
                    metadata.tags = self.tag_draft.clone();
                } else {
                    metadata.description = self.description_draft.text();
                }
                self.saving_metadata = Some(tags);
                return self.store(Command::SetFrameMetadata(selection, metadata));
            }
            Message::Tick => self.now = Local::now(),
            Message::SystemThemeChanged(mode) => self.system_theme = mode,
            Message::Toggle if self.ready() => {
                self.recovered_session = false;
                if self.snapshot.active.is_some() {
                    return self.store(Command::Pause(Utc::now()));
                }
                if let Some(task) = &self.selected {
                    return self.store(Command::Start(task.id, Utc::now()));
                }
            }
            Message::SelectTask(task) if self.ready() => {
                self.task_menu_open = false;
                self.new_task_name.clear();
                self.editing_task = None;
                if self.snapshot.active.is_some() {
                    return self.store(Command::Start(task.id, Utc::now()));
                }
                self.selected = Some(task);
            }
            Message::ToggleTaskMenu if self.ready() => {
                self.task_menu_open = !self.task_menu_open;
                self.search_index = 0;
                self.editing_task = None;
                self.new_task_name.clear();
            }
            Message::SearchMove(direction)
                if self.ready()
                    && (self.task_menu_open || !self.new_task_name.trim().is_empty())
                    && self.editing_task.is_none() =>
            {
                let count = if self.task_menu_open {
                    ui::task_selector::recent_tasks(self).count()
                } else {
                    ui::task_selector::matching_tasks(self).count()
                };
                if count > 0 {
                    self.search_index =
                        (self.search_index as i32 + direction).clamp(0, count as i32 - 1) as usize;
                }
            }
            Message::MenuSubmit
                if self.ready() && self.task_menu_open && self.editing_task.is_none() =>
            {
                let selected = ui::task_selector::recent_tasks(self)
                    .nth(self.search_index)
                    .cloned();
                if let Some(task) = selected {
                    return self.update(Message::SelectTask(task));
                }
            }
            Message::SearchSubmit
                if self.ready()
                    && self.editing_task.is_none()
                    && !self.new_task_name.trim().is_empty() =>
            {
                let selected = ui::task_selector::matching_tasks(self)
                    .nth(self.search_index)
                    .cloned();
                if let Some(task) = selected {
                    return self.update(Message::SelectTask(task));
                }
                return self.update(Message::AddTask);
            }
            Message::DismissSearch => {
                self.editing_task = None;
                self.new_task_name.clear();
                self.search_index = 0;
            }
            Message::DismissTaskMenu => {
                self.task_menu_open = false;
                self.editing_task = None;
            }
            Message::Escape => {
                self.new_task_name.clear();
                self.search_index = 0;
                if self.ready() {
                    self.editing_tags = false;
                    self.editing_description = false;
                    self.tag_input.clear();
                    if self.editing_frame_field.take().is_some() {
                        self.frame_hour_input.clear();
                        self.frame_minute_input.clear();
                        self.error = None;
                    }
                    self.editing_task = None;
                    self.task_menu_open = false;
                }
            }
            Message::EditTask(task) if self.ready() => {
                self.editing_task = Some((task.id, task.name));
                return iced::widget::operation::focus("task-name-edit");
            }
            Message::EditTaskName(name) => {
                if let Some((_, current)) = &mut self.editing_task {
                    *current = name;
                }
            }
            Message::SaveTaskName if self.ready() => {
                if let Some((id, name)) = &self.editing_task {
                    return self.store(Command::RenameTask(*id, name.clone()));
                }
            }
            Message::TaskName(name) => {
                self.editing_task = None;
                self.new_task_name = name;
                self.search_index = 0;
            }
            Message::AddTask if self.ready() => {
                return self.store(Command::AddTask(self.new_task_name.clone()));
            }
            Message::ChangeView(mode) => {
                self.mode = mode;
                self.now = Local::now();
                return ui::calendar::scroll_to_now(self);
            }
            Message::Navigate(direction) => {
                let days = if self.mode == ViewMode::Day { 1 } else { 7 };
                if let Some(date) = self
                    .date
                    .checked_add_signed(chrono::Duration::days(direction * days))
                {
                    self.date = date;
                    self.now = Local::now();
                    return ui::calendar::scroll_to_now(self);
                }
            }
            Message::Today => {
                self.now = Local::now();
                self.date = self.now.date_naive();
                return ui::calendar::scroll_to_now(self);
            }
            Message::WindowOpened(id) if id == self.main_window => {
                self.now = Local::now();
                return ui::calendar::scroll_to_now(self);
            }
            Message::WindowResized(id, size) if id == self.main_window => {
                self.main_window_width = size.width;
            }
            Message::OpenStatistics => {
                if let Some(id) = self.statistics_window {
                    return window::gain_focus(id);
                }
                let (id, open) = window::open(window::Settings {
                    size: iced::Size::new(540.0, 580.0),
                    exit_on_close_request: false,
                    platform_specific: platform::window_settings(),
                    ..Default::default()
                });
                self.statistics_window = Some(id);
                return open.map(Message::WindowOpened);
            }
            Message::OpenSettings => {
                if let Some(id) = self.settings_window {
                    return window::gain_focus(id);
                }
                let (id, open) = window::open(window::Settings {
                    size: iced::Size::new(440.0, 320.0),
                    exit_on_close_request: false,
                    platform_specific: platform::window_settings(),
                    ..Default::default()
                });
                self.settings_window = Some(id);
                return open.map(Message::WindowOpened);
            }
            Message::SetTheme(theme) if self.ready() && theme != self.settings.theme => {
                return self.store(Command::SetTheme(theme));
            }
            Message::TaskLimitInput(value) => {
                if value.is_empty()
                    || (value.bytes().all(|byte| byte.is_ascii_digit())
                        && value.parse::<u32>().ok().is_some_and(|limit| {
                            (MIN_TASK_LIMIT..=MAX_TASK_LIMIT).contains(&limit)
                        }))
                {
                    self.task_limit_input = value;
                    self.task_limit_error = false;
                }
            }
            Message::SaveTaskLimit if self.ready() => {
                match self.task_limit_input.trim().parse::<u32>() {
                    Ok(limit) if (MIN_TASK_LIMIT..=MAX_TASK_LIMIT).contains(&limit) => {
                        if limit != self.settings.task_limit {
                            return self.store(Command::SetTaskLimit(limit));
                        }
                        self.task_limit_input = limit.to_string();
                    }
                    _ => self.task_limit_error = true,
                }
            }
            Message::AdjustTaskLimit(change) if self.ready() => {
                let current = self
                    .task_limit_input
                    .parse::<u32>()
                    .ok()
                    .filter(|value| (MIN_TASK_LIMIT..=MAX_TASK_LIMIT).contains(value))
                    .unwrap_or(self.settings.task_limit);
                let limit = current
                    .saturating_add_signed(change)
                    .clamp(MIN_TASK_LIMIT, MAX_TASK_LIMIT);
                self.task_limit_input = limit.to_string();
                self.task_limit_error = false;
                if limit != self.settings.task_limit {
                    return self.store(Command::SetTaskLimit(limit));
                }
            }
            Message::CloseRequested(id) => {
                if id == self.main_window {
                    if self.busy {
                        self.error =
                            Some("Please wait for storage to finish before closing.".into());
                    } else if self.snapshot.active.is_some() {
                        self.confirm_quit = true;
                    } else {
                        return iced::exit();
                    }
                } else {
                    if Some(id) == self.add_frame_window {
                        if self.pending_new_frame.is_some() {
                            return Task::none();
                        }
                        self.add_frame_window = None;
                    }
                    if Some(id) == self.statistics_window {
                        self.statistics_window = None;
                    }
                    if Some(id) == self.settings_window {
                        self.settings_window = None;
                    }
                    return window::close(id);
                }
            }
            Message::ConfirmQuit if self.ready() => {
                self.quitting = true;
                return self.store(Command::Pause(Utc::now()));
            }
            Message::CancelQuit => self.confirm_quit = false,
            Message::Retry if !self.busy => return self.store(Command::Load),
            _ => {}
        }
        Task::none()
    }

    pub fn frame_details(&self) -> Option<(uuid::Uuid, DateTime<Utc>, Option<DateTime<Utc>>)> {
        match self.snapshot.frame(self.selected_frame?)? {
            tracking_core::FrameRef::Completed(frame) => {
                Some((frame.task_id, frame.started_at, Some(frame.ended_at)))
            }
            tracking_core::FrameRef::Active(active) => {
                Some((active.task_id, active.started_at, None))
            }
        }
    }

    pub fn frame_metadata(&self) -> Option<&tracking_core::FrameMetadata> {
        match self.snapshot.frame(self.selected_frame?)? {
            tracking_core::FrameRef::Completed(frame) => Some(&frame.metadata),
            tracking_core::FrameRef::Active(active) => Some(&active.metadata),
        }
    }

    pub fn tag_suggestions(&self) -> Vec<String> {
        let query = self.tag_input.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.snapshot
            .tags
            .iter()
            .filter(|tag| {
                tag.to_lowercase().contains(&query)
                    && !self
                        .tag_draft
                        .iter()
                        .any(|selected| selected.eq_ignore_ascii_case(tag))
            })
            .cloned()
            .collect()
    }

    pub fn ready(&self) -> bool {
        self.loaded && !self.busy
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if Some(id) == self.add_frame_window {
            ui::add_frame_window::view(self)
        } else if Some(id) == self.statistics_window {
            ui::statistics_window::view(self)
        } else if Some(id) == self.settings_window {
            ui::settings_window::view(self)
        } else {
            ui::main_window::view(self)
        }
    }
}
