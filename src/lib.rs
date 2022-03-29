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

const ADMIN_DB: &str = "postgres";

#[doc = "Type alias for using [`errors::CustomError`] with [`tokio_postgres`][`deadpool_postgres::tokio_postgres`]."]
pub type CustomError = errors::CustomError;

/// Creates a new database using this [`tokio_postgres::Config`][`deadpool_postgres::tokio_postgres::Config`].
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
/// This function will force drop the database using the Force option introduced in PostgreSQL 13.
///
/// # Details
/// From the postgres doc :
/// Attempt to terminate all existing connections to the target database.
/// It doesn't terminate if prepared transactions, active logical replication slots or subscriptions are present in the target database.
/// This will fail if the current user has no permissions to terminate other connections.
/// Required permissions are the same as with pg_terminate_backend, described in Section 9.27.2.
/// This will also fail if we are not able to terminate connections.
///
/// obtain a [Result<u64, CustomError>] via a callback Closure
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

/// Handles creating and dropping the database
async fn handle_db<F, T, U>(
    config: &mut PgConfig,
    db_name: &str,
    tls: T,
    mut cb: F,
    action: &str,
) -> U
where
    F: FnMut(Result<u64, CustomError>) -> U,
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    let _ = config.dbname(ADMIN_DB);
    match config.connect(tls).await {
        Ok((client, connection)) => {
            let db_sql = get_sql_statement(action, db_name);
            let _ = config.dbname(db_name);
            // Note : to be changed
            let _ = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });
            // maybe handle error before passing the to call back
            match (&client).execute(&db_sql, &[]).await {
                Ok(res) => cb(Ok(res)),
                Err(pgerror) => {
                    // Note: review code check if error handeling is neccesary here
                    cb(Err(errors::CustomError::new(pgerror)))
                }
            }
        }
        Err(pgerror) => {
            println!("cb received pg result");
            cb(Err(errors::CustomError::new(pgerror)))
        }
    }
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

    let db_name = config.dbname.clone().unwrap().to_owned();

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
///Convenient function that attempts to establish a connection with *db_name and then return [`tokio_postgres`][`deadpool_postgres::tokio_postgres`] [`Client`].
///
/// This function will attempt to establish a connection using the *`db_name` and it will handle the "42P04", "Attempting to create a duplicate database." postgres error if returned, by creating a new database named after the *`db_name` provided
/// and then returns a [`tokio_postgres`][`deadpool_postgres::tokio_postgres`] [`Client`].
///
/// # Errors
///
/// See [`tokio_postgres::error`][`deadpool_postgres::tokio_postgres::error`] for details.
///
///
pub async fn connect<T>(
    config: PgConfig,
    db_name: &str,
    tls: T,
) -> Result<(Client, Connection<Socket, T::Stream>), TokioError>
where
    T: MakeTlsConnect<Socket> + Clone + Sync + Send + 'static,
    T::Stream: Sync + Send,
    T::TlsConnect: Sync + Send,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
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

fn get_sql_statement(action: &str, db_name: &str) -> String {
    let stm = action.split(',').collect::<Vec<&str>>();
    let db_sql = include_str!("../sql/create_or_drop_db.sql").replace("$db_name", db_name);
    let mut db_sql = db_sql.replace("$action", stm[0]).trim().to_string();
    if action.contains("DROP, WITH (FORCE);") {
        let _ = db_sql.pop();
        db_sql.push_str(stm[1]);
    }
    db_sql
}

/// A convenient way to access the error message and code
pub mod errors {
    use deadpool_postgres::tokio_postgres::Error as PGError;

    /// Wrapper to make it convenient to access the error message and code or the entire [`tokio_postgres::Error`][`PGError`].
    #[derive(Debug)]
    pub struct CustomError {
        ///Error message
        pub message: String,
        ///Error Code
        pub code: String,
        ///Postgres Error
        pub pg_error: PGError,
    }
    impl CustomError {
        /// Create a new [`CustomError`]
        pub fn new(error: PGError) -> CustomError {
            CustomError {
                message: if error.as_db_error() != None {
                    error.as_db_error().unwrap().message().replace('\"', "")
                } else {
                    "".to_string()
                },
                code: if error.code() != None {
                    error.code().unwrap().code().to_string()
                } else {
                    "".to_string()
                },
                pg_error: error,
            }
        }
    }
}
