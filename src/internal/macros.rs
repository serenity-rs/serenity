//! A set of macros for easily working with internals.

#[cfg(feature = "http")]
macro_rules! request {
    ($route:expr, $method:ident($body:expr), $url:expr, $($rest:tt)*) => {{
        let client = request_client!();

        request($route, || client
            .$method(&format!(api!($url), $($rest)*))
            .body(&$body))?
    }};
    ($route:expr, $method:ident($body:expr), $url:expr) => {{
        let client = request_client!();

        request($route, || client
            .$method(api!($url))
            .body(&$body))?
    }};
    ($route:expr, $method:ident, $url:expr, $($rest:tt)*) => {{
        let client = request_client!();

        request($route, || client
            .$method(&format!(api!($url), $($rest)*)))?
    }};
    ($route:expr, $method:ident, $url:expr) => {{
        let client = request_client!();

        request($route, || client
            .$method(api!($url)))?
    }};
}

#[cfg(feature = "http")]
macro_rules! request_client {
    () => {{
        use hyper::net::HttpsConnector;
        use hyper_native_tls::NativeTlsClient;

        let tc = NativeTlsClient::new()?;
        let connector = HttpsConnector::new(tc);

        HyperClient::with_connector(connector)
    }}
}

#[cfg(any(feature = "model", feature = "utils"))]
macro_rules! cdn {
    ($e:expr) => {
        concat!("https://cdn.discordapp.com", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(cdn!($e), $($rest)*)
    };
}

#[cfg(feature = "http")]
macro_rules! api {
    ($e:expr) => {
        concat!("https://discordapp.com/api/v6", $e)
    };
    ($e:expr, $($rest:tt)*) => {
        format!(api!($e), $($rest)*)
    };
}

#[cfg(feature = "http")]
macro_rules! status {
    ($e:expr) => {
        concat!("https://status.discordapp.com/api/v2", $e)
    }
}

// Enable/disable check for cache
#[cfg(any(all(feature = "cache", any(feature = "client", feature = "model"))))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(any(all(not(feature = "cache"), any(feature = "client", feature = "model"))))]
macro_rules! feature_cache {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for framework
#[cfg(all(feature = "client", feature = "framework"))]
macro_rules! feature_framework {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(all(feature = "client", not(feature = "framework")))]
macro_rules! feature_framework {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

// Enable/disable check for voice
#[cfg(all(feature = "gateway", feature = "voice"))]
macro_rules! feature_voice {
    ($enabled:block else $disabled:block) => {
        {
            $enabled
        }
    }
}

#[cfg(all(feature = "gateway", not(feature = "voice")))]
macro_rules! feature_voice {
    ($enabled:block else $disabled:block) => {
        {
            $disabled
        }
    }
}

macro_rules! enum_number {
    (#[$attr_:meta] $name:ident { $(#[$attr:meta] $variant:ident = $value:expr, )* }) => {
        #[$attr_]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
        pub enum $name {
            $(
                #[$attr]
                $variant = $value,
            )*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter)
                        -> ::std::fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> ::std::result::Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

macro_rules! try_opt {
    ($x:expr) => (match $x {
        Some(v) => v,
        None => return None,
    });
}
