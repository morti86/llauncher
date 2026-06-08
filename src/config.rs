#![allow(non_camel_case_types)]
use std::collections::BTreeMap;
use std::fmt::Display;
use iced::{Alignment, Font};
use iced::widget::{Column, Scrollable, slider, button, checkbox, column, container, pick_list, row, scrollable, text, text_input, toggler};
use serde::{Serialize, Deserialize};
use tracing::debug;
use crate::{make_enum, message::Message, button_nf};

const LB_WIDTH: f32 = 300.0;
const PADDING: f32 = 5.0;
const SPACING: f32 = 2.0;
const HEIGHT: f32 = 38.0;
pub const W_WIDTH: f32 = 1000.0;
pub const W_HEIGHT: f32 = 940.0;
const BATCH_SIZES: &[u16] = &[128, 256, 512, 1024, 2048, 4096];

macro_rules! label {
    ($txt:expr) => {
        text(t!($txt)).width(LB_WIDTH)
    };
}

macro_rules! p_row {
    ($($element:expr),+) => {
        row![ $($element,)+ ].height(HEIGHT).padding(PADDING).spacing(SPACING)
    };
}

macro_rules! option_bool {
    ($label:expr, $value:expr, $msg:expr, $msgv:expr) => {
        row![
            checkbox($value.is_some()).label(t!($label)).width(LB_WIDTH).on_toggle($msg), 
            if let Some(v) = $value { toggler(v).on_toggle($msgv) } else { toggler(false) }
        ].height(HEIGHT).align_y(Alignment::Center).padding(PADDING).spacing(SPACING)
    };
}

macro_rules! option {
    ($label:expr, $value:expr, $msg:expr, $msgv:expr) => {
        row![
            checkbox($value.is_some()).label(t!($label)).width(LB_WIDTH).on_toggle($msg), 
            if let Some(v) = $value { text_input("", &format!("{}",v)).width(100.0).on_input($msgv) } else { text_input("","").width(100.0) }
        ].height(HEIGHT).padding(PADDING).spacing(SPACING)
    };
    ($label:expr, $value:expr, $msg:expr, $msgv:expr, $w:expr) => {
        row![
            checkbox($value.is_some()).label(t!($label)).width(LB_WIDTH).on_toggle($msg), 
            if let Some(v) = $value { text_input("", &format!("{}",v)).width($w).on_input($msgv) } else { text_input("","").width($w) }
        ].height(HEIGHT).padding(PADDING).spacing(SPACING)
    };
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Window {
    pub width: f32,
    pub height: f32,
    pub theme: String,
    pub lang: String,
}

impl Window {
    fn new() -> Self {
        Self { 
            width: 500.0, 
            height: 400.0, 
            theme: String::from("Dark"),
            lang: String::from("EN_US"),
        }
    }
}

make_enum!(CacheType, [f32, f16, bf16, q8_0, q4_0, q4_1, iq4_nl, q5_0, q5_1, turbo2, turbo3, turbo4, iso3, planar3]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NgramMethod {
    None,
    NgramCache,
    NgramSimple,
    NgramMapK,
    NgramMapK4V,
    NgramMod,
    Mtp,
    Eagle3,
}

impl NgramMethod {
    const ALL: &[NgramMethod] = &[NgramMethod::None,NgramMethod::NgramCache,NgramMethod::NgramSimple,NgramMethod::NgramMapK,NgramMethod::NgramMapK4V,NgramMethod::NgramMod,NgramMethod::Mtp,NgramMethod::Eagle3];
}

impl Display for NgramMethod {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NgramMethod::None => "",
            NgramMethod::NgramCache => "ngram-cache",
            NgramMethod::NgramSimple => "ngram-simple",
            NgramMethod::NgramMapK => "ngram-map-k",
            NgramMethod::NgramMapK4V => "ngram-map-k4v",
            NgramMethod::NgramMod => "ngram-mod",
            NgramMethod::Eagle3 => "draft-eagle3",
            NgramMethod::Mtp => "draft-mtp",
        };
        write!(f, "{}", s)
    }
}

