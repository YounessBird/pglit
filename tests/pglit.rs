#![allow(dead_code, unused_macros, unused_imports)]

use deadpool_postgres::tokio_postgres::{config::Config as tkconfig, NoTls};
use deadpool_postgres::{Config as dpconfig, ConfigError, Pool};
use dotenv::dotenv;
use pglit::{connect, create_db, deadpool_create_db, drop_db};

use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(PostgresMapper, Deserialize, Serialize, Debug)]
#[pg_mapper(table = "student")]
pub struct Record {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub age: String,
    pub address: String,
    pub email: String,
}

fn create_pool() -> Pool {
    dotenv().ok();
    let cfg = config::Config::from_env();
    cfg.pg.create_pool(None, NoTls).unwrap()
}
fn get_tokio_config() -> tkconfig {
    dotenv().ok();
    let cfg = config::Config::from_env();
    cfg.pg.get_pg_config().unwrap()
}
fn get_deadpool_config() -> dpconfig {
    dotenv().ok();
    let cfg = config::Config::from_env();
    cfg.pg
}

async fn reset_test(config: &mut tkconfig, db_name: &str) {
    drop_db(&mut config.clone(), db_name, NoTls, |res| match res {
        Ok(_n) => eprintln!("db successfuly deleted"),
        Err(e) => {
            if e.code == "3D000" {
                eprintln!("attempting to delete a db that doesn't exist");
            }
        }
    })
    .await;
}

#[cfg(feature = "quotes")]
#[tokio::test]
async fn db_name_test() {
    let mut config = get_tokio_config();
    let db_name_with_hyphen = "pglit-test";
    let _ = reset_test(&mut config, db_name_with_hyphen).await;

    /*
     Database name created without double quotes and contains a hyphen is not allowed
     for more information please refer to https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS
    */

    // this should succeed if feature="quotes"
    create_db(&mut config.clone(), db_name_with_hyphen, NoTls, |res| {
        assert!(res.is_ok());
        eprintln!("database successfully created");
    })
    .await;

    drop_db(&mut config.clone(), db_name_with_hyphen, NoTls, |res| {
        assert!(res.is_ok());
    })
    .await;
}

#[cfg(not(feature = "quotes"))]
#[tokio::test]

//  if default feature is set a syntax error, code "42601" should be returned
async fn db_name_test() {
    let config = get_tokio_config();
    let db_name_with_hyphen = "pglit-test";

    create_db(&mut config.clone(), db_name_with_hyphen, NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!("42601", e.code);
        }
    })
    .await;

    drop_db(&mut config.clone(), db_name_with_hyphen, NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!("42601", e.code);
        }
    })
    .await;
}

#[cfg(not(feature = "quotes"))]
#[tokio::test]
async fn createdb_dropdb_test() {
    let mut config = get_tokio_config();
    let db_name = "pglit_test_createdb_dropdb";

    //reset test if run more than once
    let _ = reset_test(&mut config, db_name).await;

    // createdb test
    create_db(&mut config.clone(), db_name, NoTls, |res| {
        assert!(res.is_ok());
    })
    .await;

    // Attempting to create a duplicate db
    create_db(&mut config.clone(), db_name, NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            if e.code != "42P04" {
                assert_eq!("42P04", e.code);
            }
        }
    })
    .await;

    //drop_db test
    drop_db(&mut config.clone(), db_name, NoTls, |res| {
        assert!(res.is_ok());
    })
    .await;

    // Attempting to drop a db that was dropped by the previous drop_db call
    drop_db(&mut config.clone(), db_name, NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!("3D000", e.code);
        }
    })
    .await;
}

