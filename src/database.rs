use postgres::*;
use serde::{Deserialize, Serialize};
use rust_decimal::prelude::*;
use rust_decimal::*;

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

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PoliceStation {
    pub id: String,
    pub name: String,
    pub position: Position,
    pub crew: Vec<String>,
    pub drones: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct OperatorMark {
    pub uid: u128,
    pub position: Position,
    pub height: f64,
    pub level: i32,
    pub drone: bool,
    pub desc: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DispatchedRoutes {
    pub route: Vec<(f64, f64)>,
    pub belong: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UnifiedData {
    #[serde(rename = "3durl")]
    _3durl: String,
    #[serde(rename = "3ddstoken")]
    _3ddstoken: String,
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
                    token           VARCHAR NOT NULL,
                    name            VARCHAR NOT NULL,
                    type            INT
                  )", &[]).unwrap();
        self.conn.execute("CREATE TABLE IF NOT EXISTS police_station_data (
                    id              SERIAL PRIMARY KEY,
                    uid             VARCHAR NOT NULL,
                    name            VARCHAR NOT NULL,
                    positionX       DOUBLE PRECISION,
                    positionY       DOUBLE PRECISION,
                    positionZ       DOUBLE PRECISION,
                    crew            VARCHAR[],
                    drone           INT
                  )", &[]).unwrap();
        self.conn.execute("CREATE TABLE IF NOT EXISTS telephone_operator_data (
                    id              SERIAL PRIMARY KEY,
                    uid             DECIMAL,
                    positionX       DOUBLE PRECISION,
                    positionY       DOUBLE PRECISION,
                    positionZ       DOUBLE PRECISION,
                    drone           BOOL,
                    height          DOUBLE PRECISION,
                    level           INT,
                    description     VARCHAR
                  )", &[]).unwrap();
        self.conn.execute("CREATE TABLE IF NOT EXISTS dispatch_routes (
                    id              SERIAL PRIMARY KEY,
                    belong          INT,
                    xs              DOUBLE PRECISION[],
                    ys              DOUBLE PRECISION[]
                    )", &[]).unwrap();
        self.conn.execute("CREATE TABLE IF NOT EXISTS init_data (
                    key             VARCHAR PRIMARY KEY,
                    value           VARCHAR
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
                user_type: row.get(3),
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
    pub fn add_route(&self, route: DispatchedRoutes) {
        let iters: (Vec<_>, Vec<_>) = route.route.iter().cloned().unzip();
        self.conn.execute("INSERT INTO dispatch_routes (belong, xs, ys) VALUES ($1, $2, $3)",
                          &[&(route.belong as i64), &iters.0, &&iters.1]).unwrap();
    }

    pub fn get_routes(&self) -> Vec<DispatchedRoutes> {
        let rows = self.conn
            .query("SELECT * FROM dispatch_routes", &[]).unwrap();
        rows.iter().map(|row| {
            DispatchedRoutes {
                belong: row.get::<usize, i64>(1) as usize,
                route: row.get::<usize, Vec<f64>>(2).into_iter().zip(row.get::<usize, Vec<f64>>(3)).collect::<Vec<_>>(),
            }
        }).collect()
    }

    pub fn remove_routes(&self, belong: usize) -> Result<u64> {
        self.conn.execute("DELETE FROM dispatch_routes WHERE belong=$1", &[&(belong as i64)])
    }
}

impl DatabaseAccess {
    pub fn try_init(&self) -> bool {
        let rows = self.conn
            .query("SELECT * FROM init_data",
                   &[]).unwrap();
        let users = self.conn
            .query("SELECT * FROM user_data",
                   &[]).unwrap();
        if rows.len() < 1 && users.len() < 1 {
            drop(rows);
            drop(users);
            self.add_user(User {
                username: "admin".to_string(),
                passwd: crate::fast_sha256("adminadmin"), // init passwd - admin
                user_type: 1,
            })
        } else if rows.len() > 1 {
            return true;
        }
        false
    }

    pub fn feed_init(&self, data: UnifiedData) {
        self.conn.execute("INSERT INTO init_data (key, value) VALUES ('3durl', $1)"
                          , &[&data._3durl]).unwrap();
        self.conn.execute("INSERT INTO init_data (key, value) VALUES ('3ddstoken', $1)"
                          , &[&data._3ddstoken]).unwrap();
    }

    pub fn load_init(&self) -> UnifiedData {
        let mut unified = UnifiedData {
            _3ddstoken: String::new(),
            _3durl: String::new(),
        };
        let data = self.conn.query("SELECT * FROM init_data", &[]).unwrap();
        data.iter().for_each(|v|
            {
                match &(v.get::<usize, String>(0))[..] {
                    "3durl" => {
                        unified._3durl = v.get::<usize, String>(1).clone();
                    }
                    "3ddstoken" => {
                        unified._3ddstoken = v.get::<usize, String>(1).clone();
                    }
                    _ => {}
                }
            }
        );
        unified
    }
}

impl DatabaseAccess {
    pub fn add_mark(&self, telephone_operator: OperatorMark) -> i32 {
        self.conn.execute(
            "INSERT INTO telephone_operator_data (uid, positionX, positionY, positionZ, drone, height, level, description) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) "
            , &[&Decimal::from_u128(telephone_operator.uid).unwrap(), &telephone_operator.position.x, &telephone_operator.position.y, &telephone_operator.position.z, &telephone_operator.drone,
                &telephone_operator.height, &telephone_operator.level, &telephone_operator.desc]).unwrap();
        let rows = self.conn.query("SELECT id FROM telephone_operator_data WHERE uid = $1"
                                   , &[&Decimal::from_u128(telephone_operator.uid).unwrap()]).unwrap();
        rows.iter().collect::<Vec<_>>().first().unwrap().get(0)
    }

    pub fn find_mark(&self) -> Vec<OperatorMark> {
        let rows = self.conn
            .query("SELECT * FROM telephone_operator_data",
                   &[]).unwrap();
        let marks: Vec<OperatorMark> = rows.iter().map(|row| {
            OperatorMark {
                uid: row.get::<usize, i32>(0) as u128,
                position: Position {
                    x: row.get(2),
                    y: row.get(3),
                    z: row.get(4),
                },
                drone: row.get(5),
                height: row.get(6),
                level: row.get(7),
                desc: row.get(8),
            }
        }).collect();
        return marks;
    }

    pub fn delete_mark(&self, uid: i32) -> bool {
        self.conn.execute("DELETE FROM telephone_operator_data WHERE id=$1"
                          , &[&uid]).is_ok()
    }
}

impl DatabaseAccess {
    pub fn add_police_station(&self, police_station: PoliceStation) {
        self.conn.execute(
            "INSERT INTO police_station_data (uid, name, positionX, positionY, positionZ, crew, drone) VALUES ($1, $2, $3, $4, $5, $6, $7) "
            , &[&police_station.id, &police_station.name,
                &police_station.position.x,
                &police_station.position.y,
                &police_station.position.z,
                &police_station.crew, &police_station.drones]).unwrap();
    }

    pub fn find_police_station(&self) -> Vec<PoliceStation> {
        let rows = self.conn
            .query("SELECT * FROM police_station_data",
                   &[]).unwrap();
        let police_station: Vec<PoliceStation> = rows.iter().map(|row| {
            PoliceStation {
                id: row.get(1),
                name: row.get(2),
                position: Position {
                    x: row.get(3),
                    y: row.get(4),
                    z: row.get(5),
                },
                crew: row.get(6),
                drones: row.get(7),
            }
        }).collect();
        return police_station;
    }

    pub fn delete_police_station(&self, id: String) -> bool {
        self.conn.execute("DELETE FROM police_station_data WHERE uid=$1"
                          , &[&id]).is_ok()
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
                user_type: row.get(3),
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

