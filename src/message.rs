use std::fmt::Debug;

use crate::config::NgramMethod;

#[derive(Clone, Debug)]
pub enum ModelAction {
    Name(String),
}

#[derive(Clone, Debug)]
pub enum Message {
    ModelSelected(String),
    ModelNew,
    ModelSave,
    ModelRemove,

    ModelNameChange(String),
    ModelCtxSizeChangePlus,
    ModelCtxSizeChangeMinus,
    CtxChanged(u32),
    ModelFileSelect,
    ModelFileSelected(String),
    ModelMmprojSelect,
    ModelMmprojSelected(String),

    Temp(bool),
    TempV(String),

    Advanced(bool),

    FlashAttn(bool),
    FlashAttnV(bool),

    NCpuMoe(bool),
    NCpuMoeV(String),

    CacheTypeK(crate::config::CacheType),
    CacheTypeV(crate::config::CacheType),

    Reasoning(bool),
    ReasoningV(bool),

    ReasoningBudget(bool),
    ReasoningBudgetV(String),

    PresencePenalty(bool),
    PresencePenaltyV(String),

    RepeatPenalty(bool),
    RepeatPenaltyV(String),

    Jinja(bool),
    JinjaV(bool),

    Port(String),

    Threads(bool),
    ThreadsV(String),

    SaveChanges,
    ShowModal(String),

    ApiKey(bool),
    ApiKeyV(String),

    GpuLayers(bool),
    GpuLayersV(String),

    BatchSizeV(u16),
    UBatchSizeV(u16),

    // If value is empty, None is to be written
    JsonSchemaFileSelect,
    JsonSchemaFileSelected(String),
    ChatTemplateFileSelect,
    ChatTemplateFileSelected(String),

    Host(bool),
    HostV(String),

    LogFile(bool),
    LogFileV(String),

    LogTimestamps(bool),
    LogTimestampsV(bool),

    Mmap(bool),

    TopP(bool),
    TopPV(String),
    TopK(bool),
    TopKV(String),
    MinP(bool),
    MinPV(String),

    ChatKwargsV(String),

    Void,

    LlamaStart,
    LlamaStop,
    LlamaStatus(crate::sipper::LlamaEvent),

    Language(String),

    SpecTypeV(NgramMethod),
    SpecTypeValue(NgramMethod, bool),

    Theme(iced::Theme),
    Tools(bool),
    Special(bool),

    SpecDraftNMax(bool),
    SpecDraftNMaxV(String),
    SpecDraftNMin(bool),
    SpecDraftNMinV(String),
    SpecDraftPMin(bool),
    SpecDraftPMinV(String),
    ModelDraft(bool),
    ModelDraftV(String),
    ModelDraftSelect,

    BeforeExit,
    Exit,
}