#[cfg(not(feature = "quotes"))]
#[tokio::test]
async fn connect_db_test() {
    let config = get_tokio_config();
    let db_name = "pglit_db_test_connect";

    //Test connect create database and return (client, connection)
    let table = include_str!("./sql/create_table_test.sql");
    let text = include_str!("./sql/insert_into_table_test.sql");

    let try_connect = connect(config.clone(), db_name, NoTls).await;
    if let Err(e) = &try_connect {
        println!("here is the error, {:?}", e);
    }
    assert!(try_connect.is_ok());

    // Attempt to create duplicate database should result an error
    create_db(&mut config.clone(), db_name, NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e.code, "42P04");
            eprintln!("error creating dublicate db {:?}", e.message);
        }
    })
    .await;

    // connect Shoudl handle duplicate database creation error,INSERT table and return value to print in console
    let try_connect_handle_duplicate = connect(config.clone(), db_name, NoTls).await;
    assert!(try_connect_handle_duplicate.is_ok());

    match try_connect_handle_duplicate {
        Ok((client, connection)) => {
            let _ = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            let _ = &client.query(table, &[]).await;
            match &client
                .query(
                    text,
                    &[
                        &"joe",
                        &"doe",
                        &"9",
                        &"88 Colin P Kelly Jr St, San Francisco, CA 94107, United States",
                        &"joe.doe@example.com",
                    ],
                )
                .await
            {
                Ok(vec) => {
                    let records = vec
                        .iter()
                        .map(|row| Record::from_row_ref(row).unwrap())
                        .collect::<Vec<Record>>();
                    for r in records.iter() {
                        eprintln!("Rows from db {:?}", r)
                    }
                }
                Err(e) => {
                    eprintln!("an error trying to insert data in db {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("error trying to connect to db {:?}", e)
        }
    }
}

#[cfg(not(feature = "quotes"))]
#[tokio::test]
async fn create_db_and_get_pool() {
    let mut cfg = get_deadpool_config();
    let config = get_tokio_config();

    let table = include_str!("./sql/create_table_test.sql");
    let text = include_str!("./sql/insert_into_table_test.sql");

    // function should generate an error; however, it shouldn't panic.
    cfg.dbname = Some(String::from(""));
    let result = deadpool_create_db(cfg.clone(), None, NoTls).await;
    assert!(result.is_err());
    if let Err(result) = result {
        assert_err!(
            result,
            deadpool::managed::CreatePoolError::Config(ConfigError::DbnameMissing)
        );
    }

    // create db and return a pool
    cfg.dbname = Some(String::from("pglit_test_db"));
    let pool = deadpool_create_db(cfg.clone(), None, NoTls).await.unwrap();

    // test if db is already created
    create_db(&mut config.clone(), "pglit_test_db", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e.code, "42P04");
        }
    })
    .await;
    // test pool
    let db_conn = pool.get().await.unwrap();

    let _ = db_conn.query(table, &[]).await;
    match db_conn
        .query(
            text,
            &[
                &"joe",
                &"doe",
                &"9",
                &"88 Colin P Kelly Jr St, San Francisco, CA 94107, United States",
                &"joe.doe@example.com",
            ],
        )
        .await
    {
        Ok(vec) => {
            let records = vec
                .iter()
                .map(|row| Record::from_row_ref(row).unwrap())
                .collect::<Vec<Record>>();
            for r in records.iter() {
                eprintln!("Pool successfuly used to return Row from db {:?}", r)
            }
        }
        Err(e) => {
            eprintln!("an error trying to insert data in db {}", e);
        }
    }

    // test duplicate db handling
    assert!(deadpool_create_db(cfg, None, NoTls).await.is_ok());
}

use std::{collections::HashMap, env};
struct Env {
    backup: HashMap<String, Option<String>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            backup: HashMap::new(),
        }
    }
    pub fn set(&mut self, name: &str, value: &str) {
        self.backup.insert(name.to_string(), env::var(name).ok());
        env::set_var(name, value);
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        for (name, value) in self.backup.iter() {
            println!("setting {} = {:?}", name, value);
            match value {
                Some(val) => env::set_var(name.as_str(), val),
                None => env::remove_var(name.as_str()),
            }
        }
    }
}

#[cfg(not(feature = "quotes"))]
#[test]
fn config_from_env() {
    let mut env = Env::new();
    env.set("ENV_TEST__PG__HOST", "127.0.0.1");
    env.set("ENV_TEST__PG__PORT", "5432");
    env.set("ENV_TEST__PG__USER", "john_doe");
    env.set("ENV_TEST__PG__PASSWORD", "secret");
    env.set("ENV_TEST__PG__DBNAME", "testdb");

    let cfg = config::Config::from_env_with_prefix("ENV_TEST");
    assert_eq!(cfg.pg.host, Some("127.0.0.1".to_string()));
    assert_eq!(cfg.pg.port, Some(5432));
    assert_eq!(cfg.pg.user, Some("john_doe".to_string()));
    assert_eq!(cfg.pg.password, Some("secret".to_string()));
    assert_eq!(cfg.pg.dbname, Some("testdb".to_string()));
}

mod config {

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Config {
        pub pg: deadpool_postgres::Config,
    }
    impl Config {
        pub fn from_env() -> Self {
            config::Config::builder()
                .add_source(config::Environment::default())
                .build()
                .unwrap()
                .try_deserialize::<Self>()
                .unwrap()
        }

        pub fn from_env_with_prefix(prefix: &str) -> Self {
            config::Config::builder()
                .add_source(config::Environment::with_prefix(prefix).separator("__"))
                .build()
                .unwrap()
                .try_deserialize::<Self>()
                .unwrap()
        }
    }
}

macro_rules! assert_err {
    ($expression:expr, $($pattern:tt)+) => {
        match $expression {
            $($pattern)+ => (),
            ref e => panic!("expected `{}` but got `{:?}`", stringify!($($pattern)+), e),
        }
    }
}
use assert_err;
