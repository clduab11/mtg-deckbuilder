#[derive(Clone, Debug)]
pub struct MulliganDecision {
    pub keep: bool,
    pub reason: String,
    pub score: f64,
}

pub fn bottom_count_after_mulligans(mulligans: i32) -> anyhow::Result<u32> {
    if mulligans < 0 {
        anyhow::bail!("mulligans must be nonnegative");
    }
    Ok(mulligans as u32)
}
