use std::{
    convert::From, fmt, io, num, path::PathBuf, str::Utf8Error, string::FromUtf8Error
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(num::ParseIntError),
    InvalidFilePath(PathBuf),
    ConfigKeyDoesntExist(String),
    UnsupportedRepositoryVersion,
    RepositoryConfigurationIssue,
    NoMinitRepository,
    ObjectNotDefined(String),
    FromUtf8Error(FromUtf8Error),
    Utf8Error(Utf8Error),
    StringNotFound(String, String),
    NameNotDefined,
    AmbiguousReference(Vec<String>),
    ObjectNotFound,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // TODO: implement this!
        unimplemented!()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IoError: {:#?}", err),
            Error::Parse(err) => write!(f, "ParseIntError: {:#?}", err),
            Error::InvalidFilePath(err) => write!(f, "InvalidFilePath: {:#?}", err),
            Error::ConfigKeyDoesntExist(err) => write!(f, "ConfigKeyDoesntExist: {:#?}", err),
            Error::UnsupportedRepositoryVersion => write!(f, "UnsupportedRepositoryVersion"),
            Error::RepositoryConfigurationIssue => write!(f, "RepositoryConfigurationIssue"),
            Error::NoMinitRepository => write!(f, "NoMinitRepositoryFound"),
            Error::ObjectNotDefined(obj) => write!(f, "ObjectNotDefined: {:#?}", obj),
            Error::FromUtf8Error(err) => write!(f, "FromUtf8Error: {:#?}", err),
            Error::Utf8Error(err) => write!(f, "Utf8Error: {:#?}", err),
            Error::StringNotFound(source, needle) => write!(f, "StringNotFound: needle: {:#?} haystack: {:#?}", needle, source),
            Error::NameNotDefined => write!(f, "NameNotDefined"),
            Error::ObjectNotFound => write!(f, "ObjectNotFound"),
            Error::AmbiguousReference(v) => write!(f, "AmbiguousReference: {:#?}", v),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Error {
        Error::Io(value)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(value: num::ParseIntError) -> Error {
        Error::Parse(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Error {
        Error::FromUtf8Error(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Error {
        Error::Utf8Error(value)
    }
}
