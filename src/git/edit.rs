pub enum EditGitRepositoryStatus {
    Success { did_delete_branch: bool },
    MergeConflict
}
