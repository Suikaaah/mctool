pub trait MapErrAnyhow<T> {
    fn map_err_anyhow(self) -> anyhow::Result<T>;
}

impl<T, E: std::fmt::Display> MapErrAnyhow<T> for Result<T, E> {
    fn map_err_anyhow(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!("{e}"))
    }
}
