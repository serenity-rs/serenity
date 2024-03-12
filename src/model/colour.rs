// Disable this lint to avoid it wanting to change `0xABCDEF` to `0xAB_CDEF`.
#![allow(clippy::unreadable_literal)]

/// A utility struct to help with working with the basic representation of a colour.
///
/// This is particularly useful when working with a [`Role`]'s colour, as the API works with an
/// integer value instead of an RGB value.
///
/// Instances can be created by using the struct's associated functions. These produce presets
/// equivalent to those found in the official client's colour picker.
///
/// # Examples
///
/// Passing in a role's colour, and then retrieving its green component via [`Self::g`]:
///
/// ```rust
/// # use serde_json::{json, from_value};
/// # use serenity::model::guild::Role;
/// # use serenity::model::id::RoleId;
/// # use serenity::model::id::GuildId;
/// # use serenity::model::permissions;
/// #
/// # fn main() {
/// # let role = from_value::<Role>(json!({
/// #     "color": Colour::BLURPLE,
/// #     "hoist": false,
/// #     "id": RoleId::new(1),
/// #     "guild_id": GuildId::new(2),
/// #     "managed": false,
/// #     "mentionable": false,
/// #     "name": "test",
/// #     "permissions": permissions::PRESET_GENERAL,
/// #     "position": 7,
/// # })).unwrap();
/// #
/// use serenity::model::Colour;
///
/// // assuming a `role` has already been bound
///
/// let green = role.colour.g();
///
/// println!("The green component is: {}", green);
/// # }
/// ```
///
/// Creating an instance with the [`Self::DARK_TEAL`] preset:
///
/// ```rust
/// use serenity::model::Colour;
///
/// let colour = Colour::DARK_TEAL;
///
/// assert_eq!(colour.tuple(), (17, 128, 106));
/// ```
///
/// Colours can also be directly compared for equivalence:
///
/// ```rust
/// use serenity::model::Colour;
///
/// let blitz_blue = Colour::BLITZ_BLUE;
/// let fooyoo = Colour::FOOYOO;
/// let fooyoo2 = Colour::FOOYOO;
/// assert!(blitz_blue != fooyoo);
/// assert_eq!(fooyoo, fooyoo2);
/// assert!(blitz_blue > fooyoo);
/// ```
///
/// [`Role`]: crate::model::guild::Role
#[derive(
    Clone, Copy, Debug, Default, Eq, Ord, Hash, PartialEq, PartialOrd, Deserialize, Serialize,
)]
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
pub struct Colour(pub u32);

pub type Color = Colour;

impl Colour {
    /// Generates a new Colour with the given integer value set.
    ///
    /// # Examples
    ///
    /// Create a new Colour, and then ensure that its inner value is equivalent to a specific RGB
    /// value, retrieved via [`Self::tuple`]:
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// let colour = Colour::new(6573123);
    ///
    /// assert_eq!(colour.tuple(), (100, 76, 67));
    /// ```
    #[must_use]
    pub const fn new(value: u32) -> Colour {
        Colour(value)
    }

