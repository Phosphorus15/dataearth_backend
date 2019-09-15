use postgres::*;
use serde::{Deserialize, Serialize};

pub struct DatabaseAccess {
    conn: Connection
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub username: String,
    pub passwd: String,
    #[serde(rename = "type")]
    pub user_type: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LoginInfo {
    pub username: String,
    pub token: String,
    #[serde(rename = "type")]
    pub user_type: i32,
}

impl DatabaseAccess {
    pub fn new(url: &'_ str) -> Self {
        DatabaseAccess {
            conn: Connection::connect(url, TlsMode::None).unwrap()
        }
    }
}

impl DatabaseAccess {
    pub fn init(&self) {
        self.conn.execute("CREATE TABLE IF NOT EXISTS user_data (
                    id              SERIAL PRIMARY KEY,
                    name            VARCHAR NOT NULL,
                    passwd          VARCHAR NOT NULL,
                    type            INT
                  )", &[]).unwrap();
        self.conn.execute("CREATE TABLE IF NOT EXISTS login_data (
                    id              SERIAL PRIMARY KEY,
                    token            VARCHAR NOT NULL,
                    name          VARCHAR NOT NULL,
                    type            INT
                  )", &[]).unwrap();
    }

    pub fn add_user(&self, user: User) {
        self.conn.execute(
            "INSERT INTO user_data (name, passwd, type) VALUES ($1, $2, $3) "
            , &[&user.username, &user.passwd, &user.user_type]).unwrap();
    }

    pub fn find_user(&self, username: String) -> Option<User> {
        let rows = self.conn
            .query("SELECT * FROM user_data WHERE name=$1",
                   &[&username]).unwrap();
        let users: Vec<User> = rows.iter().map(|row| {
            User {
                username: row.get(1),
                passwd: row.get(2),
                user_type: row.get(3)
            }
        }).collect();
        return users.first().map(|u| u.clone());
    }

    pub fn delete_user(&self, username: String) -> bool {
        self.conn.execute("DELETE FROM user_data WHERE name=$1"
                          , &[&username]).is_ok()
    }

}

impl DatabaseAccess {
    pub fn add_login(&self, user: LoginInfo) {
        self.conn.execute(
            "INSERT INTO login_data (name, token, type) VALUES ($1, $2, $3) "
            , &[&user.username, &user.token, &user.user_type]).unwrap();
    }

    pub fn find_login(&self, token: String) -> Option<LoginInfo> {
        let rows = self.conn
            .query("SELECT * FROM login_data WHERE token=$1",
                   &[&token]).unwrap();
        let info: Vec<LoginInfo> = rows.iter().map(|row| {
            LoginInfo {
                username: row.get(2),
                token: row.get(1),
                user_type: row.get(3)
            }
        }).collect();
        return info.first().map(|u| u.clone());
    }

    pub fn logout(&self, token: String) -> bool {
        self.conn.execute("DELETE FROM login_data WHERE token=$1"
            , &[&token]).is_ok()
    }
}


impl Default for DatabaseAccess {
    fn default() -> Self {
        Self::new("postgres://postgres:12345@localhost:5432")
    }
}
