use bcrypt;
use diesel;
use r2d2;

use futures_channel::mpsc;
use std::fmt;
use tungstenite::protocol::Message;

#[derive(Debug)]
pub enum Error {
    DBError(diesel::result::Error),
    R2D2Error(r2d2::Error),
    BCryptError(bcrypt::BcryptError),
    NoSuchUser,
    MalformedApiKey,
    InvalidApiKey,
    IncorrectCredentials,
    EmailAlreadyTaken,
    InvalidCommand(String),
    InvalidNumberOfArguments {
        cmd: String,
        expected: usize,
        actual: usize,
    },
    NoSuchConnectedClient,
    ClientTxChannelClosed(mpsc::TrySendError<Message>),
    MessageParseError,
    NotLoggedIn,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use Error::*;
        match self {
            NoSuchUser => match other {
                NoSuchUser => true,
                _ => false,
            },
            MalformedApiKey => match other {
                MalformedApiKey => true,
                _ => false,
            },
            InvalidApiKey => match other {
                InvalidApiKey => true,
                _ => false,
            },
            IncorrectCredentials => match other {
                IncorrectCredentials => true,
                _ => false,
            },
            EmailAlreadyTaken => match other {
                EmailAlreadyTaken => true,
                _ => false,
            },
            InvalidCommand(self_cmd) => match other {
                InvalidCommand(other_cmd) => *self_cmd == *other_cmd,
                _ => false,
            },
            InvalidNumberOfArguments {
                cmd,
                expected,
                actual,
            } => match other {
                InvalidNumberOfArguments {
                    cmd: cmd_other,
                    expected: expected_other,
                    actual: actual_other,
                } => *cmd == *cmd_other && *expected == *expected_other && *actual == *actual_other,
                _ => false,
            },
            DBError(_) => match other {
                DBError(_) => true,
                _ => false,
            },
            BCryptError(_) => match other {
                BCryptError(_) => true,
                _ => false,
            },
            NoSuchConnectedClient => match other {
                NoSuchConnectedClient => true,
                _ => false,
            },
            ClientTxChannelClosed(_) => match other {
                ClientTxChannelClosed(_) => true,
                _ => false,
            },
            R2D2Error(_) => match other {
                R2D2Error(_) => true,
                _ => false,
            },
            MessageParseError => match other {
                MessageParseError => true,
                _ => false,
            },
            NotLoggedIn => match other {
                NotLoggedIn => true,
                _ => false,
            },
        }
    }
}

impl Eq for Error {}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        Error::DBError(e)
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(e: bcrypt::BcryptError) -> Error {
        Error::BCryptError(e)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Error {
        Error::R2D2Error(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            DBError(e) => write!(f, "database error: {}", *e),
            BCryptError(e) => write!(f, "bcrypt error: {}", *e),
            NoSuchUser => write!(f, "no such user"),
            MalformedApiKey => write!(f, "malformed api key"),
            InvalidApiKey => write!(f, "invalid api key"),
            IncorrectCredentials => write!(f, "incorrect login credentials"),
            EmailAlreadyTaken => write!(f, "email is already taken"),
            InvalidCommand(cmd) => write!(f, "unrecognized command: {}", cmd),
            InvalidNumberOfArguments {
                cmd,
                expected,
                actual,
            } => write!(
                f,
                "invalid number of arguments for command {} - expected {}, found {}",
                cmd, expected, actual
            ),
            NoSuchConnectedClient => write!(f, "no such connected client"),
            ClientTxChannelClosed(_) => write!(f, "client transmit channel is closed"),
            R2D2Error(_) => write!(
                f,
                "database pool error: could not establish database connection"
            ),
            MessageParseError => write!(
                f,
                "couldn't parse client command as text (make sure to use utf-8 encoded messages)"
            ),
            NotLoggedIn => write!(f, "you are not logged in"),
        }
    }
}
