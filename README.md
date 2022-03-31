# Pglit

![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

An implementation of PostgreSQL's createdb, dropdb and other tools.

This crate uses [`tokio-postgres`](https://crates.io/crates/tokio-postgres) to implement postgresql's tools such as createdb, dropdb. It also uses [`deadpool-postgres`](https://crates.io/crates/deadpool-postgres) crate to support connection pooling.

## Example: create database using `tokio_postgres::Config` object

```rust,no_run
use tokio_postgres::{config::Config, NoTls};
use Pglit::create_db;

async fn connect_to_db() {
    let mut config = Config::new();
    config.user("testuser");
    config.password("password");
    config.dbname("testdb");

    create_db(&mut config, "testdb", NoTls, |result| match result {
        Ok(_n) => println!("database successfully dropped"),
        Err(e) => println!("pg_error ,{:?}", e),
    })
    .await
}
```

## Example: drop database using `tokio_postgres::Config` object

```rust,no_run
use tokio_postgres::{config::Config, NoTls};
use Pglit::drop_db;
async fn drop_the_db() {
    let mut config = Config::new();
    config.user("testuser");
    config.password("password");
    config.dbname("testdb");

    drop_db(&mut config, "testdb", NoTls, |result| match result {
        Ok(_n) => println!("database successfully dropped"),
        Err(e) => println!("pg_error ,{:?}", e),
    })
    .await
}
```

## Example with `deadpool-postgres` and `config` crates

```rust,no_run
use Pglit::deadpool_create_db;

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub pg: deadpool_postgres::Config,
}
impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        ::config::Config::builder()
            .add_source(::config::Environment::default())
            .build()?
            .try_deserialize()
    }
}

async fn create_db_and_get_pool() {
    dotenv().ok();
    let cfg = Config::from_env().unwrap();
    let cfg = cfg.pg;

    let result = deadpool_create_db(cfg, None, NoTls).await;
    if let Ok(pool) = &result {
        let p = pool.get().await;
        match &p {
            Ok(_obj) => {
                println!("pool object created & returned");
            }
            Err(e) => {
                println!("error from pool {:?}", e);
            }
        };
    }
}

```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
