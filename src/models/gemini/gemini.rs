use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub prompt_feedback: Option<PromptFeedback>,
    pub usage_meta_deta: Option<UsageMetaData>,
    pub model_version: Option<String>,
    pub response_id: Option<String>,
    pub model_status: Option<ModelStatus>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptFeedback {
    pub block_reason: Option<BlockReason>,
    pub safety_ratings: Vec<SafetyRating>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockReason {
    BlockReasonUnspecified,
    Safety,
    Other,
    BlockList,
    ProhibitedContent,
    ImageSafety,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetaData {
    pub prompt_token_count: String,
    pub thoughts_token_count: String,
    pub total_token_count: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub model_stage: ModelStage,
    pub retirement_time: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelStage {
    ModelStageUnspecified,
    Experimental,
    Preview,
    Stable,
    Legacy,
    Retired,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: Option<FinishReason>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
    pub index: u32,
    pub finish_message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FinishReason {
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Language,
    Other,
    Blocklist,
    ProhibitedContent,
    Spii,
    MalformedFunctionCall,
    ImageSafety,
    ImageProhibitedContent,
    ImageOther,
    NoImage,
    ImageRecitation,
    UnexpectedToolCall,
    TooManyToolCalls,
    MissingThoughtSignature,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: HarmProbability,
    pub blocked: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,

    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    HarmCategoryUnspecified,
    HarmCategoryDerogratory,
    HarmCategoryToxicity,
    HarmCategoryViolence,
    HarmCategroySexual,
    HarmCategoryMedical,
    HarmCategoryDangerous,
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,

    #[serde(other)]
    Unkown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub parts: Vec<Part>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub thought: Option<bool>,
    pub thought_signature: Option<String>,
    pub text: Option<String>,
    pub inline_data: Option<Blob>,
    pub file_data: Option<FileData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub mime_type: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    pub mime_type: String,
    pub file_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub status: String,
    pub details: Option<Vec<serde_json::Value>>,
}

#[derive(serde::Serialize)]
pub struct GeminiRequest<'a> {
    pub contents: Vec<GeminiContent<'a>>,
}

#[derive(serde::Serialize)]
pub struct GeminiContent<'a> {
    pub role: &'a str,
    pub parts: Vec<GeminiPart<'a>>,
}

#[derive(serde::Serialize)]
pub struct GeminiPart<'a> {
    pub text: &'a str,
}
