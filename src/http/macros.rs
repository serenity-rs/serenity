macro_rules! status {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}
