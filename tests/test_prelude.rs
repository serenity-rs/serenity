#![allow(unused_imports)]

extern crate serenity;

use serenity::prelude::{Mentionable, SerenityError};

#[cfg(feature = "client")]
use serenity::prelude::{Client, ClientError};

// parking_lot re-exports
use serenity::prelude::{Mutex, RwLock};
