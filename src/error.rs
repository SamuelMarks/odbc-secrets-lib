use std::cmp::PartialEq;

#[derive(
    strum_macros::AsRefStr,
    strum_macros::Display,
    strum_macros::EnumDiscriminants,
    strum_macros::IntoStaticStr,
    Debug,
)]
#[repr(u16)]
pub enum OdbcSecretsLibError {
    #[strum(to_string = "{0}")]
    NotFound(String) = 404,

    // ************************
    // * Library level errors *
    // ************************
    #[strum(to_string = "`std::io::Error` error. {error:?}")]
    StdIoError { error: std::io::Error } = 700,

    #[strum(to_string = "`std::fmt::Error` error. {error:?}")]
    StdFmtError { error: std::fmt::Error } = 709,

    #[strum(to_string = "{0:?}")]
    ExitCode(std::process::ExitCode) = 710,

    #[strum(to_string = "`serde_json::Error` error. {error:?}")]
    SerdeJsonError { error: serde_json::Error } = 721,

    #[strum(to_string = "`csv::Error` error. {error:?}")]
    CsvError { error: csv::Error } = 740,

    #[strum(to_string = "`odbc_api::Error` error. {error:?}")]
    OdbcApiError { error: odbc_api::Error } = 741,

    #[strum(to_string = "`vaultrs::client::VaultClientSettingsBuilderError` error. {error:?}")]
    VaultClientSettingsBuilderError {
        error: vaultrs::client::VaultClientSettingsBuilderError,
    } = 742,

    #[strum(to_string = "`vaultrs::error::ClientError` error. {error:?}")]
    VaultClientError { error: vaultrs::error::ClientError } = 743,
}

impl OdbcSecretsLibError {
    fn discriminant(&self) -> u16 {
        unsafe { *(self as *const Self as *const u16) }
    }
}

impl From<std::io::Error> for OdbcSecretsLibError {
    fn from(error: std::io::Error) -> Self {
        Self::StdIoError { error }
    }
}

impl From<std::fmt::Error> for OdbcSecretsLibError {
    fn from(error: std::fmt::Error) -> Self {
        Self::StdFmtError { error }
    }
}

impl From<odbc_api::Error> for OdbcSecretsLibError {
    fn from(error: odbc_api::Error) -> Self {
        Self::OdbcApiError { error }
    }
}

impl From<csv::Error> for OdbcSecretsLibError {
    fn from(error: csv::Error) -> Self {
        Self::CsvError { error }
    }
}

impl From<vaultrs::client::VaultClientSettingsBuilderError> for OdbcSecretsLibError {
    fn from(error: vaultrs::client::VaultClientSettingsBuilderError) -> Self {
        Self::VaultClientSettingsBuilderError { error }
    }
}

impl From<vaultrs::error::ClientError> for OdbcSecretsLibError {
    fn from(error: vaultrs::error::ClientError) -> Self {
        Self::VaultClientError { error }
    }
}

impl From<serde_json::Error> for OdbcSecretsLibError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJsonError { error }
    }
}

impl From<std::process::ExitCode> for OdbcSecretsLibError {
    fn from(error: std::process::ExitCode) -> Self {
        Self::ExitCode(error)
    }
}

impl std::process::Termination for OdbcSecretsLibError {
    fn report(self) -> std::process::ExitCode {
        if let OdbcSecretsLibError::ExitCode(exit_code) = self {
            return exit_code;
        }
        let status_code = self.discriminant();
        if status_code > u8::MAX as u16 {
            eprintln!("exit code {}", status_code);
            std::process::ExitCode::FAILURE
        } else {
            std::process::ExitCode::from(status_code as u8)
        }
    }
}
