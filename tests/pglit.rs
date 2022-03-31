#![allow(dead_code)]

use deadpool_postgres::tokio_postgres::{config::Config as tkconfig, NoTls};
use deadpool_postgres::{Config as dpconfig, Pool};
use dotenv::dotenv;
use pglit::{connect, create_db, deadpool_create_db, drop_db, forcedrop_db};

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

async fn reset_test(mut config: &mut tkconfig) {
    drop_db(&mut config, "pglit_db_test", NoTls, |res| match res {
        Ok(_n) => eprintln!("db successfuly deleted"),
        Err(e) => {
            if e.code == "3D000" {
                eprintln!("attempting to delete a db that doesn't exist");
            } else {
                eprintln!("{:?}", e);
            }
        }
    })
    .await;
}

#[tokio::test]

async fn createdb_and_dropdb_test() {
    let mut config = get_tokio_config();
    //reset test if run more than once
    let _ = reset_test(&mut config).await;

    // createdb test
    create_db(&mut config.clone(), "pglit_db_test", NoTls, |res| {
        // assert!(res.is_ok());
        // eprintln!("database successfully created");
        // if let Err(e) = res {
        //     eprintln!("{:?}", e.message);
        // }
        match res {
            Ok(_n) => println!("first dp created"),
            Err(e) => println!("error creating db {:?}", e),
        }
    })
    .await;

    // Attempting to create a duplicate db
    create_db(&mut config.clone(), "pglit_db_test", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            if e.code != "42P04" {
                eprintln!("creating dublicate db should result a 42P04 error");
                assert_eq!("42P04", e.code);
            }
        }
    })
    .await;

    drop_db(&mut config.clone(), "pglit_db_test", NoTls, |res| {
        assert!(res.is_ok());
        if let Err(e) = res {
            eprintln!("fail to drop database, {:?}", e);
        }
    })
    .await;

    // Attempting to drop a db that was dropped by the previous drop_db call
    drop_db(&mut config.clone(), "pglit_db_test", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!("3D000", e.code);
        }
    })
    .await
}

#[tokio::test]
async fn connect_test() {
    let config = get_tokio_config();

    let table= "CREATE TABLE student(id BIGSERIAL PRIMARY KEY, first_name VARCHAR(40) NOT NULL, last_name VARCHAR(40) NOT NULL, age VARCHAR(40), address VARCHAR(80), email VARCHAR(40))";
    let text = "INSERT INTO student(first_name, last_name, age, address, email) VALUES($1, $2, $3, $4, $5) RETURNING *";

    //Test connect create database and return (client, connection)
    let try_connect = connect(config.clone(), "pglit_db_test", NoTls).await;
    assert!(try_connect.is_ok());

    // Attempt to create duplicate database should result an error
    create_db(&mut config.clone(), "pglit_db_test", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!(e.code, "42P04");
            eprintln!("error creating dublicate db {:?}", e.message);
        }
    })
    .await;

    // connect Shoudl handle duplicate database creation error,INSERT table and return value to print in console
    let try_connect_handle_duplicate = connect(config.clone(), "pglit_db_test", NoTls).await;
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
#[tokio::test]
async fn force_drop_db_test() {
    //reset test if run more than once
    let mut config = get_tokio_config();
    let _ = reset_test(&mut config).await;
    let force_drop = create_db(&mut config.clone(), "pglit_db_test", NoTls, |res| async {
        match res {
            Ok(_n) => {
                eprintln!("database successfully created");
                forcedrop_db(&mut config.clone(), "pglit_db_test", NoTls, |result| {
                    match &result {
                        Ok(_n) => {
                            eprintln!("database was forced to drop");
                            //assert!(&result.is_ok());
                        }
                        Err(e) => eprintln!("error dropping db {:?}", e),
                    }
                })
                .await;
            }
            Err(e) => eprintln!("error creating db {:?}", e),
        }
    })
    .await;
    force_drop.await;
}

#[tokio::test]
async fn create_db_and_get_pool() {
    let cfg = get_deadpool_config();

    let result = deadpool_create_db(cfg, None, NoTls).await;
    if let Ok(pool) = &result {
        assert!(&result.is_ok());
        let p = pool.get().await;
        match &p {
            Ok(_obj) => {
                eprintln!("pool object created & returned");
                assert!(&p.is_ok())
            }
            Err(e) => {
                eprintln!("error from pool {:?}", e);
                assert!(&p.is_err())
            }
        };
    }
}

#[tokio::test]

async fn creare_db_and_get_pool_should_fail() {
    dotenv().ok();
    let cfg = get_deadpool_config();

    let result = deadpool_create_db(cfg, None, NoTls).await;
    if let Err(_e) = &result {
        assert!(&result.is_err());
    }
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
//#[cfg(feature = "serde")]
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

// Config
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
