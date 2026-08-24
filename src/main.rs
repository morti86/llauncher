#![windows_subsystem = "windows"]
#![allow(dead_code)]

use std::{path::PathBuf, sync::Arc};
#[cfg(debug_assertions)]
use tracing::Level;
pub use tracing::{debug, error, info, trace, warn};
#[cfg(debug_assertions)]
use tracing_subscriber::{
    filter::{LevelFilter, Targets},
    fmt,
    prelude::*,
};

use iced::{Element, Subscription, widget::container};

use crate::{config::Config, message::Message, sipper::LlamaEvent, utils::find_config_path};
use tokio::{process::Child, sync::RwLock};

use anyhow::Result;

mod utils;
mod config;
mod message;
mod sipper;

#[macro_use]
extern crate rust_i18n;

i18n!("locales");

const FONT: &[u8] = include_bytes!("../SymbolsNerdFont-Regular.ttf");

macro_rules! set_num {
    ($s:expr, $value:expr, $t:ty) => {
        if let Ok(r) = $value.parse::<$t>() {
            $s = r;
        }
    };
    ($s:expr, $value:expr, $t:ty, $maxv:expr) => {
        if let Ok(r) = $value.parse::<$t>() 
            && r < $maxv {
            $s = r;
        }
    };
   
}

macro_rules! set_o_num {
    ($s:expr, $value:expr, $t:ty) => {
        if let Ok(r) = $value.parse::<$t>() {
            $s = Some(r);
        }
    };
    ($s:expr, $value:expr, $t:ty, $maxv:expr) => {
        if let Ok(r) = $value.parse::<$t>() 
            && r < $maxv {
            $s = Some(r);
        }
    };
}

macro_rules! set_o_float {
    ($s:expr, $value:expr) => {
        if $value.is_empty() {
            $s = None;
        } else if let Ok(r) = $value.parse::<f32>() {
            $s = Some($value);
        } else if $value.ends_with(".") 
            && let Ok(r) = $value.replace(".","").parse::<f32>() {
            $s = Some($value);
        }
    };
    ($s:expr, $value:expr, $min:expr, $max:expr) => {
        if $value.is_empty() {
            $s = None;
        } else if let Ok(r) = $value.parse::<f32>() 
            && r <= $max 
            && r >= $min {
            $s = Some($value);
        } else if $value.ends_with(".") 
            && let Ok(_) = $value.replace(".","").parse::<f32>() {
            $s = Some($value);
        }
    };

}

// Initial value of Option<T> when toggle on and off
macro_rules! set_toggle {
    ($s:ident, $value:expr, $conf:expr, $t:ty) => {
        if let Some(model) = $conf {
            match $value {
                true => model.$s = Some(<$t>::default()),
                false => model.$s = None,
            }
        }
    };
}

macro_rules! modal_err {
    ($x:expr) => {
        match $x {
            Ok(_) => Message::Void,
            Err(e) => Message::ShowModal(e.to_string())
        }
    };
}

macro_rules! modal_o_err {
    ($x:expr) => {
        match $x {
            None => Message::Void,
            Some(e) => Message::ShowModal(e.to_string())
        }
    };
}

#[derive(Default)]
pub struct App {
    conf: Config,
    child: Arc<RwLock< Option<Child> >>,
    modal: Option<String>,
    status: Option<crate::sipper::LlamaEvent>,
    sender: Option<tokio::sync::mpsc::Sender<crate::sipper::LCommand>>,
}

