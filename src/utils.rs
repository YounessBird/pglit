use deadpool_postgres::tokio_postgres::{
    tls::MakeTlsConnect, tls::TlsConnect, Config as PgConfig, Socket,
};

type CustomError = errors::CustomError;
const ADMIN_DB: &str = "postgres";

/// Handles creating and dropping the database
pub(crate) async fn handle_db<F, T, U>(
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
    if db_name == "" {
        panic!("The database name in the `db_name` argument should not be empty");
    }
    let _ = config.dbname(ADMIN_DB);
    let mut db_name = db_name.to_string();

    if cfg!(feature = "quotes") {
        let escaped_db_name = db_name.replace('\"', "");
        db_name = format!(r#""{}""#, escaped_db_name);
    }

    let db_name = db_name.as_str();
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
pub(crate) mod errors {
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
        #[must_use]
        /// Create a new [`CustomError`]
        pub(crate) fn new(error: PGError) -> CustomError {
            CustomError {
                message: if error.as_db_error() == None {
                    "".to_string()
                } else {
                    error.as_db_error().unwrap().message().replace('\"', "")
                },
                code: if error.code() == None {
                    "".to_string()
                } else {
                    error.code().unwrap().code().to_string()
                },
                pg_error: error,
            }
        }
    }
}
