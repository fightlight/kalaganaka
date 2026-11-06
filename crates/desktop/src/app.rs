use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use iced::{Element, Subscription, Task, window};
use tracking_core::Snapshot;

use crate::{
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
pub enum FrameSelection {
    Completed(uuid::Uuid),
    Active(uuid::Uuid, DateTime<Utc>),
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
    pub frame_hour_input: String,
    pub frame_minute_input: String,
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
    main_window: window::Id,
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
            exit_on_close_request: false,
            ..Default::default()
        });
        (
            Self {
                snapshot: Snapshot::default(),
                selected_frame: None,
                editing_frame_field: None,
                frame_hour_input: String::new(),
                frame_minute_input: String::new(),
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
                main_window,
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
        if Some(id) == self.statistics_window {
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
            crate::platform::appearance::changes().map(Message::SystemThemeChanged),
            iced::event::listen_with(|event, _status, _window| match event {
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
                    ..
                }) => Some(Message::FrameTab),
                _ => None,
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
                        let tracking_changed = self.snapshot.active != snapshot.active;
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
                                if self.adding_task && snapshot.active.is_none() {
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
                            .or_else(|| snapshot.tasks.last())
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
                            if let Some(active) = &snapshot.active {
                                self.selected_frame =
                                    Some(FrameSelection::Active(active.task_id, active.started_at));
                                self.editing_frame_field = None;
                            }
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
                        if tracking_changed && !self.saving_frame {
                            self.now = Local::now();
                            return ui::calendar::scroll_to_now(self);
                        }
                    }
                    Err(error) => {
                        self.error = Some(error);
                        self.quitting = false;
                    }
                }
            }
            Message::SelectFrame(frame) if self.ready() => {
                self.selected_frame = Some(frame);
                self.editing_frame_field = None;
                self.error = None;
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
                let count = ui::task_selector::matching_tasks(self).count();
                if count > 0 {
                    self.search_index =
                        (self.search_index as i32 + direction).clamp(0, count as i32 - 1) as usize;
                }
            }
            Message::MenuSubmit
                if self.ready() && self.task_menu_open && self.editing_task.is_none() =>
            {
                let selected = ui::task_selector::matching_tasks(self)
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
            Message::OpenStatistics => {
                if let Some(id) = self.statistics_window {
                    return window::gain_focus(id);
                }
                let (id, open) = window::open(window::Settings {
                    size: iced::Size::new(540.0, 580.0),
                    exit_on_close_request: false,
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
        match self.selected_frame? {
            FrameSelection::Completed(id) => self
                .snapshot
                .frames
                .iter()
                .find(|frame| frame.id == id)
                .map(|frame| (frame.task_id, frame.started_at, Some(frame.ended_at))),
            FrameSelection::Active(task, started) => self
                .snapshot
                .active
                .as_ref()
                .filter(|active| active.task_id == task && active.started_at == started)
                .map(|active| (active.task_id, active.started_at, None)),
        }
    }

    pub fn ready(&self) -> bool {
        self.loaded && !self.busy
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if Some(id) == self.statistics_window {
            ui::statistics_window::view(self)
        } else if Some(id) == self.settings_window {
            ui::settings_window::view(self)
        } else {
            ui::main_window::view(self)
        }
    }
}
