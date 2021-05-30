use crate::apikey::ApiKey;
use crate::error::Error;
use crate::models::UserId;
use lazy_static;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

/// A command that can be sent from server to client
#[derive(PartialEq, Eq, Debug)]
pub enum ServerCommand {
    /// Report an error to the client
    Error(Error),
    /// Report the current user's newly generated api key
    GenApikey(ApiKey),
    /// Report information for the current user
    SelfUserInfo {
        id: UserId,
        name: String,
        email: Option<String>,
    },
}

/// A command sent to the server from the client
#[derive(PartialEq, Eq, Debug)]
pub enum ClientCommand<'a> {
    /// Create a new user with login credentials and login
    NewUser {
        name: &'a str,
        email: &'a str,
        password: &'a str,
    },
    /// Create a new user without login credentials and login
    NewTmpUser { name: &'a str },
    /// Login with an apikey
    Apikey(ApiKey),
    /// Login with an email and password
    Login { email: &'a str, password: &'a str },
    /// Lgout of the current session
    Logout,
    /// Set the current user's name
    Name(&'a str),
    /// Set the current user's password
    Password(&'a str),
    /// Generate an apikey for the current user (ServerCommand::GenApiKey response)
    GenApikey,
    /// Get info on the current user (ServerCommand::UserInfo response)
    SelfUserInfo,
}

impl fmt::Display for ServerCommand {
    /// Serialize the command into the textual representation expected by the client
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ServerCommand::*;
        match self {
            &Error(ref e) => write!(f, "error {}", e.to_string()),
            &GenApikey(ref key) => write!(f, "gen_apikey {}", key.to_string()),
            &SelfUserInfo {
                id,
                ref name,
                ref email,
            } => {
                let default_email = "-".to_string();
                let email_str = email.as_ref().unwrap_or_else(|| &default_email);
                write!(f, "self_user_info {}, {}, {}", id, *name, *email_str)
            }
        }
    }
}

/// Parse a command from a client into a command and arguments
fn parse_cmd(msg: &str) -> (&str, Vec<&str>) {
    let mut cmd = msg;
    let mut args = Vec::new();

    let mut cmd_end_index = msg.len();
    for (i, c) in msg.chars().enumerate() {
        if char::is_whitespace(c) {
            cmd = &msg[0..i];
            cmd_end_index = i;
            break;
        }
    }

    if cmd_end_index < msg.len() {
        for el in msg[cmd_end_index..].split(',') {
            args.push(el.trim());
        }
    }

    (cmd, args)
}

lazy_static! {
    // number of arguments expected for each command
    static ref NUM_ARGS: HashMap<&'static str, usize> = {
        let mut m = HashMap::new();
        m.insert("new_user", 3);
        m.insert("new_tmp_user", 1);
        m.insert("apikey", 1);
        m.insert("login", 2);
        m.insert("name", 1);
        m.insert("password", 1);
        m.insert("gen_apikey", 0);
        m.insert("self_user_info", 0);
        m.insert("logout", 0);
        m
    };
}

impl ClientCommand<'_> {
    /// Parse a command from the textual representation sent by a client
    pub fn deserialize(message: &str) -> Result<ClientCommand, Error> {
        use ClientCommand::*;

        let msg = message.trim();
        let (cmd, args) = parse_cmd(msg);
        // check for command existence + correct number of arguments
        let expected_args = NUM_ARGS.get(cmd);
        match expected_args {
            None => return Err(Error::InvalidCommand(cmd.to_string())),
            Some(expected) => {
                if args.len() != *expected {
                    return Err(Error::InvalidNumberOfArguments {
                        cmd: cmd.to_string(),
                        expected: *expected,
                        actual: args.len(),
                    });
                }
            }
        }
        // command is correct, so deserialize
        match cmd {
            "new_user" => Ok(NewUser {
                name: args[0],
                email: args[1],
                password: args[2],
            }),
            "new_tmp_user" => Ok(NewTmpUser { name: args[0] }),
            "apikey" => Ok(Apikey(ApiKey::try_from(args[0])?)),
            "login" => Ok(Login {
                email: args[0],
                password: args[1],
            }),
            "name" => Ok(Name(args[0])),
            "password" => Ok(Password(args[0])),
            "gen_apikey" => Ok(GenApikey),
            "self_user_info" => Ok(SelfUserInfo),
            "logout" => Ok(Logout),
            _ => Err(Error::InvalidCommand(cmd.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cmd_serialize_test() {
        assert_eq!(
            ServerCommand::SelfUserInfo {
                id: 5,
                name: "user",
                email: "sample@example.com"
            }
            .to_string(),
            "self_user_info 5, user, sample@example.com"
        );
        assert_eq!(
            ServerCommand::Error(Error::InvalidApiKey).to_string(),
            "error invalid api key"
        );
        assert_eq!(
            ServerCommand::GenApikey(
                ApiKey::try_from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                    .expect("failed to parse api key")
            )
            .to_string(),
            "gen_apikey aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
    }

    #[test]
    fn cmd_parse_test() {
        assert_eq!(
            ClientCommand::deserialize("random_cmd"),
            Err(Error::InvalidCommand("random_cmd".to_string()))
        );

        assert_eq!(
            ClientCommand::deserialize("new_user test, hi"),
            Err(Error::InvalidNumberOfArguments {
                expected: 3,
                actual: 2,
                cmd: "new_user".to_string()
            })
        );
        assert_eq!(
            ClientCommand::deserialize("new_user User Name , user@sample.com,password  "),
            Ok(ClientCommand::NewUser {
                name: "User Name",
                email: "user@sample.com",
                password: "password"
            })
        );

        assert_eq!(
            ClientCommand::deserialize("new_tmp_user   Hi  "),
            Ok(ClientCommand::NewTmpUser { name: "Hi" })
        );

        assert_eq!(
            ClientCommand::deserialize("apikey hello"),
            Err(Error::MalformedApiKey)
        );
        assert_eq!(
            ClientCommand::deserialize("apikey 0123456789abcdef0123456789abcdef"),
            Ok(ClientCommand::Apikey(
                ApiKey::try_from("0123456789abcdef0123456789abcdef")
                    .expect("failed to parse api key")
            ))
        );

        assert_eq!(
            ClientCommand::deserialize("login sample@example.com,password"),
            Ok(ClientCommand::Login {
                email: "sample@example.com",
                password: "password"
            })
        );
        assert_eq!(
            ClientCommand::deserialize("logout"),
            Ok(ClientCommand::Logout)
        );

        assert_eq!(
            ClientCommand::deserialize("gen_apikey   "),
            Ok(ClientCommand::GenApikey)
        );
        assert_eq!(
            ClientCommand::deserialize("self_user_info"),
            Ok(ClientCommand::SelfUserInfo)
        );
    }
}
