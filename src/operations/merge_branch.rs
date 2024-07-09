use crate::{git::branch_name::{RoswaalOwnedBranchKind, RoswaalOwnedGitBranchName}, utils::sqlite::{self, RoswaalSqlite}};
use anyhow::Result;

#[derive(Debug, PartialEq, Eq)]
pub enum MergeBranchStatus<'a> {
    MergedNewLocations,
    MergedTestRemovals,
    MergedNewTests,
    UnknownBranchKind(&'a RoswaalOwnedGitBranchName)
}

impl <'a> MergeBranchStatus<'a> {
    pub async fn for_merging_branch_with_name(
        branch_name: &'a RoswaalOwnedGitBranchName,
        sqlite: &RoswaalSqlite
    ) -> Result<Self> {
        match branch_name.kind() {
            Some(RoswaalOwnedBranchKind::AddLocations) => {
                Ok(Self::MergedNewLocations)
            },
            Some(RoswaalOwnedBranchKind::AddTests) => {
                Ok(Self::MergedNewTests)
            },
            Some(RoswaalOwnedBranchKind::RemoveTests) => {
                Ok(Self::MergedTestRemovals)
            },
            None => {
                Ok(Self::UnknownBranchKind(branch_name))
            }
        }
    }
}
