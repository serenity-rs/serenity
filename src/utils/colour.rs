use std::default::Default;
use ::internal::prelude::*;

macro_rules! colour {
    ($(#[$attr:meta] $name:ident, $val:expr;)*) => {
        impl Colour {
            $(
                #[$attr]
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
/// ```rust
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
    /// The raw inner 32-bit unsigned integer value of this Colour. This is
    /// worked with to generate values such as the red component value.
    pub value: u32,
}

impl Colour {
    /// Generates a new Colour with the given integer value set.
    ///
    /// # Examples
    ///
    /// Create a new Colour, and then ensure that its inner value is equivilant
    /// to a specific RGB value, retrieved via [`get_tuple`]:
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// let colour = Colour::new(6573123);
    ///
    /// assert_eq!(colour.get_tuple(), (100, 76, 67));
    /// ```
    ///
    /// [`get_tuple`]: #method.get_tuple
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).get_r(), 100);
    /// ```
    pub fn get_r(&self) -> u8 {
        ((self.value >> 16) & 255) as u8
    }

    /// Returns the green RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).get_g(), 76);
    /// ```
    pub fn get_g(&self) -> u8 {
        ((self.value >> 8) & 255) as u8
    }

    /// Returns the blue RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).get_b(), 67);
    pub fn get_b(&self) -> u8 {
        (self.value & 255) as u8
    }

    /// Returns a tuple of the red, green, and blue components of this Colour.
    ///
    /// This is equivilant to creating a tuple with the return values of
    /// [`get_r`], [`get_g`], and [`get_b`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).get_tuple(), (100, 76, 67));
    /// ```
    ///
    /// [`get_r`]: #method.get_r
    /// [`get_g`]: #method.get_g
    /// [`get_b`]: #method.get_b
    pub fn get_tuple(&self) -> (u8, u8, u8) {
        (self.get_r(), self.get_g(), self.get_b())
    }
}

impl From<i32> for Colour {
    /// Constructs a Colour from a i32.
    ///
    /// This is used for functions that accept `Into<Colour>`.
    ///
    /// This is useful when providing hex values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::from(0xDEA584).get_tuple(), (222, 165, 132));
    /// ```
    fn from(value: i32) -> Colour {
        Colour::new(value as u32)
    }
}

impl From<u32> for Colour {
    /// Constructs a Colour from a u32.
    ///
    /// This is used for functions that accept `Into<Colour>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::from(6573123u32).get_r(), 100);
    /// ```
    fn from(value: u32) -> Colour {
        Colour::new(value)
    }
}

impl From<u64> for Colour {
    /// Constructs a Colour from a u32.
    ///
    /// This is used for functions that accept `Into<Colour>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::from(6573123u64).get_r(), 100);
    /// ```
    fn from(value: u64) -> Colour {
        Colour::new(value as u32)
    }
}

colour! {
    /// Creates a new `Colour`, setting its RGB value to `(111, 198, 226)`.
    blitz_blue, 0x6FC6E2;
    /// Creates a new `Colour`, setting its RGB value to `(52, 152, 219)`.
    blue, 0x3498DB;
    /// Creates a new `Colour`, setting its RGB value to `(114, 137, 218)`.
    blurple, 0x7289DA;
    /// Creates a new `Colour`, setting its RGB value to `(32, 102, 148)`.
    dark_blue, 0x206694;
    /// Creates a new `Colour`, setting its RGB value to `(194, 124, 14)`.
    dark_gold, 0xC27C0E;
    /// Creates a new `Colour`, setting its RGB value to `(31, 139, 76)`.
    dark_green, 0x1F8B4C;
    /// Creates a new `Colour`, setting its RGB value to `(96, 125, 139)`.
    dark_grey, 0x607D8B;
    /// Creates a new `Colour`, setting its RGB value to `(173, 20, 87)`.
    dark_magenta, 0xAD1457;
    /// Creates a new `Colour`, setting its RGB value to `(168, 67, 0)`.
    dark_orange, 0xA84300;
    /// Creates a new `Colour`, setting its RGB value to `(113, 54, 138)`.
    dark_purple, 0x71368A;
    /// Creates a new `Colour`, setting its RGB value to `(153, 45, 34)`.
    dark_red, 0x992D22;
    /// Creates a new `Colour`, setting its RGB value to `(17, 128, 106)`.
    dark_teal, 0x11806A;
    /// Creates a new `Colour`, setting its RGB value to `(84, 110, 122)`.
    darker_grey, 0x546E7A;
    /// Creates a new `Colour`, setting its RGB value to `(17, 202, 128)`.
    fooyoo, 0x11CA80;
    /// Creates a new `Colour`, setting its RGB value to `(241, 196, 15)`.
    gold, 0xF1C40F;
    /// Creates a new `Colour`, setting its RGB value to `(186, 218, 85)`.
    kerbal, 0xBADA55;
    /// Creates a new `Colour`, setting its RGB value to `(151, 156, 159)`.
    light_grey, 0x979C9F;
    /// Creates a new `Colour`, setting its RGB value to `(149, 165, 166)`.
    lighter_grey, 0x95A5A6;
    /// Creates a new `Colour`, setting its RGB value to `(233, 30, 99)`.
    magenta, 0xE91E63;
    /// Creates a new `Colour`, setting its RGB value to `(230, 126, 34)`.
    orange, 0xE67E22;
    /// Creates a new `Colour`, setting its RGB value to `(155, 89, 182)`.
    purple, 0x9B59B6;
    /// Creates a new `Colour`, setting its RGB value to `(231, 76, 60)`.
    red, 0xE74C3C;
    /// Creates a new `Colour`, setting its RGB value to `(250, 177, 237)`.
    fabled_pink, 0xFAB1ED
    /// Creates a new `Colour`, setting its RGB value to `(26, 188, 156)`.
    teal, 0x1ABC9C;
}

impl Default for Colour {
    /// Creates a default value for a `Colour`, setting the inner value to `0`.
    /// This is equivilant to setting the RGB value to `(0, 0, 0)`.
    fn default() -> Colour {
        Colour {
            value: 0,
        }
    }
}
