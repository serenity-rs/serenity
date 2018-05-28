macro_rules! try_uri {
    ($url:expr) => {{
        match ::hyper::Uri::from_str($url) {
            Ok(v) => v,
            Err(why) => return Box::new(::futures::future::err(::Error::Uri(why))),
        }
    }};
}

macro_rules! api {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

macro_rules! status {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}