impl App {
    pub fn new() -> Self {
        let conf: Config = match find_config_path() {
            Some(conf_path) => {
                debug!("Config found at: {:?}", conf_path);
                match toml::from_str( std::fs::read_to_string( &conf_path ).unwrap().as_str() ) {
                    Ok(conf) => conf,
                    Err(e) => {
                        error!("Error parsing config file: {}", e);
                        if let Err(e) = std::fs::rename(&conf_path, &format!("backup_{}", conf_path.to_string_lossy())) {
                            error!("Error creating backup for file: {}", e);
                        }
                        Config::default()
                    }
                }
            }
            None => {
                let mut config_dir = dirs::config_dir()
                    .unwrap_or(PathBuf::from("./"));
                config_dir.push(crate::utils::APP_NAME);
                if let Err(e) = std::fs::create_dir_all(&config_dir) {
                    error!("Error creating config dir: {}", e);
                }
                config_dir.push(crate::utils::CONFIG_FILE);
                debug!("Creating new config at {:?}", config_dir);
                let conf = Config::new();
                match toml::to_string(&conf) {
                    Ok(s) => {
                        if let Err(e) = std::fs::write(config_dir, s) {
                            error!("Error writing to file {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error creating new config: {}", e);
                    }
                }
                conf
            }
        };

        rust_i18n::set_locale(&conf.lang().to_string());

        Self { conf, ..Default::default() }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::ALL.iter().find(|t|
            t.to_string() == self.conf.theme())
            .cloned()
            .unwrap_or(iced::Theme::Light)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let status_text = self.status.as_ref().map(|s| s.to_string());
        match &self.modal {
            Some(_) => self.modal().unwrap(),
            None => iced::widget::Column::new()
                .push(self.conf.element())
                .push(iced::widget::row![
                    iced::widget::button("▶️").on_press_maybe(
                        match self.status {
                            Some(LlamaEvent::LocalFoundNotRunning) => Some(Message::LlamaStart),
                            _ => None,
                        }),
                    iced::widget::button("⏹").on_press_maybe(
                            match self.status {
                                Some(LlamaEvent::Running) | Some(LlamaEvent::LocalFoundNotResponding) => Some(Message::LlamaStop),
                            _ => None,
                        }),
                    iced::widget::Text::new(status_text.unwrap_or_default())
                ].padding(15.0).spacing(5.0) )
                .into(),
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Language(lang) => {
                self.conf.set_lang(&lang);
            }
            Message::BeforeExit => {
                if let Some(LlamaEvent::Running) = self.status {
                    let a = self.child.clone();
                    return iced::Task::perform(async move {
                        let mut child = a.write().await;
                        if let Some(mut child) = child.take() {
                            child.kill().await
                        } else {
                            Ok(())
                        }
                    }, |r| {
                        match r {
                            Ok(_) => Message::Exit,
                            Err(e) => Message::ShowModal(e.to_string()),
                        }
                    });
                } else {
                    return iced::exit();
                }
            }
            Message::Tools(tools) => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.tools = Some(tools);
               }
            }
            Message::Exit => {
                return iced::exit();
            }
            Message::Theme(theme) => {
                self.conf.set_theme(theme);
            }
            Message::ModelNameChange(name) => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.name = name;
               }
            }
            Message::ModelSave => {
                self.conf.save_config();
            }
            Message::ModelNew => {
                self.conf.new_config();
                self.conf.selection = None;
            }
            Message::CtxChanged(ctx) => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.ctx_size = ctx;
               }
            }
            Message::ModelCtxSizeChangePlus => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.add_ctx();
               }
            }
            Message::ModelCtxSizeChangeMinus => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.sub_ctx();
               }
            }
            Message::Port(port) => {
               if let Some(model) = self.conf.get_config_mut() {
                   set_num!(model.port, port, u16);
               }
            }
            Message::Jinja(v) => {
                set_toggle!(jinja, v, self.conf.get_config_mut(), bool);
            }
            Message::JinjaV(jinja) => {
               if let Some(model) = self.conf.get_config_mut() 
                   && model.jinja.is_some() {
                   model.jinja = Some(jinja);
               }
            }
            Message::NCpuMoe(moe) => {
                set_toggle!(n_cpu_moe, moe, self.conf.get_config_mut(), u16);
            }
            Message::NCpuMoeV(moe) => {
               if let Some(model) = self.conf.get_config_mut() 
                   && model.n_cpu_moe.is_some() {
                   set_o_num!(model.n_cpu_moe, moe, u16);
               }
            }
            Message::SaveChanges => {
                self.conf.save_config();
                match toml::to_string(&self.conf) {
                    Ok(s) => {
                        return iced::Task::perform(async move {
                            tokio::fs::write(find_config_path().expect("No config file"), s).await
                        }, |r| {
                                modal_err!(r)
                        });
                    }
                    Err(e) => return modal!(e),
                }
            }
            Message::FlashAttn(f) => {
                set_toggle!(flash_attn, f, self.conf.get_config_mut(), bool);
            }
            Message::FlashAttnV(f) => {
               if let Some(model) = self.conf.get_config_mut() {
                   model.flash_attn = Some(f);
               }
            }
            Message::ReasoningBudget(rb) => {
                set_toggle!(reasoning_budget, rb, self.conf.get_config_mut(), u32);
            }
            Message::ReasoningBudgetV(rb) => {
               if let Some(model) = self.conf.get_config_mut() {
                   set_o_num!(model.reasoning_budget, rb, u32);
               }
            }
            Message::Advanced(a) => {
                self.conf.advanced = a;
            }
            Message::ModelSelected(name) => {
                self.conf.select_config(name);
            }
            Message::ModelRemove => {
                self.conf.remove_config();
                self.conf.selection = None;

            }
            Message::ModelFileSelect => {
                return iced::Task::perform(async {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("GGUF Model", &["gguf"])
                        .pick_file()
                        .await
                    {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                }, |path| {
                    if let Some(p) = path {
                        Message::ModelFileSelected(p)
                    } else {
                        Message::Void
                    }
                });
            }
            Message::ModelDraftSelect => {
                return iced::Task::perform(async {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("GGUF Model", &["gguf"])
                        .pick_file()
                        .await
                    {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                }, |path| {
                    if let Some(p) = path {
                        Message::ModelDraftV(p)
                    } else {
                        Message::Void
                    }
                });
            }

            Message::ModelMmprojSelect => {
                return iced::Task::perform(async {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("GGUF Model", &["gguf"])
                        .pick_file()
                        .await
                    {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                }, |path| {
                    if let Some(p) = path {
                        Message::ModelMmprojSelected(p)
                    } else {
                        Message::Void
                    }
                });
            }
            Message::ModelFileSelected(path) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.model_path = path;
                }
            }
            Message::ModelMmprojSelected(path) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.mmproj = Some(path);
                }
            }
            Message::JsonSchemaFileSelect => {
                return iced::Task::perform(async {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                        .await
                    {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                }, |path| {
                    if let Some(p) = path {
                        Message::JsonSchemaFileSelected(p)
                    } else {
                        Message::Void
                    }
                });
            }
            Message::ChatTemplateFileSelect => {
                return iced::Task::perform(async {
                    if let Some(path) = rfd::AsyncFileDialog::new()
                        .add_filter("Template", &["txt", "jinja"])
                        .pick_file()
                        .await
                    {
                        Some(path.path().to_string_lossy().to_string())
                    } else {
                        None
                    }
                }, |path| {
                    if let Some(p) = path {
                        Message::ChatTemplateFileSelected(p)
                    } else {
                        Message::Void
                    }
                });
            }
            Message::JsonSchemaFileSelected(path) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.json_schema_file = Some(path);
                }
            }
            Message::ChatTemplateFileSelected(path) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.chat_template = Some(path);
                }
            }
            Message::CacheTypeK(ct) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.cache_type_k = Some(ct);
                }
            }
            Message::CacheTypeV(ct) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.cache_type_v = Some(ct);
                }
            }
            Message::PresencePenalty(p) => {
                set_toggle!(presence_penalty, p, self.conf.get_config_mut(), String);
            }
            Message::PresencePenaltyV(v) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.presence_penalty, v, 0.0, 2.0);
                }
            }
            Message::Threads(t) => {
                set_toggle!(threads, t, self.conf.get_config_mut(), usize);
            }
            Message::ThreadsV(v) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.threads.is_some() {
                    set_o_num!(model.threads, v, usize);
                }
            }
            Message::ApiKey(k) => {
                set_toggle!(api_key, k, self.conf.get_config_mut(), String);
            }
            Message::ApiKeyV(v) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.api_key.is_some() {
                    model.api_key = Some(v);
                }
            }
            Message::GpuLayers(l) => {
                set_toggle!(gpu_layers, l, self.conf.get_config_mut(), u16);
            }
            Message::GpuLayersV(v) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.gpu_layers.is_some() {
                        set_o_num!(model.gpu_layers, v, u16);
                }
            }
            Message::BatchSizeV(v) => {
                if let Some(model) = self.conf.get_config_mut() {
                        model.batch_size = Some(v);
                }
            }
            Message::UBatchSizeV(v) => {
                if let Some(model) = self.conf.get_config_mut() {
                        model.ubatch_size = Some(v);
                }
            }
            Message::Host(h) => {
                set_toggle!(host, h, self.conf.get_config_mut(), String);
            }
            Message::HostV(v) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.host.is_some()
                    && !v.is_empty() {
                    model.host = Some(v);
                }
            }
            Message::LogFile(f) => {
                set_toggle!(log_file, f, self.conf.get_config_mut(), String);
            }
            Message::LogFileV(v) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.log_file.is_some() {
                    model.log_file = utils::str_to_op(v);
                }
            }
            Message::LogTimestamps(t) => {
                set_toggle!(log_timestamps, t, self.conf.get_config_mut(), bool);
            }
            Message::LogTimestampsV(t) => {
                if let Some(model) = self.conf.get_config_mut()
                    && model.log_timestamps.is_some() {
                    model.log_timestamps = Some(t);
                }
            }
            Message::Reasoning(re) => {
                set_toggle!(reasoning, re, self.conf.get_config_mut(), bool);
            }
            Message::ReasoningV(re) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.reasoning = Some(re);
                }
            }
            Message::Mmap(mmap) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.mmap = mmap;
                }
            }
            Message::TopP(tp) => {
                set_toggle!(top_p, tp, self.conf.get_config_mut(), String);
            }
            Message::TopPV(tp) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.top_p, tp, 0.0, 1.0);
                }
            } 
            Message::MinP(tp) => {
                set_toggle!(min_p, tp, self.conf.get_config_mut(), String);
            }
            Message::MinPV(tp) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.min_p, tp, 0.0, 1.0);
                }

            }
            Message::TopK(tp) => {
                set_toggle!(top_k, tp, self.conf.get_config_mut(), String);
            }
            Message::TopKV(tp) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.top_k, tp, 0.0, 100.0);
                }

            } 
            Message::RepeatPenalty(p) => {
                set_toggle!(repeat_penalty, p, self.conf.get_config_mut(), String);
            }
            Message::RepeatPenaltyV(p) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.repeat_penalty, p, 0.0, 2.0);
                }
            }
            Message::ChatKwargsV(k) => {
                if let Some(model) = self.conf.get_config_mut() {
                    if k.is_empty() {
                        model.chat_template_kwargs = None;
                    } else {
                        model.chat_template_kwargs = Some(k);
                    }
                }
            }
            Message::Temp(t) => {
                set_toggle!(temperature, t, self.conf.get_config_mut(), String);
            }
            Message::TempV(tv) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_float!(model.temperature, tv, 0.0, 1.0);
                }
            }
            Message::Special(s) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.special = if s { Some(true) } else { None };
                }
            }
            Message::SpecTypeV(_st) => {
                //if let Some(model) = self.conf.get_config_mut() {
                //    model.spec_type = Some(st);
                //}
            }
            Message::SpecTypeValue(m, v) => {
                if let Some(model) = self.conf.get_config_mut() {
                    model.spec_types.set(m, v);
                }
            }
            Message::SpecDraftNMax(sd) => {
                set_toggle!(spec_draft_n_max, sd, self.conf.get_config_mut(), u16);
            }
            Message::SpecDraftNMaxV(sd) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_num!(model.spec_draft_n_max, sd, u16);
                }
            }
            Message::SpecDraftNMin(sd) => {
                set_toggle!(spec_draft_n_min, sd, self.conf.get_config_mut(), u16);
            }
            Message::SpecDraftNMinV(sd) => {
                if let Some(model) = self.conf.get_config_mut() {
                    set_o_num!(model.spec_draft_n_min, sd, u16);
                }
            }
            Message::ShowModal(msg) => {
                self.modal = Some(msg);
            }
            Message::Void => {
                self.modal = None;
            }
            Message::LlamaStart => {
                let a = self.child.clone();
                let s = self.sender.clone();
                if let Some(conf) = self.conf.get_config()
                    && let Some(mut lc) = conf.llama_command() {
                    let host = conf.host.clone().unwrap_or(String::from("0.0.0.0"));
                    let port = conf.port;
                    return iced::Task::perform(async move {
                        let mut child = a.write().await;
                        if let Some(sender) = s {
                            let _ = sender.send(crate::sipper::LCommand::Started { host, port, command: a.clone() }).await;
                        }

                        match lc.spawn() {
                            Ok(ch) => {
                                *child = Some(ch);
                                None
                            }
                            Err(e) => {
                                Some(e)
                            }
                
                        }
                    }, |r| { modal_o_err!(r) });
                }
            }
            Message::LlamaStop => {
                let a = self.child.clone();
                let s = self.sender.clone();
                return iced::Task::perform(async move {
                    let mut child = a.write().await;
                    if let Some(mut c) = child.take() {
                        if let Some(sender) = s {
                            let _ = sender.send(crate::sipper::LCommand::Stopped).await;
                        }
                        c.kill().await
                    } else {
                        Ok(())
                    }
                }, |r| modal_err!(r));
            }
            Message::LlamaStatus(event) => {
                match &event {
                    LlamaEvent::SipperStarted(rs) => {
                        self.sender = Some(rs.clone());
                    }
                    _ => {}
                }
                self.status = Some(event);
            }
            Message::ModelDraft(m) => {
                set_toggle!(model_draft, m, self.conf.get_config_mut(), String);
            }
            Message::ModelDraftV(m) => {
                if let Some(c) = self.conf.get_config_mut() {
                    c.model_draft = Some(m);
                }
            }
            Message::SpecDraftPMin(m) => {
                set_toggle!(spec_draft_p_min, m, self.conf.get_config_mut(), String);
            }
            Message::SpecDraftPMinV(v) => {
                if let Some(c) = self.conf.get_config_mut() {
                    set_o_float!(c.spec_draft_p_min, v, 0.0, 1.0);
                }
            }
        }
        iced::Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let s_close = iced::window::close_requests().map(|_| Message::BeforeExit);
        let s_status = Subscription::run(crate::sipper::connect).map(|e| Message::LlamaStatus(e));
        let subs = [s_close, s_status];
        Subscription::batch(subs)
    }

    pub fn modal(&self) -> Option<Element<'_, Message>> {
        self.modal.as_ref().map(|msg| {
            iced::widget::Column::new()
                .push(
                    iced::widget::Container::new(iced::widget::Text::new(" "))
                        .width(iced::Length::Fill)
                        .height(iced::Length::Fill)
                        .style(|_| container::background(iced::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.7,
                        })),
                )
                .push(
                    iced::widget::Container::new(
                        iced::widget::column![
                            iced::widget::Text::new(msg.as_str()),
                            iced::widget::button("OK").on_press(Message::Void),
                        ]
                        .align_x(iced::Alignment::Center)
                        .spacing(10)
                        .padding(20),
                    )
                    .center_x(iced::Length::Fill)
                    .center_y(iced::Length::Fill)
                    .style(|_| container::background(iced::Color {
                        r: 0.15,
                        g: 0.15,
                        b: 0.15,
                        a: 1.0,
                    })),
                )
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .into()
        })
    }

}

