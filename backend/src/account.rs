use crate::database::establish_connection;
use crate::schema::accounts;
use diesel::prelude::*;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt).map(|hash| hash.to_string())
}

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = accounts)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

impl Account {
    pub fn verify_password(&self, password: &String) -> bool {
        let parsed_hash = PasswordHash::new(&self.password).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}

pub fn create_account(mut account: Account) -> Result<usize, diesel::result::Error> {
    let mut conn = establish_connection();
    account.id = Uuid::new_v4().to_string();

    diesel::insert_into(accounts::table)
        .values(&account)
        .execute(&mut conn)
}

pub fn get_account(id: &String) -> Result<Option<Account>, diesel::result::Error> {
    let mut conn = establish_connection();
    accounts::table.filter(accounts::id.eq(id)).first(&mut conn).optional()
}

pub fn update_account(account: &Account) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let affected_rows = diesel::update(accounts::table.find(&account.id))
        .set(account)
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}

pub fn delete_account(id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let affected_rows = diesel::delete(accounts::table.filter(accounts::id.eq(id)))
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}

pub fn verify_login(name: &String, password: &String) -> Result<Option<Account>, diesel::result::Error> {
    let mut conn = establish_connection();

    let account: Option<Account> = accounts::table
        .filter(accounts::name.eq(name))
        .first(&mut conn)
        .optional()?;

    if let Some(account) = account {
        if account.verify_password(password) {
            Ok(Some(account))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}