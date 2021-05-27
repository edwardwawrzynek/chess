use crate::error::Error;
use crate::apikey::ApiKey;
use crate::models::UserId;

/// A command that can be sent from server to client
pub enum ServerCommand<'a> {
    /// Report an error to the client
    Error(Error),
    /// Report the current user's newly generated api key
    GenApikey(ApiKey),
    /// Report information for a particular user
    UserInfo { id: UserId, name: &'a str, email: &'a str}
}

/// A command sent to the server from the client
pub enum ClientCommand {
    /// Create a new user with login credentials and login
    NewUser { name: String, email: String, password: String },
    /// Create a new user without login credentials and login
    NewTmpUser { name: String },
    /// Login with an apikey
    Apikey(ApiKey),
    /// Login with an email and password
    Login { email: String, password: String },
    /// Set the current user's name
    Name(String),
    /// Set the current user's password
    Password(String),
    /// Generate an apikey for the current user (ServerCommand::GenApiKey response)
    GenApikey,
    /// Get info on the current user (ServerCommand::UserInfo response)
    SelfUserInfo,
    /// Get info on user with given id (ServerCommand::UserInfo response)
    UserInfo(UserId),
}