fn obool(value: Option<bool>) -> String {
    match value {
        Some(true) => String::from("true"),
        Some(false) => String::from("false"),
        None => String::from("auto"),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct LlamaConfig {
    pub name: String,
    pub cache_type_k: Option<CacheType>, 
    pub cache_type_v: Option<CacheType>, 
    pub flash_attn: Option<bool>, 
    pub ctx_size: u32, 
    pub n_cpu_moe: Option<u16>, 
    pub reasoning_budget: Option<u32>, 
    pub temperature: Option<String>,

    pub port: u16,
    pub host: Option<String>,

    pub top_p: Option<String>,
    pub top_k: Option<String>,
    pub min_p: Option<String>,

    pub presence_penalty: Option<String>,
    pub repeat_penalty: Option<String>,

    pub model_path: String,
    pub mmproj: Option<String>,

    pub jinja: Option<bool>,
    pub reasoning: Option<bool>,
    pub threads: Option<usize>,
    pub api_key: Option<String>,
    pub mmap: bool,
    pub gpu_layers: Option<u16>,

    pub batch_size: Option<u16>,
    pub ubatch_size: Option<u16>,

    pub json_schema_file: Option<String>,
    pub chat_template: Option<String>,
    pub log_file: Option<String>,
    pub log_timestamps: Option<bool>,
    pub chat_template_kwargs: Option<String>,

    pub tools: Option<bool>,
    pub special: Option<bool>,
    pub spec_type: Option<NgramMethod>,
    pub spec_draft_n_max: Option<u16>,
    pub spec_draft_n_min: Option<u16>,

    pub vram: Option<u16>,
}

impl Display for LlamaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LlamaConfig {
    pub fn new() -> Self {
        debug!("New LlamaConfig");
        let threads = Some(num_cpus::get_physical());
        Self { 
            ctx_size: 4, 
            port: 8080, 
            mmap: true,
            threads,
            ..Default::default()
        }

    }

    pub fn add_ctx(&mut self) {
        self.ctx_size = (self.ctx_size + 4096).min(262144);
    }
    pub fn sub_ctx(&mut self) {
        self.ctx_size = (self.ctx_size - 4096).max(4096);
    }

    fn advanced<'a>(&'a self) -> Scrollable<'a, Message> {
        let contents = column![
            option!("temperature", self.temperature.as_ref(), Message::Temp, Message::TempV),
            option_bool!("flash_attn", self.flash_attn, Message::FlashAttn, Message::FlashAttnV),
            p_row![label!("cache_type_k"), pick_list(CacheType::ALL, self.cache_type_k, Message::CacheTypeK)],
            p_row![label!("cache_type_v"), pick_list(CacheType::ALL, self.cache_type_v, Message::CacheTypeV)],
            option!("n_cpu_moe", self.n_cpu_moe, Message::NCpuMoe, Message::NCpuMoeV),
            option!("top_p", self.top_p.as_ref(), Message::TopP, Message::TopPV),
            option!("top_k", self.top_k.as_ref(), Message::TopK, Message::TopKV),
            option!("min_p", self.min_p.as_ref(), Message::MinP, Message::MinPV),
            option_bool!("reasoning", self.reasoning, Message::Reasoning, Message::ReasoningV),
            option!("reasoning_budget", self.reasoning_budget, Message::ReasoningBudget, Message::ReasoningBudgetV),
            option!("host", self.host.as_ref(), Message::Host, Message::HostV),
            p_row![label!("port"), text_input("", &self.port.to_string()).on_input(Message::Port).width(100.0) ],
            option!("presence_penalty", self.presence_penalty.as_ref(), Message::PresencePenalty, Message::PresencePenaltyV),
            option!("repeat_penalty", self.repeat_penalty.as_ref(), Message::RepeatPenalty, Message::RepeatPenaltyV),
            option_bool!("jinja", self.jinja, Message::Jinja, Message::JinjaV),
            option!("threads", self.threads, Message::Threads, Message::ThreadsV),
            option!("api_key", self.api_key.as_ref(), Message::ApiKey, Message::ApiKeyV, 300.0),
            p_row![text("").width(LB_WIDTH) ,toggler(self.mmap).on_toggle(Message::Mmap).label(t!("mmap")) ],
            option!("gpu_layers", self.gpu_layers, Message::GpuLayers, Message::GpuLayersV),
            p_row![label!("batch_size"), pick_list(BATCH_SIZES, self.batch_size, Message::BatchSizeV)],
            p_row![label!("ubatch_size"), pick_list(BATCH_SIZES, self.ubatch_size, Message::UBatchSizeV)],
            p_row![label!("chat_template_kwargs"), text_input("", &self.chat_template_kwargs.clone().unwrap_or_default()).width(350.0).on_input(Message::ChatKwargsV) ],
            p_row![label!("tools"), toggler(self.tools.unwrap_or_default()).on_toggle(Message::Tools)],
            p_row![label!("special"), toggler(self.special.unwrap_or_default()).on_toggle(Message::Special)],
            p_row![label!("spec_type"), pick_list(NgramMethod::ALL, self.spec_type, Message::SpecTypeV)],
            option!("spec_draft_n_max", self.spec_draft_n_max, Message::SpecDraftNMax, Message::SpecDraftNMaxV ),
            option!("spec_draft_n_min", self.spec_draft_n_min, Message::SpecDraftNMin, Message::SpecDraftNMinV ),
            option!["log_file", self.log_file.as_ref(), Message::LogFile, Message::LogFileV, 300],
        ].padding(PADDING).spacing(10.0);
        scrollable(contents).width(900.0).height(390.0)
    }

    
    pub fn element<'a>(&'a self, advanced: bool) -> Column<'a, Message> {
        let mmproj = self.mmproj.clone().unwrap_or_default();
        column![
            row![
                label!("name"),
                text_input("", &self.name).width(300.0).on_input(Message::ModelNameChange) 
            ].padding(PADDING).spacing(SPACING),
            row![
                label!("ctx_size"),
                text_input("", &(self.ctx_size*1024).to_string()).width(100.0), 
                slider(0..=256, self.ctx_size, Message::CtxChanged).width(250.0),
            ].padding(PADDING).spacing(SPACING),
            row![
                label!("sel_model"), text_input("", &self.model_path).width(550.0), button_nf!("\u{f4d4}").on_press(Message::ModelFileSelect)
            ].padding(PADDING).spacing(SPACING),
            row![
                label!("mmproj"), text_input("", &mmproj).width(550.0), button_nf!("\u{f4d4}").on_press(Message::ModelMmprojSelect)
            ].padding(PADDING).spacing(SPACING),
            row![
                label!("json_schema"), text_input("", &self.json_schema_file.clone().unwrap_or_default()).width(550.0), button_nf!("\u{f4d4}").on_press(Message::JsonSchemaFileSelect)
            ].padding(PADDING).spacing(SPACING),
            row![
                label!("chat_template"), text_input("", &self.chat_template.clone().unwrap_or_default()).width(550.0), button_nf!("\u{f4d4}").on_press(Message::ChatTemplateFileSelect)
            ].padding(PADDING).spacing(SPACING),
            row![label!("advanced"), toggler(advanced).on_toggle(Message::Advanced)].padding(PADDING).spacing(SPACING),
            row![
                if advanced { self.advanced() } else { scrollable(text("")) }
            ].padding(PADDING).spacing(SPACING)
        ].padding(PADDING).spacing(SPACING)
    }

    pub fn llama_command(&self) -> Option<tokio::process::Command> {
        let path = if let Some(path) = which::which("llama-server").ok() {
            path
        } else {
            return None;
        };
        let mut cmd = tokio::process::Command::new(path);

        cmd.arg("-m").arg(&self.model_path);

        if let Some(ref mmproj) = self.mmproj {
            cmd.arg("--mmproj").arg(mmproj);
        }

        cmd.arg("-c").arg((self.ctx_size*1024).to_string());

        if let Some(ref host) = self.host
            && host.parse::<std::net::IpAddr>().is_ok() {
            cmd.arg("--host").arg(host);
        }

        cmd.arg("--port").arg(self.port.to_string());

        if let Some(ref api_key) = self.api_key {
            cmd.arg("--api-key").arg(api_key);
        }

        if let Some(ref cache_type_k) = self.cache_type_k {
            cmd.arg("--cache-type-k").arg(cache_type_k.to_string());
        }

        if let Some(ref cache_type_v) = self.cache_type_v {
            cmd.arg("--cache-type-v").arg(cache_type_v.to_string());
        }

        if let Some(flash_attn) = self.flash_attn {
            if flash_attn {
                cmd.arg("--flash-attn").arg(if flash_attn {"on"} else {"off"});
            }
        }
        if let Some(ref n_cpu_moe) = self.n_cpu_moe {
            cmd.arg("--n-cpu-moe").arg(n_cpu_moe.to_string());
        }

        if let Some(ref reasoning_budget) = self.reasoning_budget {
            cmd.arg("--reasoning-budget").arg(reasoning_budget.to_string());
        }

        if let Some(ref reasoning) = self.reasoning {
            if *reasoning {
                cmd.arg("--reasoning");
            }
        }
        if let Some(ref threads) = self.threads {
            cmd.arg("--threads").arg(threads.to_string());
        }

        if let Some(ref gpu_layers) = self.gpu_layers {
            cmd.arg("-ngl").arg(gpu_layers.to_string());
        }
        if let Some(ref temperature) = self.temperature {
            cmd.arg("--temperature").arg(temperature);
        }

        if let Some(ref batch_size) = self.batch_size {
            cmd.arg("-b").arg(batch_size.to_string());
        }

        if let Some(ref ubatch_size) = self.ubatch_size {
            cmd.arg("-ub").arg(ubatch_size.to_string());
        }

        if let Some(ref presence_penalty) = self.presence_penalty {
            cmd.arg("--presence-penalty").arg(presence_penalty.to_string());
        }
        if let Some(ref repeat_penalty) = self.repeat_penalty {
            cmd.arg("--repeat-penalty").arg(repeat_penalty.to_string());
        }

        if let Some(ref jinja) = self.jinja {
            if *jinja {
                cmd.arg("--jinja");
            }
        }

        if let Some(ref json_schema_file) = self.json_schema_file {
            cmd.arg("--json-schema-file").arg(json_schema_file);
        }

        if let Some(ref chat_template) = self.chat_template {
            cmd.arg("--chat-template-file").arg(chat_template);
        }

        if let Some(ref log_file) = self.log_file {
            cmd.arg("--log-file").arg(log_file);
        }

        if let Some(ref log_timestamps) = self.log_timestamps {
            if *log_timestamps {
                cmd.arg("--log-timestamps");
            }
        }
        if let Some(ref top_p) = self.top_p {
            cmd.arg("--top-p").arg(top_p);
        }
        if let Some(ref top_k) = self.top_k {
            cmd.arg("--top-k").arg(top_k);
        }
        if let Some(ref min_p) = self.min_p {
            cmd.arg("--min-p").arg(min_p);
        }
        if let Some(ref chat_template_kwargs) = self.chat_template_kwargs {
            cmd.arg("--chat-template-kwargs").arg(chat_template_kwargs);
        }
        if let Some(true) = self.tools {
            cmd.arg("--tools").arg("all");
        }
        if let Some(true) = self.special {
            cmd.arg("--special");
        }
        if let Some(sd) = self.spec_draft_n_max {
            cmd.arg("--spec-draft-n-max").arg(sd.to_string());
        }
        if let Some(sd) = self.spec_draft_n_min {
            cmd.arg("--spec-draft-n-min").arg(sd.to_string());
        }

        match self.spec_type {
            None | Some(NgramMethod::None) => {}
            Some(ng) => {
                cmd.arg("--spec-type").arg(ng.to_string());
            }
        }
        if !self.mmap {
            cmd.arg("--no-mmap");
        }

        Some(cmd)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    pub selection: Option<String>,
    configs: BTreeMap<String, LlamaConfig>,
    win: Window,

    pub advanced: bool,

    llama_server: Option<String>,

    #[serde(skip)]
    t_conf: Option<LlamaConfig>,
}

