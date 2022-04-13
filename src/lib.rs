#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

pub use deadpool_postgres;
use deadpool_postgres::tokio_postgres::{
    tls::MakeTlsConnect, tls::TlsConnect, Client, Config as PgConfig, Connection,
    Error as TokioError, Socket,
};
mod utils;
pub use utils::errors::CustomError as CustomErrors;
use utils::handle_db;

#[doc = "Type alias for using [`CustomError`][CustomErrors] with [`tokio_postgres`][`deadpool_postgres::tokio_postgres`]."]
pub type CustomError = CustomErrors;

/// Creates a new database using the [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// Note that by default the `db_name` argument shouldn't be enclosed in **double quotes** (").\    
/// To create a database that has a name enclosed in **double-quotes** ("), the **`quotes`** feature has to be enabled.
///
/// To Learn more about PostgreSQL's Syntax Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS).
///  
/// The database name in [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`] will be ignored and replaced with the `db_name` argument.
///
/// Obtain a [`Result<u64, CustomError>`] via a callback Closure
///
/// # Panics
///  
/// This function will panic if the `db_name` argument is empty.   
///
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pglit::create_db;
///
///async fn connect_to_db() {
///    let mut config = Config::new();
///    config.user("testuser");
///    config.password("secretPassword");
///    config.dbname("testdb");
///
///    create_db(&mut config, "testdb", NoTls, |result| match result {
///        Ok(_n) => println!("database successfully dropped"),
///        Err(e) => println!("pg_error ,{:?}", e),
///    })
///    .await
///}
/// ```
///
pub async fn create_db<T, F, U>(config: &mut PgConfig, db_name: &str, tls: T, cb: F) -> U
where
    F: FnMut(Result<u64, CustomError>) -> U,
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    handle_db(config, db_name, tls, cb, "CREATE").await
}

/// Dropes a database using the [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// Note that by default the `db_name` argument shouldn't be enclosed in **double quotes**.
/// To drop a database that has a name enclosed in **double-quotes** ("), the **`quotes`** feature has to be enabled.
///
/// To Learn more about PostgreSQL's Syntax Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS).
///  
/// The database name in [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`] will be ignored and replaced with the `db_name` argument.
/// Obtain a [`Result<u64, CustomError>`] via a callback Closure
///
/// # Panics
///  
/// This function will panic if the `db_name` argument is empty.
///  
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pglit::drop_db;
///async fn drop_the_db() {
///    let mut config = Config::new();
///    config.user("testuser");
///    config.password("secretPassword");
///    config.dbname("testdb");
///
///    drop_db(&mut config, "testdb", NoTls, |result| match result {
///        Ok(_n) => println!("database successfully dropped"),
///        Err(e) => println!("pg_error ,{:?}", e),
///    })
///    .await
///}
/// ```
///
pub async fn drop_db<T, F, U>(config: &mut PgConfig, db_name: &str, tls: T, cb: F) -> U
where
    F: FnMut(Result<u64, CustomError>) -> U,
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    handle_db(config, db_name, tls, cb, "DROP").await
}

/// Force drop a database using the [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// This function will force drop the database using the **_Force_** option introduced in `PostgreSQL 13`.
///
/// # Details
/// From the [postgres doc](https://www.postgresql.org/docs/current/sql-dropdatabase.html) the **_Force_** option:
///
/// Attempt to terminate all existing connections to the target database.
/// It doesn't terminate if prepared transactions, active logical replication slots or subscriptions are present in the target database.\
/// This will fail if the current user has no permissions to terminate other connections.<br/>
/// To learn more refer to [postgres doc](https://www.postgresql.org/docs/current/sql-dropdatabase.html)
///
/// # Important
/// Note that by default the `db_name` argument shouldn't be enclosed in **double quotes**.
/// To drop a database that has a name enclosed in **double-quotes** ("), the **`quotes`** feature has to be enabled.
///
///
/// Obtain a [Result<u64, CustomError>] via a callback Closure
///
/// # Panics
///  
/// This function will panic if the `db_name` argument is empty.  
///
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pglit::drop_db;
///async fn force_drop_the_db() {
///    let mut config = Config::new();
///    config.user("testuser");
///    config.password("secretPassword");
///    config.dbname("testdb");
///
///    forcedrop_db(&mut config, "testdb", NoTls, |result| match result {
///        Ok(_n) => println!("database successfully dropped"),
///        Err(e) => println!("pg_error ,{:?}", e),
///    })
///    .await
///}
/// ```
///
pub async fn forcedrop_db<T, F, U>(config: &mut PgConfig, db_name: &str, tls: T, cb: F) -> U
where
    F: FnMut(Result<u64, CustomError>) -> U,
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    handle_db(config, db_name, tls, cb, "DROP, WITH (FORCE);").await
}

