use std::sync::Arc;

use sqlx::database::HasArguments;
use sqlx::query::{Query, QueryAs};
use sqlx::{query, query_as, Executor, FromRow, Pool, Transaction};
use sqlx::sqlite::{Sqlite, SqliteArguments, SqliteQueryResult, SqliteRow};
use anyhow::Result;
use tokio::sync::{Mutex, MutexGuard};

/// A type that serializes transactions to sqlite to prevent sqlite busy errors from ocurring.
pub struct RoswaalSqlite {
    mutex: Arc<Mutex<Pool<Sqlite>>>
}

impl RoswaalSqlite {
    /// Attempts to open a new sqlite connection at the specified path.
    pub async fn open(path: &str) -> Result<Self> {
        let pool = Pool::<Sqlite>::connect(path).await?;
        Self::migrate_v1(&pool).await?;
        Ok(RoswaalSqlite { mutex: Arc::new(Mutex::new(pool)) })
    }

    /// Attempts to open an in-memory sqlite connection.
    pub async fn in_memory() -> Result<Self> {
        Self::open(":memory:").await
    }

    async fn migrate_v1(pool: &Pool<Sqlite>) -> Result<()> {
        query(
            "
CREATE TABLE IF NOT EXISTS Locations (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    name TEXT NOT NULL,
    unmerged_branch_name TEXT,
    creation_date DATETIME NOT NULL DEFAULT (unixepoch()),
    UNIQUE(name, unmerged_branch_name)
);
CREATE TABLE IF NOT EXISTS Tests (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    unmerged_branch_name TEXT,
    creation_date DATETIME NOT NULL DEFAULT (unixepoch()),
    UNIQUE(name, unmerged_branch_name)
);
CREATE TABLE IF NOT EXISTS TestSteps (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    test_id INTEGER NOT NULL,
    content TEXT NOT_NULL,
    ordinal INTEGER NOT NULL,
    did_pass INT2 NOT NULL DEFAULT FALSE,
    creation_date DATETIME NOT NULL DEFAULT (unixepoch()),
    UNIQUE(test_id, content),
    CONSTRAINT fk_test FOREIGN KEY(test_id) REFERENCES Tests(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS StagedTestRemovals (
    name TEXT NOT NULL,
    unmerged_branch_name TEXT NOT NULL,
    PRIMARY KEY(name, unmerged_branch_name)
);
            "
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl RoswaalSqlite {
    /// Attempts to open a transaction.
    pub async fn transaction(&self) -> Result<RoswaalSqliteTransaction> {
        let pool = self.mutex.lock().await;
        let transaction = pool.begin().await?;
        Ok(RoswaalSqliteTransaction { pool, transaction })
    }
}

/// A sqlite transaction runner.
#[derive(Debug)]
pub struct RoswaalSqliteTransaction<'a> {
    pool: MutexGuard<'a, Pool<Sqlite>>,
    transaction: Transaction<'static, Sqlite>
}

impl <'a> RoswaalSqliteTransaction<'a> {
    /// Returns the underlying sqlx connection for this transaction.
    pub fn connection(&mut self) -> impl Executor<Database = Sqlite> {
        self.transaction.as_mut()
    }

    /// Performs a rollback.
    pub async fn rollback(self) -> Result<()> {
        drop(self.pool);
        self.transaction.rollback().await?;
        Ok(())
    }

    /// Performs a commit.
    pub async fn commit(self) -> Result<()> {
        drop(self.pool);
        self.transaction.commit().await?;
        Ok(())
    }
}

/// Runs the work inside the transaction, rolling back if the work returns an error, or committing
/// if the work returns successfully.
#[macro_export]
macro_rules! with_transaction {
    ($transaction:ident, $work:expr) => {
        match $work.await {
            Ok(value) => {
                let result = $transaction.commit().await;
                if let Err(err) = result {
                    Err(err)
                } else {
                    Ok(value)
                }
            },
            Err(error) => {
                let result = $transaction.rollback().await;
                if let Err(err) = result {
                    Err(err)
                } else {
                    Err(error)
                }
            }
        }
    }
}

/// Returns a list of substitutable array parameters for use in sqlite queries that check if a
/// value is contained within a list of values.
///
/// ```rs
/// impl RoswaalSqliteTransaction {
///     pub async fn items(&mut self, item_names: &Vec<&str>) -> Result<Vec<Item>> {
///         let select_query_statement = format!(
///             "SELECT * FROM Items WHERE name in {};",
///             sqlite_array_fields(item_names.len())
///         );
///         // Execute query...
///     }
/// }
/// ```
pub fn sqlite_array_fields(count: usize) -> String {
    format!("({})", (0..count).map(|_| "?").collect::<Vec<&str>>().join(", "))
}

pub type SqliteQuery<'q> = Query<'q, Sqlite, SqliteArguments<'q>>;
pub type SqliteQueryAs<'q, Output> = QueryAs<'q, Sqlite, Output, SqliteArguments<'q>>;

/// A repeated Sqlite Query.
pub struct SqliteRepeat<'q, Value> {
    statements: String,
    values: &'q Vec<Value>
}

/// Repeats the SQL statement for each value in `values`.
pub fn sqlite_repeat<'q, Value>(
    statement: &'q str,
    values: &'q Vec<Value>
) -> SqliteRepeat<'q, Value> {
    let statements = values.iter().map(|_| statement).collect::<Vec<&str>>().join("\n");
    SqliteRepeat { statements, values }
}

impl <'q, Value> SqliteRepeat<'q, Value> {
    /// Given a function with a query and each value, binds the value to the query.
    pub fn bind_to_query(
        &'q self,
        bind: impl Fn(SqliteQuery<'q>, &'q Value) -> Result<SqliteQuery<'q>>
    ) -> Result<SqliteQuery<'q>> {
        let mut query = query::<Sqlite>(&self.statements);
        for value in self.values.iter() {
            query = bind(query, &value)?;
        }
        Ok(query)
    }