impl Config {
    pub fn new() -> Self {
        Self { selection: None, configs: BTreeMap::new(), win: Window::new(), advanced: false, t_conf: None, llama_server: None }
    }

    pub fn height(&self) -> f32 {
        match self.advanced {
            true => self.win.height + 400.0,
            false => self.win.height,
        }
    }

    pub fn lang(&self) -> String {
        self.win.lang.clone()
    }

    pub fn theme(&self) -> String {
        self.win.theme.clone()
    }

    pub fn set_theme(&mut self, theme: iced::Theme) {
        self.win.theme = theme.to_string();
    }

    fn t_theme(&self) -> Option<&iced::Theme> {
        iced::Theme::ALL.iter().find(|t| t.to_string() == self.win.theme)
    }

    pub fn element(&self) -> Column<'_, Message> {
        let r = rust_i18n::available_locales!().iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let idc_lang = pick_list(r, Some(self.win.lang.clone()), Message::Language).text_shaping(text::Shaping::Advanced).width(200.0);
        let idr_lang = p_row![label!("language"), idc_lang];
        column![
            p_row![text("llauncher")].align_y(Alignment::Center),
            idr_lang,
            p_row![label!("theme").width(LB_WIDTH), pick_list(iced::Theme::ALL, self.t_theme(), Message::Theme).width(200.0)],
            container(
                row![
                    pick_list(self.configs.keys().cloned().collect::<Vec<_>>(), self.selection.clone(), Message::ModelSelected).width(300.0), 
                    button_nf!("\u{f067}").on_press(Message::ModelNew), 
                    button_nf!("\u{f068}").on_press(Message::ModelRemove),
                    button_nf!("\u{f0193}").on_press_maybe(
                        if let Some(t) = self.t_conf.as_ref() && t.name.trim().is_empty() { None } else { Some(Message::SaveChanges) }),
                ].padding(PADDING).spacing(SPACING)
            ).center_x(iced::Length::Fill),
            if let Some(t_conf) = &self.t_conf {
                t_conf.element(self.advanced)
            } else if let Some(selection) = &self.selection {
                self.configs.get(selection).map(|c| c.element(self.advanced)).unwrap_or_else(|| column![])
            } else {
                column![]
            }
        ].align_x(Alignment::Center).padding(PADDING).spacing(SPACING)
    }

    pub fn new_config(&mut self) {
        debug!("Creating config!");
        self.t_conf = Some(LlamaConfig::new());
    }

    pub fn save_config(&mut self) {
        if let Some(t) = self.t_conf.take() {
            self.configs.insert(t.name.clone(), t);
        }
    }

    pub fn set_lang(&mut self, lang: &str) {
        self.win.lang = lang.to_string();
        rust_i18n::set_locale(lang);
    }

    pub fn get_config(&self) -> Option<&LlamaConfig> {
        if self.t_conf.is_some() {
            self.t_conf.as_ref()
        } else {
            self.selection.as_ref().map(|key| self.configs.get(key) ).flatten()
        }
    }

    pub fn get_config_mut(&mut self) -> Option<&mut LlamaConfig> {
        if self.t_conf.is_some() {
            self.t_conf.as_mut()
        } else {
            self.selection.as_ref().map(|key| self.configs.get_mut(key)).flatten()
        }
    }

    pub fn select_config(&mut self, name: String) {
        self.selection = Some(name);
    }

    pub fn remove_config(&mut self) {
        if let Some(key) = self.selection.clone() {
            self.configs.remove(&key);
            self.selection = None;
        }
        self.t_conf = None;
    }
}
