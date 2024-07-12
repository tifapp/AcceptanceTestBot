use anyhow::Result;

pub enum RemoveTestsStatus {

}

impl RemoveTestsStatus {
    pub async fn from_removing_tests() -> Result<()> {
        Ok(())
    }
}
