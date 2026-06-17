use std::string::FromUtf8Error;

use thiserror::Error;

/// Error returned when decompiling a Scratch project fails.
#[derive(Debug, Error)]
pub enum DecompileError {
    #[error("{context}: {source}")]
    Io {
        context: String,
        source: std::io::Error,
    },
    #[error("{context}: {source}")]
    Zip {
        context: String,
        source: zip::result::ZipError,
    },
    #[error("{context}: {source}")]
    Json {
        context: String,
        source: serde_json::Error,
    },
    #[error("invalid Scratch project: {message}")]
    InvalidScratchProject { message: String },
    #[error("{context}: {source}")]
    XmlParse {
        context: String,
        source: xmltree::ParseError,
    },
    #[error("{context}: {source}")]
    Xml {
        context: String,
        source: xmltree::Error,
    },
    #[error("{context}: {source}")]
    Utf8 {
        context: String,
        source: FromUtf8Error,
    },
    #[error("{context}: {source}")]
    Image {
        context: String,
        source: image::ImageError,
    },
}

pub(crate) fn io(context: impl Into<String>, source: std::io::Error) -> DecompileError {
    DecompileError::Io {
        context: context.into(),
        source,
    }
}

pub(crate) fn zip_error(
    context: impl Into<String>,
    source: zip::result::ZipError,
) -> DecompileError {
    DecompileError::Zip {
        context: context.into(),
        source,
    }
}

pub(crate) fn json_error(context: impl Into<String>, source: serde_json::Error) -> DecompileError {
    DecompileError::Json {
        context: context.into(),
        source,
    }
}

pub(crate) fn xml_parse_error(
    context: impl Into<String>,
    source: xmltree::ParseError,
) -> DecompileError {
    DecompileError::XmlParse {
        context: context.into(),
        source,
    }
}

pub(crate) fn xml_error(context: impl Into<String>, source: xmltree::Error) -> DecompileError {
    DecompileError::Xml {
        context: context.into(),
        source,
    }
}

pub(crate) fn utf8_error(context: impl Into<String>, source: FromUtf8Error) -> DecompileError {
    DecompileError::Utf8 {
        context: context.into(),
        source,
    }
}

pub(crate) fn image_error(context: impl Into<String>, source: image::ImageError) -> DecompileError {
    DecompileError::Image {
        context: context.into(),
        source,
    }
}

pub(crate) fn invalid_project(message: impl Into<String>) -> DecompileError {
    DecompileError::InvalidScratchProject {
        message: message.into(),
    }
}
