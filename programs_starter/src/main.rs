use iced::{
    Element,
    Length::Fill,
    Task,
    widget::{button, checkbox, column, container, row, scrollable, text, text_input},
};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
struct AppLauncher {
    applications: Vec<AppEntry>,
    input_path: String,
    is_launching: bool,
}

#[derive(Debug, Clone)]
struct AppEntry {
    path: String,
    selected: bool,
}

#[derive(Debug, Clone)]
enum Message {
    AddApplication,
    RemoveApplication(usize),
    OpenFileDialog(usize),
    FileSelected(usize, Option<PathBuf>),
    ToggleCheckbox(usize),
    LaunchSelected,
    LaunchCompleted,
}

impl AppLauncher {
    fn new() -> (Self, Task<Message>) {
        (
            AppLauncher {
                applications: vec![
                    AppEntry {
                        path: "select an app".to_string(),
                        selected: false,
                    },
                ],
                input_path: String::new(),
                is_launching: false,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        "Application Launcher".to_string()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddApplication => {
                self.applications.push(AppEntry {
                    path: self.input_path.clone(),
                    selected: false,
                });
                self.input_path.clear();

                Task::none()
            }
            Message::RemoveApplication(index) => {
                if index < self.applications.len() {
                    self.applications.remove(index);
                }
                Task::none()
            }
            Message::OpenFileDialog(index) => Task::perform(open_file_dialog(), move |path| {
                Message::FileSelected(index, path)
            }),
            Message::FileSelected(index, path) => {
                if let (Some(path), Some(app)) = (path, self.applications.get_mut(index)) {
                    app.path = path.to_string_lossy().to_string();
                }
                Task::none()
            }
            Message::ToggleCheckbox(index) => {
                if let Some(app) = self.applications.get_mut(index) {
                    app.selected = !app.selected;
                }
                Task::none()
            }
            Message::LaunchSelected => {
                if !self.is_launching {
                    self.is_launching = true;
                    let selected_apps: Vec<String> = self
                        .applications
                        .iter()
                        .filter(|app| app.selected)
                        .map(|app| app.path.clone())
                        .collect();

                    Task::perform(launch_applications(selected_apps), |_| {
                        Message::LaunchCompleted
                    })
                } else {
                    Task::none()
                }
            }
            Message::LaunchCompleted => {
                self.is_launching = false;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("Application Launcher").size(24).width(Fill);

        let mut app_list = column![];

        for (index, app) in self.applications.iter().enumerate() {
            let app_row = row![
                text_input("select an app", &app.path).width(Fill).size(16),
                checkbox("", app.selected)
                    .on_toggle(move |_| Message::ToggleCheckbox(index))
                    .size(16),
                button(text("Select App")).on_press(Message::OpenFileDialog(index)),
                button(text("Delete")).on_press(Message::RemoveApplication(index)),
            ]
            .spacing(10)
            .padding(5);

            app_list = app_list.push(app_row);
        }

        let control_buttons = row![
            text("").width(Fill),
            button(text("Add Program"))
                .on_press(Message::AddApplication),
            button(text(if self.is_launching {
                "Launching..."
            } else {
                "Launch"
            }))
            .on_press(Message::LaunchSelected),
        ]
        .spacing(10)
        .padding(10);

        let content = column![
            title,
            scrollable(app_list).height(Fill),
            control_buttons,
        ]
        .spacing(10)
        .padding(20);

        container(content).width(Fill).height(Fill).into()
    }
}

async fn open_file_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_title("Select Application")
        .add_filter("Applications", &["app", "exe", "dmg"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

async fn launch_applications(apps: Vec<String>) -> Result<(), String> {
    for app_path in apps {
        println!("Launching application: {}", app_path);

        // Try to launch application on macOS
        let result = if app_path.ends_with(".app") {
            // macOS application bundle
            Command::new("open").arg(&app_path).spawn()
        } else {
            // Executable file
            Command::new(&app_path).spawn()
        };

        match result {
            Ok(_) => {
                println!("Successfully launched: {}", app_path);
                // Wait one second before launching next application
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                eprintln!("Failed to launch {}: {}", app_path, e);
            }
        }
    }
    Ok(())
}

fn main() -> iced::Result {
    iced::application(
        "Application Launcher",
        AppLauncher::update,
        AppLauncher::view,
    )
    .run_with(AppLauncher::new)
}
