use crate::apikey::ApiKey;
use crate::diesel::prelude::*;
use crate::error::Error;
use crate::models::{NewUser, User, UserId};
use crate::schema::users;
use bcrypt;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PoolError, PooledConnection};

impl User {
    pub fn check_password(&self, password: &str) -> bool {
        match self.password_hash.as_deref() {
            None => false,
            Some(hash) => match bcrypt::verify(password.as_bytes(), &hash) {
                Ok(true) => true,
                _ => false,
            },
        }
    }
}

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_db_pool(db_url: &str) -> Result<PgPool, PoolError> {
    let manage = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder().build(manage)
}

/// A database connection wrapper, which associates the database with functions to manipulate it
pub struct DBWrapper(PooledConnection<ConnectionManager<PgConnection>>);

impl DBWrapper {
    /// Wrap an existing pg connection
    pub fn from_pg_pool(pool: &PgPool) -> Result<DBWrapper, Error> {
        Ok(DBWrapper(pool.get()?))
    }

    /// Lookup a user with the given id
    pub fn find_user(&self, id: UserId) -> Result<User, Error> {
        match users::dsl::users
            .find(id)
            .first::<User>(&self.0)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUser),
        }
    }

    /// Lookup user by api key
    pub fn find_user_by_apikey(&self, key: &ApiKey) -> Result<User, Error> {
        let hashed = key.hash();
        match users::dsl::users
            .filter(users::dsl::api_key_hash.eq(hashed.to_string()))
            .first::<User>(&self.0)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::InvalidApiKey),
        }
    }

    /// Lookup user by email
    fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        match users::dsl::users
            .filter(users::dsl::email.eq(email))
            .first::<User>(&self.0)
            .optional()?
        {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUser),
        }
    }

    /// Lookup user by email and password
    pub fn find_user_by_credentials(&self, email: &str, pass: &str) -> Result<User, Error> {
        let user = self.find_user_by_email(email)?;
        match user.check_password(pass) {
            true => Ok(user),
            false => Err(Error::IncorrectCredentials),
        }
    }

    /// Insert a new user into the db
    fn insert_user(&self, user: NewUser) -> Result<User, Error> {
        Ok(diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(&self.0)?)
    }

    /// Create a new user with given info
    pub fn new_user(&self, name: &str, email: &str, pass: &str) -> Result<User, Error> {
        // check for existing user
        match self.find_user_by_email(email) {
            Ok(_) => Err(Error::EmailAlreadyTaken),
            Err(Error::NoSuchUser) => {
                let hashed_pass = bcrypt::hash(pass.as_bytes(), bcrypt::DEFAULT_COST)?;
                let user = NewUser {
                    name,
                    email: Some(email),
                    is_admin: false,
                    password_hash: Some(&*hashed_pass),
                    api_key_hash: &*ApiKey::new().hash().to_string(),
                };
                self.insert_user(user)
            }
            Err(err) => Err(err),
        }
    }

    /// Create a new user with no login credentials
    pub fn new_tmp_user(&self, name: &str) -> Result<User, Error> {
        let user = NewUser {
            name,
            email: None,
            is_admin: false,
            password_hash: None,
            api_key_hash: &*ApiKey::new().hash().to_string(),
        };
        self.insert_user(user)
    }

    /// Update a user already in the db
    pub fn save_user(&self, user: &User) -> Result<(), Error> {
        diesel::update(users::dsl::users.find(user.id))
            .set(user)
            .execute(&self.0)?;
        Ok(())
    }
}