use iced::{
    button, scrollable, window, text_input, HorizontalAlignment, Scrollable, Container, Length, Align, Button, Column, Element, Row, Sandbox, Settings, Text, TextInput,
};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::env;
use serde::{Serialize, Deserialize};

// save tasks json file
const SAVE_FILENAME: &str = "task.json";

pub fn main() -> iced::Result {
    TaskManager::run(Settings {
        window: window::Settings {
            size: (400, 500),
            max_size: None,
            resizable: false,
            ..window::Settings::default()
        },
        default_font: Some(include_bytes!(
            "../fonts/YujiSyuku-Regular.ttf"
        )),
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskStatus {
    New,
    Progress,
    Stop,
    Done,
}

impl TaskStatus {
    fn to_string(&self) -> &str {
        match self {
            Self::New => "新規",
            Self::Progress => "実行中",
            Self::Stop => "停止",
            Self::Done => "完了",
        }
    }

    fn next_status(&self) -> Self {
        match self {
            Self::New => Self::Progress,
            Self::Progress => Self::Stop,
            Self::Stop => Self::Done,
            Self::Done => Self::New,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u32,
    content: String,
    status: TaskStatus,

    #[serde(skip)]
    delete_state: button::State,
    #[serde(skip)]
    status_state: button::State,
}

impl Task {
    /// create task
    fn create(id: u32, content: String) -> Self {
        Self {
            id: id,
            content: content,
            status: TaskStatus::New,
            delete_state: button::State::new(),
            status_state: button::State::new(),
        }
    }

    /// change task status
    fn change_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// task equals
    fn equals(&self, id: u32) -> bool {
        self.id == id
    }

    /// create task view
    fn view(&mut self) -> Element<Message> {
        Row::new()
            .push(
                Container::new(
                    Text::new(self.content.to_string())
                )
                    .width(Length::Fill)
                    .padding(5)
            )
            .push(
                Button::new(&mut self.status_state, Text::new(self.status.to_string()))
                    .style(style::StatusButton::to_status_button(&self.status))
                    .on_press(Message::ChangeStatusTask(self.id)),
            )
            .push(
                Button::new(&mut self.delete_state, Text::new("削除"))
                    .style(
                        style::Button::Warning
                    )
                    .on_press(Message::DeleteTask(self.id)),
            )
            .spacing(10)
            .into()
    }
}

/// task collection
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct Tasks {
    id_counter: u32,
    tasks: Vec<Task>,
}

impl Tasks {
    /// construct
    fn new() -> Self {
        Self {
            id_counter: 0,
            tasks: Vec::new(),
        }
    }

    /// find task by id
    fn find_by_id(&mut self, id: u32) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.equals(id))
    }

    /// add task
    fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// remove task by id
    fn remove_by_id(&mut self, id: u32) {
        self.tasks.retain(|task| !task.equals(id));
    }

    /// clear all tasks
    fn clear(&mut self) {
        self.tasks.retain(|_| false);
    }

    /// is empty
    fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// generate task_id
    fn generate_task_id(&mut self) -> u32 {
        self.id_counter += 1;
        self.id_counter
    }
}

#[derive(Default)]
struct TaskManager {
    content: String,
    content_input_state: text_input::State,
    tasks: Tasks,
    repository: TaskRepository,
    add_button: button::State,
    clear_button: button::State,
    scroll: scrollable::State,
}

#[derive(Debug, Clone)]
enum Message {
    AddTask,
    ClearTask,
    ChangeStatusTask(u32),
    DeleteTask(u32),
    InputChangedTaskContent(String),
}

impl Sandbox for TaskManager {
    type Message = Message;

    fn new() -> TaskManager {
        let repository = TaskRepository::new(SAVE_FILENAME.to_string());

        TaskManager {
            content: "".to_string(),
            content_input_state: text_input::State::new(),
            tasks: repository.load(),
            repository: repository,
            add_button: button::State::new(),
            clear_button: button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Tasking!")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AddTask => {
                if self.content.len() > 0 {
                    let id = self.tasks.generate_task_id();
                    self.tasks.add(
                        Task::create(
                            id,
                        self.content.to_string()
                        )
                    );

                    self.content = "".to_string();

                    self.repository.save(&self.tasks);
                }
            }
            Message::ClearTask => {
                self.tasks.clear();
                self.repository.save(&self.tasks);
            },
            Message::ChangeStatusTask(id) => {
                if let Some(task) = self.tasks.find_by_id(id) {
                    task.change_status(task.status.next_status());
                    self.repository.save(&self.tasks);
                }
            }
            Message::DeleteTask(id) => {
                self.tasks.remove_by_id(id);
                self.repository.save(&self.tasks);
            }
            Message::InputChangedTaskContent(input) => {
                self.content = input;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let mut column = Column::new()
            .padding(20)
            .align_items(Align::Start)
            .push(
                Row::new()
                    .push(
                        TextInput::new(
                            &mut self.content_input_state,
                            "新しいタスクを入力してください",
                            self.content.as_str(),
                            Message::InputChangedTaskContent,
                        )
                            .padding(10)
                            .size(20)
                    )
                    .push(
                        Button::new(&mut self.add_button, Text::new("追加"))
                            .padding(10)
                            .style(style::Button::Primary)
                            .on_press(Message::AddTask),
                    )
                    .spacing(10),
            );

        if !self.tasks.is_empty() {
            let elements = self.tasks
                .tasks
                .iter_mut()
                .fold(
                    Column::new(),
                    |col, task| {
                        col.push(task.view()).spacing(10)
                    }
                );
            column = column
            .push(
                Scrollable::new(&mut self.scroll)
                    .height(Length::Fill)
                    .padding(10)
                    .scroller_width(5)
                    .push(elements)
            )
            .push(
                Button::new(
                    &mut self.clear_button,
                    Text::new("クリア")
                            .width(Length::Fill)
                            .horizontal_alignment(HorizontalAlignment::Center)
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .style(style::Button::Secondary)
                    .on_press(Message::ClearTask),
            );
        } else {
            column = column
            .push(
            Text::new("タスクがありません")
                .width(Length::Fill)
                .horizontal_alignment(HorizontalAlignment::Center)
            );
        }

        Container::new(
        column.spacing(20)
        )
            .into()
    }
}

/// task json repository
#[derive(Default)]
struct TaskRepository {
    filename: String,
}

impl TaskRepository {
    /// construct
    fn new(filename: String) -> Self {

        let path_buf = env::current_exe().expect("カレントパスを取得できませんでした");
        println!("{}", path_buf.display());
        if let Some(path) = path_buf.parent() {
            let pb = path.join(filename.clone());
            if let Some(p) = pb.to_str() {
                println!("{}", p.to_string());
                return Self {
                    filename: p.to_string(),
                }
            }
        }

        Self {
            filename: filename,
        }
    }

    /// save tasks
    fn save(&self, tasks: &Tasks) {
        let path = Path::new(self.filename.as_str());
        let serialized = serde_json::to_string(&tasks).expect("シリアライズできませんでした");
        let mut file = fs::File::create(path).expect(format!("{} ファイルが開けませんでした", path.display()).as_str());
        writeln!(file, "{}", serialized).expect("ファイルに書き込めませんでした");
    }

    /// load tasks from json
    fn load(&self) -> Tasks {
        if let Ok(serialized) = fs::read_to_string(Path::new(self.filename.as_str())) {
            if let Ok(tasks) = serde_json::from_str::<Tasks>(&serialized) {
                return tasks
            }
        }

        Tasks::new()
    }
}

mod style {
    use iced::{button, Background, Color, Vector};
    use super::TaskStatus;

    pub enum Button {
        Primary,
        Secondary,
        Warning,
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Button::Primary => Color::from_rgb8(153,50,204),
                    Button::Secondary => Color::from_rgb8(119,136,153),
                    Button::Warning => Color::from_rgb8(154,205,50),
                })),
                border_radius: 4.0,
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }

    pub enum StatusButton {
        New,
        Progress,
        Stop,
        Done,
    }

    impl button::StyleSheet for StatusButton {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(match self {
                    Self::New => Color::from_rgb8(0,255,255),
                    Self::Progress => Color::from_rgb8(	0,0,255),
                    Self::Stop => Color::from_rgb8(	169,169,169),
                    Self::Done => Color::from_rgb8(177,53,70),
                })),
                border_radius: 4.0,
                text_color: match self {
                    StatusButton::New => Color::BLACK,
                    _ => Color::WHITE,
                },
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                ..self.active()
            }
        }
    }

    impl StatusButton {
        pub fn to_status_button(status: &TaskStatus) -> StatusButton {
            match status {
                TaskStatus::New => Self::New,
                TaskStatus::Progress => Self::Progress,
                TaskStatus::Stop => Self::Stop,
                TaskStatus::Done => Self::Done,
            }
        }
    }
}