    /// Given a function with a query and each value from `values`, binds the value to the query.
    pub fn bind_custom_values_to_query<V>(
        &'q self,
        values: impl Iterator<Item = V>,
        bind: impl Fn(SqliteQuery<'q>, &V) -> Result<SqliteQuery<'q>>
    ) -> Result<SqliteQuery<'q>> {
        let mut query = query::<Sqlite>(&self.statements);
        for value in values {
            query = bind(query, &value)?;
        }
        Ok(query)
    }

    /// Given a function with q query and each value, binds the value to the query.
    pub fn bind_to_query_as<Output: for<'r> FromRow<'r, SqliteRow>>(
        &'q self,
        bind: impl Fn(SqliteQueryAs<'q, Output>, &'q Value) -> Result<SqliteQueryAs<'q, Output>>
    ) -> Result<SqliteQueryAs<'q, Output>> {
        let mut query = query_as::<Sqlite, Output>(&self.statements);
        for value in self.values.iter() {
            query = bind(query, &value)?;
        }
        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{prelude::FromRow, query_as};

    use super::*;

    #[derive(FromRow, Debug, PartialEq, Eq)]
    struct TestRecord { id: i32 }

    #[tokio::test]
    async fn test_basic_query() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        _ = query("CREATE TABLE Test (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT)")
            .execute(transaction.connection())
            .await;
        _ = query("INSERT INTO Test (id) VALUES (1)")
            .execute(transaction.connection())
            .await;
        let result: TestRecord = query_as("SELECT * FROM Test")
            .fetch_one(transaction.connection())
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(result, TestRecord { id: 1 });
    }

    #[tokio::test]
    async fn test_commit_and_rollback_on_failure() {
        let sqlite = RoswaalSqlite::in_memory().await.unwrap();
        let mut transaction = sqlite.transaction().await.unwrap();
        _ = with_transaction!(transaction, async {
            _ = query("CREATE TABLE Test (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT)")
                .execute(transaction.connection())
                .await?;
            _ = query("INSERT INTO Test (id) VALUES (1)")
                .execute(transaction.connection())
                .await?;
            Ok(())
        });
        transaction = sqlite.transaction().await.unwrap();
        let transaction_result = with_transaction!(transaction, async {
            _ = query("INSERT INTO Test (id) VALUES (5)").execute(transaction.connection()).await?;
            _ = query("INSERT INTO Test (id) VALUES (1)").execute(transaction.connection()).await?;
            Ok(())
        });
        transaction = sqlite.transaction().await.unwrap();
        assert!(transaction_result.is_err());
        let result: Vec<TestRecord> = query_as("SELECT * FROM Test")
            .fetch_all(transaction.connection())
            .await
            .unwrap();
        assert_eq!(result, vec![TestRecord { id: 1 }]);
    }
}
