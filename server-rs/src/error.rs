use bcrypt;
use diesel;

#[derive(Debug)]
pub enum Error {
    DBError(diesel::result::Error),
    BCryptError(bcrypt::BcryptError),
    NoSuchUser,
    MalformedApiKey,
    InvalidApiKey,
    IncorrectCredentials,
    EmailAlreadyTaken,
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error { Error::DBError(e) }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(e: bcrypt::BcryptError) -> Error { Error::BCryptError(e) }
}