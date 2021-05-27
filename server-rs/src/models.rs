use super::schema::users;

pub type UserId = i32;

#[derive(Queryable, AsChangeset)]
pub struct User {
    pub id: UserId,
    pub email: Option<String>,
    pub name: String,
    pub is_admin: bool,
    pub password_hash: Option<String>,
    pub api_key_hash: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: Option<&'a str>,
    pub name: &'a str,
    pub is_admin: bool,
    pub password_hash: Option<&'a str>,
    pub api_key_hash: &'a str,
}
