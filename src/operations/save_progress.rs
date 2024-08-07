use anyhow::Result;

use crate::{
    tests_data::progress::RoswaalTestProgressUpload, utils::sqlite::RoswaalSqlite, with_transaction,
};

pub async fn save_test_progress(
    progress: &Vec<RoswaalTestProgressUpload>,
    sqlite: &RoswaalSqlite,
) -> Result<()> {
    let mut transaction = sqlite.transaction().await?;
    with_transaction!(transaction, async {
        transaction.save_test_progess(progress).await
    })
}
