#![allow(dead_code)]

use deadpool_postgres::tokio_postgres::{config::Config as tkconfig, NoTls};
use deadpool_postgres::{Config as dpconfig, Pool};
use dotenv::dotenv;
use pglit::{connect, create_db, deadpool_create_db, drop_db, forcedrop_db};

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
    drop_db(&mut config, "pgtools_db_test", NoTls, |res| {
        if let Err(e) = res {
            if e.code == "3D000" {
                eprintln!("attempting to delete a db that doesn't exist");
            } else {
                eprintln!("{:?}", e);
            }
        } else {
            eprintln!("db successfuly deleted");
        }
    })
    .await;
}

#[tokio::test]

async fn createdb_and_dropdb_test() {
    let mut config = get_tokio_config();
    //reset test if run more than once
    let _ = reset_test(&mut config).await;
    create_db(
        &mut config.clone(),
        "pgtools_db_test",
        NoTls,
        |res| match res {
            Ok(_n) => {
                eprintln!("database successfully created");
            }
            Err(e) => eprintln!("error creating db {:?}", e),
        },
    )
    .await;
    // Test workflow success / INSERT table and return value to print in console 
     let table= "CREATE TABLE student(id SERIAL PRIMARY KEY, firstName VARCHAR(40) NOT NULL, lastName VARCHAR(40) NOT NULL, age VARCHAR(40), address VARCHAR(80), email VARCHAR(40))";
    let text = "INSERT INTO student(firstname, lastname, age, address, email) VALUES($1, $2, $3, $4, $5) RETURNING *";
    
    match connect(config.clone(), "pgtools_db_test", NoTls).await {
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
                        &"Mona the",
                        &"Octocat",
                        &"9",
                        &"88 Colin P Kelly Jr St, San Francisco, CA 94107, United States",
                        &"octocat@github.com",
                    ],
                )
                .await
            {
                Ok(vec) => {
                    for raw in vec.iter() {
                        eprintln!("Row back from db : {:?}", r);
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
    // Attempting to create a duplicate db
    create_db(&mut config.clone(), "pgtools_db_test", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            if e.code != "42P04" {
                eprintln!("creating dublicate db should result a 42P04 error");
                assert_eq!("42P04", e.code);
            }
        }
    })
    .await;

    drop_db(&mut config.clone(), "pgtools_db_test", NoTls, |res| {
        assert!(res.is_ok());
        if let Err(e) = res {
            eprintln!("fail to drop database, {:?}", e);
        }
    })
    .await;

    // Attempting to drop a db that was dropped by the previous drop_db call
    drop_db(&mut config.clone(), "pgtools_db_test", NoTls, |res| {
        assert!(res.is_err());
        if let Err(e) = res {
            assert_eq!("3D000", e.code);
        }
    })
    .await
}

#[tokio::test]
async fn force_drop_db_test() {
    //reset test if run more than once
    let mut config = get_tokio_config();
    let _ = reset_test(&mut config).await;
    let force_drop = create_db(&mut config.clone(), "pgtools_db_test", NoTls, |res| async {
        match res {
            Ok(_n) => {
                eprintln!("database successfully created");
                forcedrop_db(&mut config.clone(), "pgtools_db_test", NoTls, |result| {
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
                eprint!("pool object created & returned");
                assert!(&p.is_ok())
            }
            Err(e) => {
                println!("error from pool {:?}", e);
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

#[tokio::test]

async fn connect_to_db_should_succeed() {
    dotenv().ok();
    let cfg = get_tokio_config();
    let result = connect(cfg, "pgtools", NoTls).await;
    assert!(result.is_ok());
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
