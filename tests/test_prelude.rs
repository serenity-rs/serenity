#![allow(unused_imports)]

use serenity::prelude::{Mentionable, SerenityError};

#[cfg(feature = "client")]
use serenity::prelude::{Client, ClientError};

// parking_lot re-exports
use serenity::prelude::{Mutex, RwLock};
