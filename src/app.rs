// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::models::gemini::{self, get_gemini_response};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Subscription, widget::column, widget::markdown, window::Id};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use cosmic::{Element, iced};
use futures_util::SinkExt;
use rdev::display_size;
use std::sync::Arc;

pub const APPID: &str = "com.github.Ignavar.cosmic-ai-interface";

pub struct Chat {
    pub role: String,
    pub content: String,
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// The popup id.
    popup: Option<Id>,
    /// Configuration data that persists between application runs.
    config: Config,
    /// Input text field.
    input_text: String,
    /// Chat history.
    chat_history: Arc<Vec<Chat>>,
    ///
    is_loading: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    SubscriptionChannel,
    UpdateConfig(Config),
    SubmitInput(String),
    InputChanged(String),
    GeminiMessage(gemini::Message),
    UrlClicked(markdown::Url),
}

impl From<gemini::Message> for Message {
    fn from(message: gemini::Message) -> Self {
        Self::GeminiMessage(message)
    }
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = APPID;

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel will be drawn using the main view method.
    /// This view should emit messages to toggle the applet's popup window, which will
    /// be drawn using the `view_window` method.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button(constcat::concat!(APPID, "-symbolic"))
            .on_press(Message::TogglePopup)
            .into()
    }

    /// The applet's popup window will be drawn using this view method. If there are
    /// multiple poups, you may match the id parameter to determine which popup to
    /// create a view for.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let (width, height) = display_size().unwrap_or((1280, 720));
        let content = widget::container(
            column!(
                self.chat_view(),
                widget::text_input("Enter text", &self.input_text)
                    .on_input(Message::InputChanged)
                    .width(cosmic::iced::Length::Fill)
                    .padding(10)
                    .on_submit(Message::SubmitInput)
            )
            .spacing(10),
        )
        .padding([18, 10]);

        self.core
            .applet
            .popup_container(content)
            .limits(
                cosmic::iced::Limits::NONE
                    .min_height(height as f32 / 1.2)
                    .min_width(width as f32 / 3.5)
                    .max_width(width as f32 / 3.5)
                    .max_height(height as f32 / 1.2),
            )
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::InputChanged(text) => {
                self.input_text = text;
            }
            Message::SubmitInput(text) => {
                if self.is_loading {
                    return Task::none();
                }
                let Some(history) = Arc::get_mut(&mut self.chat_history) else {
                    return Task::none();
                };
                self.is_loading = true;
                history.push(Chat {
                    role: "user".into(),
                    content: text.into(),
                });
                self.input_text.clear();
                let cloned = Arc::clone(&self.chat_history);
                return cosmic::task::future(async move {
                    Message::GeminiMessage(get_gemini_response(cloned).await)
                });
            }
            Message::UrlClicked(_) => {}
            Message::SubscriptionChannel => {
                // For example purposes only.
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::GeminiMessage(message) => {
                self.is_loading = false;
                let Some(history) = Arc::get_mut(&mut self.chat_history) else {
                    return Task::none();
                };
                match message {
                    gemini::Message::RequestError(error) => {
                        history.push(Chat {
                            role: "model".into(),
                            content: error,
                        });
                    }
                    gemini::Message::ApiKeyNotSet => {
                        history.push(Chat {
                            role: "model".into(),
                            content: "API key not set".into(),
                        });
                    }
                    gemini::Message::ApiResultParsingError(error) => {
                        history.push(Chat {
                            role: "model".into(),
                            content: format!("API result parsing error: {}", error),
                        });
                    }
                    gemini::Message::ApiError(error) => {
                        history.push(Chat {
                            role: "model".into(),
                            content: format!("API error: {}", error),
                        });
                    }
                    gemini::Message::EmptyResponse => {
                        history.push(Chat {
                            role: "model".into(),
                            content: "No response from model".into(),
                        });
                    }
                    gemini::Message::PromptBlocked(error) => {
                        history.push(Chat {
                            role: "model".into(),
                            content: format!("Prompt blocked: {}", error),
                        });
                    }
                    gemini::Message::Response(response) => {
                        history.push(Chat {
                            role: "model".into(),
                            content: response.into(),
                        });
                    }
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl AppModel {
    fn chat_view(&self) -> cosmic::Element<'_, Message> {
        if self.chat_history.is_empty() {
            widget::container(widget::text("Start a new Chat!"))
                .center_y(cosmic::iced::Length::Fill)
                .center_x(cosmic::iced::Length::Fill)
                .into()
        } else {
            let mut chats: Vec<cosmic::Element<_>> = Vec::with_capacity(self.chat_history.len());

            for chat in self.chat_history.iter() {
                let markdown: Vec<markdown::Item> = markdown::parse(&chat.content).collect();
                let content = markdown::view(
                    &markdown,
                    markdown::Settings::with_text_size(15),
                    markdown::Style::from_palette(iced::Theme::TokyoNight.palette()),
                )
                .map(Message::UrlClicked);
                let bubble = if chat.role == "user" {
                    widget::container(
                        widget::container(content)
                            .class(cosmic::theme::Container::List)
                            .padding(10),
                    )
                    .align_right(iced::Length::Fill)
                    .into()
                } else {
                    widget::container(
                        widget::container(content)
                            .class(cosmic::theme::Container::List)
                            .padding(10),
                    )
                    .align_left(iced::Length::Fill)
                    .into()
                };
                chats.push(bubble);
            }

            widget::container(
                widget::scrollable(widget::Column::with_children(chats).spacing(20))
                    .spacing(2)
                    .scroller_width(0)
                    .scrollbar_width(0),
            )
            .center_x(cosmic::iced::Length::Fill)
            .align_top(iced::Length::Fill)
            .into()
        }
    }
}
