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

/// Creates a new database using this [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// Note that by default the database name created will be without **the double quotes**. This comes with some benefits:
///
///  - Omit typing the double quotes all the time.
///  - It will always be folded to lower case.
///
/// If you wish the database name to be created with the double quotes, you need to enable the **`quotes`** feature.
///
/// Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS) For more details.
///  
/// The database name in [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`] will be ignored and replaced with the `db_name` argument.
///
/// Obtain a [`Result<u64, CustomError>`] via a callback Closure
///
///
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pgtools::create_db;
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

/// Dropes a database using this [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// Note that by default the database name dropped will be without the **double quotes**.
///If you wish the database name to be dropped with **the double quotes**, you need to enable the **`quotes`** feature.
///
/// Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS) For more details.
///  
/// The database name in [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`] will be ignored and replaced with the `db_name` argument.
/// Obtain a [`Result<u64, CustomError>`] via a callback Closure
///
///
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pgtools::drop_db;
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

/// Force Drop a database using this [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
///
/// This function will force drop the database using the Force option introduced in `PostgreSQL 13`.
///
/// # Details
/// From the postgres doc :
/// Attempt to terminate all existing connections to the target database.
/// It doesn't terminate if prepared transactions, active logical replication slots or subscriptions are present in the target database.
/// This will fail if the current user has no permissions to terminate other connections.
/// Required permissions are the same as with pg_terminate_backend, described in Section 9.27.2.
/// This will also fail if we are not able to terminate connections.
///
/// obtain a [Result<u64, `CustomError`>] via a callback Closure
///
///
/// # Errors
///
/// See [`CustomError`] for details.
///
/// # Example
///
/// ```
/// use tokio_postgres::{config::Config,NoTls};
/// use pgtools::drop_db;
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
pub async fn forcedrop_db<T, F>(config: &mut PgConfig, db_name: &str, tls: T, cb: F)
where
    F: FnMut(Result<u64, CustomError>),
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    handle_db(config, db_name, tls, cb, "DROP, WITH (FORCE);").await;
}

use {
    deadpool::managed::BuildError,
    deadpool_postgres::CreatePoolError,
    deadpool_postgres::{Config as dpConfig, Pool, Runtime},
};

/// Convenient function to programmatically create a database.
///
///This function will attempt to create a database and using the [`deadpool_postgres`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres) crate to return a [`Pool`](https://docs.rs/deadpool-postgres/0.10.1/deadpool_postgres/type.Pool.html),
///and it will handle the "42P04", "Attempting to create a duplicate database." postgres error if returned.
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
/// async fn create_db_and_get_pool_should_succeed() {
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
/// }
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
/// Note that by default the database name created will be without the **double quotes**.
/// If you wish the database name to be created with **the double quotes**, you need to enable the **`quotes`** feature.
///
/// Refer to [PosgreSql doc](https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS) For more details.
///
/// This function will attempt to establish a connection using the `db_name` argument and it will handle the "42P04", "Attempting to create a duplicate database." postgres error if returned, by creating a new database named after the `db_name` argument provided
/// and then returns a [`tokio_postgres`][`deadpool_postgres::tokio_postgres`] [`Client`].
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