use {
    deadpool::managed::BuildError,
    deadpool_postgres::CreatePoolError,
    deadpool_postgres::{Config as dpConfig, Pool, Runtime},
};

/// Convenient function to create a database and get a connection pool using the [`deadpool_postgres`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres) crate .
///
///This function will attempt to create a database by using the [`deadpool_postgres`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres) crate to return a [`Pool`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres/type.Pool.html),
///and it will handle the *"42P04", "Attempting to create a duplicate database."* postgres error if returned.
///
/// # Important
/// Note that by default the `dbname` in the `config` shouldn't be enclosed in **double quotes**.
/// To create a database that has a name enclosed in **double-quotes** ("), the **`quotes`** feature has to be enabled.
///
/// # Errors
///
/// See [`CreatePoolError`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres/type.CreatePoolError.html) for details.
///  
/// # Example
///
///```rust
/// #[derive(serde::Deserialize, Debug)]
/// pub struct Config {
///     pub pg: deadpool_postgres::Config,
/// }
/// impl Config {
///     pub fn from_env() -> Result<Self, config::ConfigError> {
///         ::config::Config::builder()
///             .add_source(::config::Environment::default())
///             .build()?
///             .try_deserialize()
///     }
/// }
///
/// async fn create_db_and_get_pool() {
/// dotenv().ok();
/// let cfg = Config::from_env().unwrap();
/// let cfg = cfg.pg;
///
/// let result = deadpool_create_db(cfg, None, NoTls).await;
/// if let Ok(pool) = &result {
///     let p = pool.get().await;
///     match &p {
///         Ok(_obj) => {
///             println!("pool object created & returned");
///         }
///         Err(e) => {
///             println!("error from pool {:?}", e);
///         }
///     };
///   }
/// }
///```
///
///
///

pub async fn deadpool_create_db<T>(
    config: dpConfig,
    runtime: Option<Runtime>,
    tls: T,
) -> Result<Pool, CreatePoolError>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    let mut pgconfig = config
        .get_pg_config()
        .map_err(deadpool::managed::CreatePoolError::Config)?;

    let db_name = config.dbname.clone().unwrap();

    create_db(&mut pgconfig, &db_name, tls.clone(), |res| match res {
        Ok(_r) => config.create_pool(runtime, tls.clone()),
        Err(e) => {
            if e.code == "42P04" {
                config.create_pool(runtime, tls.clone())
            } else {
                let err =
                    deadpool::managed::CreatePoolError::Build(BuildError::Backend(e.pg_error));
                Err(err)
            }
        }
    })
    .await
}
///Convenient function that attempts to establish a connection with `db_name` and then return [`tokio_postgres`][`deadpool_postgres::tokio_postgres`] [`Client`].
///
/// This function will attempt to establish a connection using the `db_name` argument and it will handle the *"42P04", "Attempting to create a duplicate database."* postgres error if returned, by creating a new database named after the `db_name` argument
/// and then returns a [`tokio_postgres`][`deadpool_postgres::tokio_postgres`] [`Client`].
///
/// Note that by default the `db_name` argument shouldn't be enclosed in **double quotes**.
/// To drop a database that has a name enclosed in **double-quotes** ("), the **`quotes`** feature has to be enabled.
///
/// To Learn more about PostgreSQL's Syntax Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS).
///
/// # Errors
///
/// See [`tokio_postgres::error`][`deadpool_postgres::tokio_postgres::error`] for details.
///
///
pub async fn connect<T>(
    mut config: PgConfig,
    db_name: &str,
    tls: T,
) -> Result<(Client, Connection<Socket, T::Stream>), TokioError>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    let _ = config.dbname(db_name);
    let client_result = create_db(&mut config.clone(), db_name, tls.clone(), |result| async {
        match result {
            Ok(_n) => config.connect(tls.clone()).await,
            Err(e) => {
                if e.code == "42P04" {
                    config.connect(tls.clone()).await
                } else {
                    Err(e.pg_error)
                }
            }
        }
    })
    .await;
    client_result.await
}