    /// Generates a new Colour from an RGB value, creating an inner u32 representation.
    ///
    /// # Examples
    ///
    /// Creating a [`Colour`] via its RGB values will set its inner u32 correctly:
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert!(Colour::from_rgb(255, 0, 0).0 == 0xFF0000);
    /// assert!(Colour::from_rgb(217, 23, 211).0 == 0xD917D3);
    /// ```
    ///
    /// And you can then retrieve those same RGB values via its methods:
    ///
    /// ```rust
    /// use serenity::model::Colour;
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
    #[must_use]
    pub const fn from_rgb(red: u8, green: u8, blue: u8) -> Colour {
        Colour(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// Returns the red RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).r(), 100);
    /// ```
    #[must_use]
    pub const fn r(self) -> u8 {
        ((self.0 >> 16) & 255) as u8
    }

    /// Returns the green RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).g(), 76);
    /// ```
    #[must_use]
    pub const fn g(self) -> u8 {
        ((self.0 >> 8) & 255) as u8
    }

    /// Returns the blue RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).b(), 67);
    /// ```
    #[must_use]
    pub const fn b(self) -> u8 {
        (self.0 & 255) as u8
    }

    /// Returns a tuple of the red, green, and blue components of this Colour.
    ///
    /// This is equivalent to creating a tuple with the return values of [`Self::r`], [`Self::g`],
    /// and [`Self::b`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).tuple(), (100, 76, 67));
    /// ```
    #[must_use]
    pub const fn tuple(self) -> (u8, u8, u8) {
        (self.r(), self.g(), self.b())
    }

    /// Returns a hexadecimal string of this Colour.
    ///
    /// This is equivalent to passing the integer value through [`std::fmt::UpperHex`] with 0
    /// padding and 6 width.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).hex(), "644C43");
    /// ```
    #[must_use]
    pub fn hex(self) -> String {
        format!("{:06X}", self.0)
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
    /// use serenity::model::Colour;
    ///
    /// assert_eq!(Colour::from(6573123u32).r(), 100);
    /// ```
    fn from(value: u32) -> Colour {
        Colour(value & 0xffffff)
    }
}

impl From<(u8, u8, u8)> for Colour {
    /// Constructs a Colour from RGB.
    fn from((red, green, blue): (u8, u8, u8)) -> Self {
        Colour::from_rgb(red, green, blue)
    }
}

impl Colour {
    /// Creates a new [`Colour`], setting its RGB value to `(111, 198, 226)`.
    pub const BLITZ_BLUE: Colour = Colour(0x6FC6E2);
    /// Creates a new [`Colour`], setting its RGB value to `(52, 152, 219)`.
    pub const BLUE: Colour = Colour(0x3498DB);
    /// Creates a new [`Colour`], setting its RGB value to `(114, 137, 218)`.
    pub const BLURPLE: Colour = Colour(0x7289DA);
    /// Creates a new [`Colour`], setting its RGB value to `(32, 102, 148)`.
    pub const DARK_BLUE: Colour = Colour(0x206694);
    /// Creates a new [`Colour`], setting its RGB value to `(194, 124, 14)`.
    pub const DARK_GOLD: Colour = Colour(0xC27C0E);
    /// Creates a new [`Colour`], setting its RGB value to `(31, 139, 76)`.
    pub const DARK_GREEN: Colour = Colour(0x1F8B4C);
    /// Creates a new [`Colour`], setting its RGB value to `(96, 125, 139)`.
    pub const DARK_GREY: Colour = Colour(0x607D8B);
    /// Creates a new [`Colour`], setting its RGB value to `(173, 20, 87)`.
    pub const DARK_MAGENTA: Colour = Colour(0xAD1457);
    /// Creates a new [`Colour`], setting its RGB value to `(168, 67, 0)`.
    pub const DARK_ORANGE: Colour = Colour(0xA84300);
    /// Creates a new [`Colour`], setting its RGB value to `(113, 54, 138)`.
    pub const DARK_PURPLE: Colour = Colour(0x71368A);
    /// Creates a new [`Colour`], setting its RGB value to `(153, 45, 34)`.
    pub const DARK_RED: Colour = Colour(0x992D22);
    /// Creates a new [`Colour`], setting its RGB value to `(17, 128, 106)`.
    pub const DARK_TEAL: Colour = Colour(0x11806A);
    /// Creates a new [`Colour`], setting its RGB value to `(84, 110, 122)`.
    pub const DARKER_GREY: Colour = Colour(0x546E7A);
    /// Creates a new [`Colour`], setting its RGB value to `(250, 177, 237)`.
    pub const FABLED_PINK: Colour = Colour(0xFAB1ED);
    /// Creates a new [`Colour`], setting its RGB value to `(136, 130, 196)`.
    pub const FADED_PURPLE: Colour = Colour(0x8882C4);
    /// Creates a new [`Colour`], setting its RGB value to `(17, 202, 128)`.
    pub const FOOYOO: Colour = Colour(0x11CA80);
    /// Creates a new [`Colour`], setting its RGB value to `(241, 196, 15)`.
    pub const GOLD: Colour = Colour(0xF1C40F);
    /// Creates a new [`Colour`], setting its RGB value to `(186, 218, 85)`.
    pub const KERBAL: Colour = Colour(0xBADA55);
    /// Creates a new [`Colour`], setting its RGB value to `(151, 156, 159)`.
    pub const LIGHT_GREY: Colour = Colour(0x979C9F);
    /// Creates a new [`Colour`], setting its RGB value to `(149, 165, 166)`.
    pub const LIGHTER_GREY: Colour = Colour(0x95A5A6);
    /// Creates a new [`Colour`], setting its RGB value to `(233, 30, 99)`.
    pub const MAGENTA: Colour = Colour(0xE91E63);
    /// Creates a new [`Colour`], setting its RGB value to `(230, 131, 151)`.
    pub const MEIBE_PINK: Colour = Colour(0xE68397);
    /// Creates a new [`Colour`], setting its RGB value to `(230, 126, 34)`.
    pub const ORANGE: Colour = Colour(0xE67E22);
    /// Creates a new [`Colour`], setting its RGB value to `(155, 89, 182)`.
    pub const PURPLE: Colour = Colour(0x9B59B6);
    /// Creates a new [`Colour`], setting its RGB value to `(231, 76, 60)`.
    pub const RED: Colour = Colour(0xE74C3C);
    /// Creates a new [`Colour`], setting its RGB value to `(117, 150, 255)`.
    pub const ROHRKATZE_BLUE: Colour = Colour(0x7596FF);
    /// Creates a new [`Colour`], setting its RGB value to `(246, 219, 216)`.
    pub const ROSEWATER: Colour = Colour(0xF6DBD8);
    /// Creates a new [`Colour`], setting its RGB value to `(26, 188, 156)`.
    pub const TEAL: Colour = Colour(0x1ABC9C);
}

/// Colour constants used by Discord for their branding, role colour palette, etc.
pub mod colours {
    pub mod branding {
        use super::super::Colour;

        /// Creates a new [`Colour`], setting its value to `rgb(88, 101, 242)`.
        pub const BLURPLE: Colour = Colour(0x5865F2);
        /// Creates a new [`Colour`], setting its value to `rgb(87, 242, 135)`.
        pub const GREEN: Colour = Colour(0x57F287);
        /// Creates a new [`Colour`], setting its value to `rgb(254, 231, 92)`.
        pub const YELLOW: Colour = Colour(0xFEE75C);
        /// Creates a new [`Colour`], setting its value to `rgb(235, 69, 158)`.
        pub const FUCHSIA: Colour = Colour(0xEB459E);
        /// Creates a new [`Colour`], setting its value to `rgb(237, 66, 69)`.
        pub const RED: Colour = Colour(0xED4245);
        /// Creates a new [`Colour`], setting its value to `rgb(255, 255, 255)`.
        pub const WHITE: Colour = Colour(0xFFFFFF);
        /// Creates a new [`Colour`], setting its value to `rgb(35, 39, 42)`.
        pub const BLACK: Colour = Colour(0x23272A);
    }
    pub mod css {
        use super::super::Colour;

        /// Creates a new [`Colour`], setting its value to `hsl(139, 47.3%, 43.9%)`.
        pub const POSITIVE: Colour = Colour(0x3BA55D);
        /// Creates a new [`Colour`], setting its value to `hsl(38, 95.7%, 54.1%)`.
        pub const WARNING: Colour = Colour(0xFAA81A);
        /// Creates a new [`Colour`], setting its value to `hsl(359, 82.6%, 59.4%)`.
        pub const DANGER: Colour = Colour(0xED4245);
    }
    pub mod roles {
        use super::super::Colour;

        /// Creates a new [`Colour`], setting its value to `rgb(153, 170, 181)`.
        pub const DEFAULT: Colour = Colour(0x99AAB5);
        /// Creates a new [`Colour`], setting its value to `rgb(26, 188, 156)`.
        pub const TEAL: Colour = Colour(0x1ABC9C);
        /// Creates a new [`Colour`], setting its value to `rgb(17, 128, 106)`.
        pub const DARK_TEAL: Colour = Colour(0x11806A);
        /// Creates a new [`Colour`], setting its value to `rgb(46, 204, 113)`.
        pub const GREEN: Colour = Colour(0x2ECC71);
        /// Creates a new [`Colour`], setting its value to `rgb(31, 139, 76)`.
        pub const DARK_GREEN: Colour = Colour(0x1F8B4C);
        /// Creates a new [`Colour`], setting its value to `rgb(52, 152, 219)`.
        pub const BLUE: Colour = Colour(0x3498DB);
        /// Creates a new [`Colour`], setting its value to `rgb(32, 102, 148)`.
        pub const DARK_BLUE: Colour = Colour(0x206694);
        /// Creates a new [`Colour`], setting its value to `rgb(155, 89, 182)`.
        pub const PURPLE: Colour = Colour(0x9B59B6);
        /// Creates a new [`Colour`], setting its value to `rgb(113, 54, 138)`.
        pub const DARK_PURPLE: Colour = Colour(0x71368A);
        /// Creates a new [`Colour`], setting its value to `rgb(233, 30, 99)`.
        pub const MAGENTA: Colour = Colour(0xE91E63);
        /// Creates a new [`Colour`], setting its value to `rgb(173, 20, 87)`.
        pub const DARK_MAGENTA: Colour = Colour(0xAD1457);
        /// Creates a new [`Colour`], setting its value to `rgb(241, 196, 15)`.
        pub const GOLD: Colour = Colour(0xF1C40F);
        /// Creates a new [`Colour`], setting its value to `rgb(194, 124, 14)`.
        pub const DARK_GOLD: Colour = Colour(0xC27C0E);
        /// Creates a new [`Colour`], setting its value to `rgb(230, 126, 34)`.
        pub const ORANGE: Colour = Colour(0xE67E22);
        /// Creates a new [`Colour`], setting its value to `rgb(168, 67, 0)`.
        pub const DARK_ORANGE: Colour = Colour(0xA84300);
        /// Creates a new [`Colour`], setting its value to `rgb(231, 76, 60)`.
        pub const RED: Colour = Colour(0xE74C3C);
        /// Creates a new [`Colour`], setting its value to `rgb(153, 45, 34)`.
        pub const DARK_RED: Colour = Colour(0x992D22);
        /// Creates a new [`Colour`], setting its value to `rgb(149, 165, 166)`.
        pub const LIGHTER_GREY: Colour = Colour(0x95A5A6);
        /// Creates a new [`Colour`], setting its value to `rgb(151, 156, 159)`.
        pub const LIGHT_GREY: Colour = Colour(0x979C9F);
        /// Creates a new [`Colour`], setting its value to `rgb(96, 125, 139)`.
        pub const DARK_GREY: Colour = Colour(0x607D8B);
        /// Creates a new [`Colour`], setting its value to `rgb(84, 110, 122)`.
        pub const DARKER_GREY: Colour = Colour(0x546E7A);
    }
}

#[cfg(test)]
mod test {
    use super::Colour;

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
        assert_eq!(Colour::from(7u32).0, 7);
    }
}
