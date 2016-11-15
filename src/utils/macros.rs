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

// Enable/disable check for extras
macro_rules! feature_extras {
    ($enabled:block) => {
        {
            feature_extras_enabled! {{
                $enabled
            }}
        }
    };
    ($enabled:block $disabled:block) => {
        {
            feature_extras_enabled! {{
                $enabled
            }}

            feature_extras_disabled! {{
                $disabled
            }}
        }
    };
}

#[cfg(feature = "extras")]
macro_rules! feature_extras_enabled {
    ($enabled:block) => {{
        {
            $enabled
        }
    }}
}

#[cfg(not(feature = "extras"))]
macro_rules! feature_extras_enabled {
    ($enabled:block) => {}
}

#[cfg(feature = "extras")]
macro_rules! feature_extras_disabled {
    ($disabled:block) => {}
}

#[cfg(not(feature = "extras"))]
macro_rules! feature_extras_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for framework
macro_rules! feature_framework {
    ($enabled:block) => {
        {
            feature_framework_enabled! {{
                $enabled
            }}
        }
    };
    ($enabled:block $disabled:block) => {
        {
            feature_framework_enabled! {{
                $enabled
            }}

            feature_framework_disabled! {{
                $disabled
            }}
        }
    };
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
    ($enabled:block) => {}
}

#[cfg(feature = "framework")]
macro_rules! feature_framework_disabled {
    ($disabled:block) => {}
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
macro_rules! feature_methods {
    ($enabled:block) => {
        {
            feature_methods_enabled! {{
                $enabled
            }}
        }
    };
    ($enabled:block $disabled:block) => {
        {
            feature_methods_enabled! {{
                $enabled
            }}

            feature_methods_disabled! {{
                $disabled
            }}
        }
    };
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
    ($enabled:block) => {}
}

#[cfg(feature = "methods")]
macro_rules! feature_methods_disabled {
    ($disabled:block) => {}
}

#[cfg(not(feature = "methods"))]
macro_rules! feature_methods_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for state
#[cfg(feature = "state")]
macro_rules! feature_state {
    ($enabled:block) => {
        {
            feature_state_enabled! {{
                $enabled
            }}
        }
    };
    ($enabled:block else $disabled:block) => {
        {
            feature_state_enabled! {{
                $enabled
            }}

            feature_state_disabled! {{
                $disabled
            }}
        }
    };
}

#[cfg(feature = "state")]
macro_rules! feature_state_enabled {
    ($enabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(not(feature = "state"))]
macro_rules! feature_state_enabled {
    ($enabled:block) => {}
}

#[cfg(feature = "state")]
macro_rules! feature_state_disabled {
    ($disabled:block) => {}
}

#[cfg(not(feature = "state"))]
macro_rules! feature_state_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for voice
macro_rules! feature_voice {
    ($enabled:block) => {
        {
            feature_voice_enabled! {{
                $enabled
            }}
        }
    };
    ($enabled:block $disabled:block) => {
        {
            feature_voice_enabled! {{
                $enabled
            }}

            feature_voice_disabled! {{
                $disabled
            }}
        }
    };
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
    ($enabled:block) => {}
}

#[cfg(feature = "voice")]
macro_rules! feature_voice_disabled {
    ($disabled:block) => {}
}

#[cfg(not(feature = "voice"))]
macro_rules! feature_voice_disabled {
    ($disabled:block) => {
        {
            $disabled
        }
    }
}
