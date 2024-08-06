use crate::{
    git::branch_name::{self, RoswaalOwnedBranchKind, RoswaalOwnedGitBranchName},
    utils::sqlite::RoswaalSqlite,
    with_transaction,
};
use anyhow::Result;

pub enum CloseBranchStatus<'a> {
    Closed(RoswaalOwnedBranchKind),
    UnknownBranchKind(&'a RoswaalOwnedGitBranchName),
}

impl<'a> CloseBranchStatus<'a> {
    pub async fn from_closing_branch(
        branch_name: &'a RoswaalOwnedGitBranchName,
        sqlite: &RoswaalSqlite,
    ) -> Result<Self> {
        match branch_name.kind() {
            Some(kind) => {
                let mut transaction = sqlite.transaction().await?;
                with_transaction!(transaction, async {
                    match kind {
                        RoswaalOwnedBranchKind::AddTests => {
                            transaction.close_add_tests_branch(branch_name).await?;
                        }
                        RoswaalOwnedBranchKind::AddLocations => {
                            transaction.close_add_locations_branch(branch_name).await?;
                        }
                        RoswaalOwnedBranchKind::RemoveTests => {
                            transaction.close_remove_tests_branch(branch_name).await?;
                        }
                    };
                    Ok(Self::Closed(kind))
                })
            }
            None => Ok(Self::UnknownBranchKind(branch_name)),
        }
    }
}
