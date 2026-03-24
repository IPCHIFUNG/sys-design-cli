use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("YAML 解析错误: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON 序列化错误: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("文件 I/O 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("验证失败: {0}")]
    Validation(String),

    #[error("元素未找到: {0}")]
    ElementNotFound(String),

    #[error("元素已存在: {0}")]
    ElementAlreadyExists(String),

    #[error("无效的操作: {0}")]
    InvalidOperation(String),

    #[error("配置错误: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
