// Disable this lint to avoid it wanting to change `0xABCDEF` to `0xAB_CDEF`.
#![allow(clippy::unreadable_literal)]

macro_rules! colour {
    ($(#[$attr:meta] $constname:ident, $name:ident, $val:expr;)*) => {
        impl Colour {
            $(
                #[$attr]
                pub const $constname: Colour = Colour($val);
            )*
        }
    }
}

/// A utility struct to help with working with the basic representation of a
/// colour. This is particularly useful when working with a [`Role`]'s colour,
/// as the API works with an integer value instead of an RGB value.
///
/// Instances can be created by using the struct's associated functions. These
/// produce presets equivalent to those found in the official client's colour
/// picker.
///
/// # Examples
///
/// Passing in a role's colour, and then retrieving its green component
/// via [`g`]:
///
/// ```rust
/// # extern crate serde_json;
/// # extern crate serenity;
/// #
/// # use serde_json::json;
/// # use serenity::model::guild::Role;
/// # use serenity::model::id::RoleId;
/// # use serenity::model::permissions;
/// #
/// # fn main() {
/// # let role = serde_json::from_value::<Role>(json!({
/// #     "color": Colour::BLURPLE,
/// #     "hoist": false,
/// #     "id": RoleId(1),
/// #     "managed": false,
/// #     "mentionable": false,
/// #     "name": "test",
/// #     "permissions": permissions::PRESET_GENERAL,
/// #     "position": 7,
/// # })).unwrap();
/// #
/// use serenity::utils::Colour;
///
/// // assuming a `role` has already been bound
///
/// let green = role.colour.g();
///
/// println!("The green component is: {}", green);
/// # }
/// ```
///
/// Creating an instance with the [`DARK_TEAL`] preset:
///
/// ```rust
/// use serenity::utils::Colour;
///
/// let colour = Colour::DARK_TEAL;
///
/// assert_eq!(colour.tuple(), (17, 128, 106));
/// ```
///
/// Colours can also be directly compared for equivalence:
///
/// ```rust
/// use serenity::utils::Colour;
///
/// let blitz_blue = Colour::BLITZ_BLUE;
/// let fooyoo = Colour::FOOYOO;
/// let fooyoo2 = Colour::FOOYOO;
/// assert!(blitz_blue != fooyoo);
/// assert_eq!(fooyoo, fooyoo2);
/// assert!(blitz_blue > fooyoo);
/// ```
///
/// [`Role`]: ../model/guild/struct.Role.html
/// [`DARK_TEAL`]: #associatedconstant.DARK_TEAL
/// [`g`]: #method.g
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Colour(pub u32);

impl Colour {
    /// Generates a new Colour with the given integer value set.
    ///
    /// # Examples
    ///
    /// Create a new Colour, and then ensure that its inner value is equivalent
    /// to a specific RGB value, retrieved via [`tuple`]:
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// let colour = Colour::new(6573123);
    ///
    /// assert_eq!(colour.tuple(), (100, 76, 67));
    /// ```
    ///
    /// [`tuple`]: #method.tuple
    #[inline]
    pub const fn new(value: u32) -> Colour { Colour(value) }

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
    /// assert!(Colour::from_rgb(255, 0, 0).0 == 0xFF0000);
    /// assert!(Colour::from_rgb(217, 23, 211).0 == 0xD917D3);
    /// ```
    ///
    /// And you can then retrieve those same RGB values via its methods:
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// let colour = Colour::from_rgb(217, 45, 215);
    ///
    /// assert_eq!(colour.r(), 217);
    /// assert_eq!(colour.g(), 45);
    /// assert_eq!(colour.b(), 215);
    /// assert_eq!(colour.tuple(), (217, 45, 215));
    /// ```
    // Clippy wants to use `u32::from` instead `as`-casts,
    // but this not doable as `u32::from` is not a const fn.
    #[allow(clippy::cast_lossless)]
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Colour {
        Colour((red as u32) << 16 | (green as u32) << 8 | blue as u32)
    }

    /// Returns the red RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).r(), 100);
    /// ```
    pub const fn r(self) -> u8 { ((self.0 >> 16) & 255) as u8 }

    /// Returns the green RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).g(), 76);
    /// ```
    pub const fn g(self) -> u8 { ((self.0 >> 8) & 255) as u8 }

    /// Returns the blue RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).b(), 67);
    pub const fn b(self) -> u8 { (self.0 & 255) as u8 }

    /// Returns a tuple of the red, green, and blue components of this Colour.
    ///
    /// This is equivalent to creating a tuple with the return values of
    /// [`r`], [`g`], and [`b`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).tuple(), (100, 76, 67));
    /// ```
    ///
    /// [`r`]: #method.r
    /// [`g`]: #method.g
    /// [`b`]: #method.b
    pub const fn tuple(self) -> (u8, u8, u8) { (self.r(), self.g(), self.b()) }

    /// Returns a hexadecimal string of this Colour.
    ///
    /// This is equivalent to passing the integer value through
    /// `std::fmt::UpperHex` with 0 padding and 6 width.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).hex(), "644C43");
    /// ```
    pub fn hex(self) -> String {
        format!("{:06X}", self.0)
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
    /// assert_eq!(Colour::from(0xDEA584).tuple(), (222, 165, 132));
    /// ```
    fn from(value: i32) -> Colour { Colour(value as u32) }
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
    /// assert_eq!(Colour::from(6573123u32).r(), 100);
    /// ```
    fn from(value: u32) -> Colour { Colour(value) }
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
    /// assert_eq!(Colour::from(6573123u64).r(), 100);
    /// ```
    fn from(value: u64) -> Colour { Colour(value as u32) }
}

impl From<(u8, u8, u8)> for Colour {
    /// Constructs a Colour from RGB.
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Colour::from_rgb(red, green, blue)
    }
}

colour! {
    /// Creates a new `Colour`, setting its RGB value to `(111, 198, 226)`.
    BLITZ_BLUE, blitz_blue, 0x6FC6E2;
    /// Creates a new `Colour`, setting its RGB value to `(52, 152, 219)`.
    BLUE, blue, 0x3498DB;
    /// Creates a new `Colour`, setting its RGB value to `(114, 137, 218)`.
    BLURPLE, blurple, 0x7289DA;
    /// Creates a new `Colour`, setting its RGB value to `(32, 102, 148)`.
    DARK_BLUE, dark_blue, 0x206694;
    /// Creates a new `Colour`, setting its RGB value to `(194, 124, 14)`.
    DARK_GOLD, dark_gold, 0xC27C0E;
    /// Creates a new `Colour`, setting its RGB value to `(31, 139, 76)`.
    DARK_GREEN, dark_green, 0x1F8B4C;
    /// Creates a new `Colour`, setting its RGB value to `(96, 125, 139)`.
    DARK_GREY, dark_grey, 0x607D8B;
    /// Creates a new `Colour`, setting its RGB value to `(173, 20, 87)`.
    DARK_MAGENTA, dark_magenta, 0xAD1457;
    /// Creates a new `Colour`, setting its RGB value to `(168, 67, 0)`.
    DARK_ORANGE, dark_orange, 0xA84300;
    /// Creates a new `Colour`, setting its RGB value to `(113, 54, 138)`.
    DARK_PURPLE, dark_purple, 0x71368A;
    /// Creates a new `Colour`, setting its RGB value to `(153, 45, 34)`.
    DARK_RED, dark_red, 0x992D22;
    /// Creates a new `Colour`, setting its RGB value to `(17, 128, 106)`.
    DARK_TEAL, dark_teal, 0x11806A;
    /// Creates a new `Colour`, setting its RGB value to `(84, 110, 122)`.
    DARKER_GREY, darker_grey, 0x546E7A;
    /// Creates a new `Colour`, setting its RGB value to `(250, 177, 237)`.
    FABLED_PINK, fabled_pink, 0xFAB1ED;
    /// Creates a new `Colour`, setting its RGB value to `(136, 130, 196)`.
    FADED_PURPLE, faded_purple, 0x8882C4;
    /// Creates a new `Colour`, setting its RGB value to `(17, 202, 128)`.
    FOOYOO, fooyoo, 0x11CA80;
    /// Creates a new `Colour`, setting its RGB value to `(241, 196, 15)`.
    GOLD, gold, 0xF1C40F;
    /// Creates a new `Colour`, setting its RGB value to `(186, 218, 85)`.
    KERBAL, kerbal, 0xBADA55;
    /// Creates a new `Colour`, setting its RGB value to `(151, 156, 159)`.
    LIGHT_GREY, light_grey, 0x979C9F;
    /// Creates a new `Colour`, setting its RGB value to `(149, 165, 166)`.
    LIGHTER_GREY, lighter_grey, 0x95A5A6;
    /// Creates a new `Colour`, setting its RGB value to `(233, 30, 99)`.
    MAGENTA, magenta, 0xE91E63;
    /// Creates a new `Colour`, setting its RGB value to `(230, 131, 151)`.
    MEIBE_PINK, meibe_pink, 0xE68397;
    /// Creates a new `Colour`, setting its RGB value to `(230, 126, 34)`.
    ORANGE, orange, 0xE67E22;
    /// Creates a new `Colour`, setting its RGB value to `(155, 89, 182)`.
    PURPLE, purple, 0x9B59B6;
    /// Creates a new `Colour`, setting its RGB value to `(231, 76, 60)`.
    RED, red, 0xE74C3C;
    /// Creates a new `Colour`, setting its RGB value to `(117, 150, 255)`.
    ROHRKATZE_BLUE, rohrkatze_blue, 0x7596FF;
    /// Creates a new `Colour`, setting its RGB value to `(246, 219, 216)`.
    ROSEWATER, rosewater, 0xF6DBD8;
    /// Creates a new `Colour`, setting its RGB value to `(26, 188, 156)`.
    TEAL, teal, 0x1ABC9C;
}

impl Default for Colour {
    /// Creates a default value for a `Colour`, setting the inner value to `0`.
    fn default() -> Colour { Colour(0) }
}

#[cfg(test)]
mod test {
    use super::Colour;
    use std::u32;

    #[test]
    fn new() {
        assert_eq!(Colour::new(1).0, 1);
        assert_eq!(Colour::new(u32::MIN).0, u32::MIN);
        assert_eq!(Colour::new(u32::MAX).0, u32::MAX);
    }

    #[test]
    fn from_rgb() {
        assert_eq!(Colour::from_rgb(255, 0, 0).0, 0xFF0000);
        assert_eq!(Colour::from_rgb(0, 255, 0).0, 0x00FF00);
        assert_eq!(Colour::from_rgb(0, 0, 255).0, 0x0000FF);
    }

    #[test]
    fn r() {
        assert_eq!(Colour::new(0x336123).r(), 0x33);
    }

    #[test]
    fn g() {
        assert_eq!(Colour::new(0x336123).g(), 0x61);
    }

    #[test]
    fn b() {
        assert_eq!(Colour::new(0x336123).b(), 0x23);
    }

    #[test]
    fn tuple() {
        assert_eq!(Colour::new(0x336123).tuple(), (0x33, 0x61, 0x23));
    }

    #[test]
    fn default() {
        assert_eq!(Colour::default().0, 0);
    }

    #[test]
    fn from() {
        assert_eq!(Colour::from(7i32).0, 7);
        assert_eq!(Colour::from(7u32).0, 7);
        assert_eq!(Colour::from(7u64).0, 7);
    }
}
