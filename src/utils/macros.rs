macro_rules! request {
    ($route:expr, $method:ident($body:expr), $url:expr, $($rest:tt)*) => {{
        let client = HyperClient::new();
        try!(request($route, || client
            .$method(&format!(api!($url), $($rest)*))
            .body(&$body)))
    }};
    ($route:expr, $method:ident($body:expr), $url:expr) => {{
        let client = HyperClient::new();
        try!(request($route, || client
            .$method(api!($url))
            .body(&$body)))
    }};
    ($route:expr, $method:ident, $url:expr, $($rest:tt)*) => {{
        let client = HyperClient::new();
        try!(request($route, || client
            .$method(&format!(api!($url), $($rest)*))))
    }};
    ($route:expr, $method:ident, $url:expr) => {{
        let client = HyperClient::new();
        try!(request($route, || client
            .$method(api_concat!($url))))
    }};
}

macro_rules! cdn_concat {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    }
}
macro_rules! api {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

macro_rules! api_concat {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    }
}
macro_rules! status_concat {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}

// Enable/disable check for cache
#[cfg(feature = "cache")]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "cache"))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

#[cfg(feature = "cache")]
macro_rules! feature_cache_enabled {
    ($enabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "cache"))]
macro_rules! feature_cache_enabled {
    ($enabled:block) => {{}}
}

#[cfg(feature = "cache")]
macro_rules! feature_cache_disabled {
    ($disabled:block) => {{}}
}

#[cfg(not(feature = "cache"))]
macro_rules! feature_cache_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for framework
#[cfg(feature = "framework")]
macro_rules! feature_framework {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "framework"))]
macro_rules! feature_framework {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

#[cfg(feature = "framework")]
macro_rules! feature_framework_enabled {
    ($enabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "framework"))]
macro_rules! feature_framework_enabled {
    ($enabled:block) => {{}}
}

#[cfg(feature = "framework")]
macro_rules! feature_framework_disabled {
    ($disabled:block) => {{}}
}

#[cfg(not(feature = "framework"))]
macro_rules! feature_framework_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for methods
#[cfg(feature = "methods")]
macro_rules! feature_methods {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "methods"))]
macro_rules! feature_methods {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

#[cfg(feature = "methods")]
macro_rules! feature_methods_enabled {
    ($enabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "methods"))]
macro_rules! feature_methods_enabled {
    ($enabled:block) => {{}}
}

#[cfg(feature = "methods")]
macro_rules! feature_methods_disabled {
    ($disabled:block) => {{}}
}

#[cfg(not(feature = "methods"))]
macro_rules! feature_methods_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for voice
#[cfg(feature = "voice")]
macro_rules! feature_voice {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "voice"))]
macro_rules! feature_voice {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

#[cfg(feature = "voice")]
macro_rules! feature_voice_enabled {
    ($enabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "voice"))]
macro_rules! feature_voice_enabled {
    ($enabled:block) => {{}}
}

#[cfg(feature = "voice")]
macro_rules! feature_voice_disabled {
    ($disabled:block) => {{}}
}

#[cfg(not(feature = "voice"))]
macro_rules! feature_voice_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}
