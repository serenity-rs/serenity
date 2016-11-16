use std::default::Default;
use ::internal::prelude::*;

macro_rules! colour {
    ($struct_:ident; $($name:ident, $val:expr;)*) => {
        impl $struct_ {
            $(
                pub fn $name() -> Colour {
                    Colour::new($val)
                }
            )*
        }
    }
}

/// A utility struct to help with working with the basic representation of a
/// colour. This is particularly useful when working with a [`Role`]'s colour,
/// as the API works with an integer value instead of an RGB value.
///
/// Instances can be created by using the struct's associated functions. These
/// produce presets equivilant to those found in the official client's colour
/// picker.
///
/// # Examples
///
/// Passing in a role's colour, and then retrieving its green component
/// via [`get_g`]:
///
/// ```rust,ignore
/// use serenity::utils::Colour;
///
/// // assuming a `role` has already been bound
///
/// let colour = Colour::new(role.colour);
/// let green = colour.get_g();
///
/// println!("The green component is: {}", green);
/// ```
///
/// Creating an instance with the [`dark_teal`] presets:
///
/// ```rust,ignore
/// use serenity::utils::Colour;
///
/// let colour = Colour::dark_teal();
///
/// assert_eq!(colour.get_tuple(), (17, 128, 106));
/// ```
///
/// [`Role`]: ../model/struct.Role.html
/// [`dark_teal`]: #method.dark_teal
/// [`get_g`]: #method.get_g
#[derive(Clone, Copy, Debug)]
pub struct Colour {
    /// The raw inner integer value of this Colour. This is worked with to
    /// generate values such as the red component value.
    pub value: u32,
}

impl Colour {
    /// Generates a new Colour with the given integer value set.
    pub fn new(value: u32) -> Colour {
        Colour {
            value: value,
        }
    }

    /// Generates a new Colour from an RGB value, creating an inner u32
    /// representation.
    ///
    /// # Examples
    ///
    /// Creating a `Colour` via its RGB values will set its inner u32 correctly:
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert!(Colour::from_rgb(255, 0, 0).value == 0xFF0000);
    /// assert!(Colour::from_rgb(217, 23, 211).value == 0xD917D3);
    /// ```
    ///
    /// And you can then retrieve those same RGB values via its methods:
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// let colour = Colour::from_rgb(217, 45, 215);
    ///
    /// assert_eq!(colour.get_r(), 217);
    /// assert_eq!(colour.get_g(), 45);
    /// assert_eq!(colour.get_b(), 215);
    /// assert_eq!(colour.get_tuple(), (217, 45, 215));
    /// ```
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Colour {
        let mut uint = r as u32;
        uint = (uint << 8) | (g as u32);
        uint = (uint << 8) | (b as u32);

        Colour::new(uint)
    }

    #[doc(hidden)]
    pub fn decode(value: Value) -> Result<Colour> {
        match value {
            Value::U64(v) => Ok(Colour::new(v as u32)),
            Value::I64(v) => Ok(Colour::new(v as u32)),
            other => Err(Error::Decode("Expected valid colour", other)),
        }
    }

    /// Returns the red RGB component of this Colour.
    pub fn get_r(&self) -> u8 {
        ((self.value >> 16) & 255) as u8
    }

    /// Returns the green RGB component of this Colour.
    pub fn get_g(&self) -> u8 {
        ((self.value >> 8) & 255) as u8
    }

    /// Returns the blue RGB component of this Colour.
    pub fn get_b(&self) -> u8 {
        (self.value & 255) as u8
    }

    /// Returns a tuple of the red, green, and blue components of this Colour.
    pub fn get_tuple(&self) -> (u8, u8, u8) {
        (self.get_r(), self.get_g(), self.get_b())
    }
}

impl From<u32> for Colour {
    fn from(value: u32) -> Colour {
        Colour::new(value)
    }
}

impl From<u64> for Colour {
    fn from(value: u64) -> Colour {
        Colour::new(value as u32)
    }
}

colour! {
    Colour;
    blue, 0x3498db;
    dark_blue, 0x206694;
    dark_green, 0x1f8b4c;
    dark_gold, 0xc27c0e;
    dark_grey, 0x607d8b;
    dark_magenta, 0xad1457;
    dark_orange, 0xa84300;
    dark_purple, 0x71368a;
    dark_red, 0x992d22;
    dark_teal, 0x11806a;
    darker_grey, 0x546e7a;
    gold, 0xf1c40f;
    light_grey, 0x979c9f;
    lighter_grey, 0x95a5a6;
    magenta, 0xe91e63;
    orange, 0xe67e22;
    purple, 0x9b59b6;
    red, 0xe74c3c;
    teal, 0x1abc9c;
}

impl Default for Colour {
    fn default() -> Colour {
        Colour {
            value: 0,
        }
    }
}