pub fn run(theme: &str) -> Result<(), iced::Error> {
    let mut settings = iced::Settings::default();
    settings.default_text_size = iced::Pixels(14.0);
    debug!("Run with theme {}", theme);
    for e in iced::Theme::ALL {
        if theme.to_string() == e.to_string() {
            return iced::application(App::new, App::update, App::view)
                .theme(App::theme)
                .window_size(iced::Size::new(config::W_WIDTH,config::W_HEIGHT))
                .subscription(App::subscription)
                .exit_on_close_request(false)
                .settings(settings)
                .font(FONT)
                .run()
        }
    }
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .window_size(iced::Size::new(config::W_WIDTH,config::W_HEIGHT))
        .exit_on_close_request(false)
        .font(FONT)
        .settings(settings)
        .run()?;
    Ok(())
}

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    let env_rust_log = Level::DEBUG;
    
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .compact()
                .with_filter(LevelFilter::from_level(env_rust_log)),
        )
        .with(
            fmt::layer().with_writer(std::io::stdout)
        )
        .with(
            Targets::default()
            .with_target("llauncher", env_rust_log)
            .with_target("iced", Level::WARN)
        )
        .init();

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt()
        .with_thread_names(true)
        .with_max_level(tracing::Level::INFO)
        .init();

    run("Dark")?;
    Ok(())
}
