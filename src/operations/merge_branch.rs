use crate::{
    git::branch_name::{RoswaalOwnedBranchKind, RoswaalOwnedGitBranchName},
    utils::sqlite::{self, RoswaalSqlite},
    with_transaction,
};
use anyhow::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum MergeBranchStatus<'a> {
    Merged(RoswaalOwnedBranchKind),
    UnknownBranchKind(&'a RoswaalOwnedGitBranchName),
}

impl<'a> MergeBranchStatus<'a> {
    pub async fn from_merging_branch_with_name(
        branch_name: &'a RoswaalOwnedGitBranchName,
        sqlite: &RoswaalSqlite,
    ) -> Result<Self> {
        match branch_name.kind() {
            Some(kind) => {
                let mut transaction = sqlite.transaction().await?;
                with_transaction!(transaction, async {
                    match kind {
                        RoswaalOwnedBranchKind::AddTests => {
                            transaction.merge_unmerged_tests(&branch_name).await?;
                        }
                        RoswaalOwnedBranchKind::AddLocations => {
                            transaction.merge_unmerged_locations(&branch_name).await?;
                        }
                        RoswaalOwnedBranchKind::RemoveTests => {
                            transaction.merge_test_removals(&branch_name).await?;
                        }
                    }
                    Ok(Self::Merged(kind))
                })
            }
            None => Ok(Self::UnknownBranchKind(branch_name)),
        }
    }
}
