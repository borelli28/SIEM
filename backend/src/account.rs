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
    pub password_hash: String,
}

impl Account {
    pub fn new(name: &String, password: &String) -> Result<Self, argon2::password_hash::Error> {
        let password_hash = hash_password(password)?;

        Ok(Account {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            password_hash,
        })
    }

    pub fn verify_password(&self, password: &String) -> bool {
        let parsed_hash = PasswordHash::new(&self.password_hash).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}

pub fn create_account(name: &String, password: &String) -> Result<Account, diesel::result::Error> {
    let mut conn = establish_connection();
    let account = Account::new(name, password).map_err(|e| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::__Unknown, Box::new(e.to_string())
    ))?;

    diesel::insert_into(accounts::table)
        .values(&account)
        .execute(&mut conn)?;
    Ok(account)
}

pub fn get_account(id: &String) -> Result<Option<Account>, diesel::result::Error> {
    let mut conn = establish_connection();
    accounts::table.find(id).first(&mut conn).optional()
}

pub fn update_account(id: &String, name: Option<&String>, password: Option<&String>) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let mut updates = Vec::new();

    if let Some(new_name) = name {
        updates.push(accounts::name.eq(new_name));
    }

    if let Some(new_password) = password {
        let password_hash = hash_password(new_password).map_err(|e| diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::__Unknown, Box::new(e.to_string())
        ))?;
        updates.push(accounts::password_hash.eq(password_hash));
    }

    let affected_rows = diesel::update(accounts::table.find(id))
        .set(updates)
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}

pub fn delete_account(id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let affected_rows = diesel::delete(accounts::table.find(id))
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