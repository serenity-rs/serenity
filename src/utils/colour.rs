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
/// produce presets equivalent to those found in the official client's colour
/// picker.
///
/// # Examples
///
/// Passing in a role's colour, and then retrieving its green component
/// via [`g`]:
///
/// ```rust
/// # use serenity::model::{Role, RoleId, permissions};
/// use serenity::utils::Colour;
/// #
/// # let role = Role {
/// #     colour: Colour::blurple(),
/// #     hoist: false,
/// #     id: RoleId(1),
/// #     managed: false,
/// #     mentionable: false,
/// #     name: "test".to_owned(),
/// #     permissions: permissions::PRESET_GENERAL,
/// #     position: 7,
/// # };
///
/// // assuming a `role` has already been bound
///
/// let green = role.colour.g();
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
/// assert_eq!(colour.tuple(), (17, 128, 106));
/// ```
///
/// Colours can also be directly compared for equivalence:
///
/// ```rust
/// use serenity::utils::Colour;
///
/// let blitz_blue = Colour::blitz_blue();
/// let fooyoo = Colour::fooyoo();
/// let fooyoo2 = Colour::fooyoo();
/// assert!(blitz_blue != fooyoo);
/// assert_eq!(fooyoo, fooyoo2);
/// assert!(blitz_blue > fooyoo);
/// ```
///
/// [`Role`]: ../model/struct.Role.html
/// [`dark_teal`]: #method.dark_teal
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
    pub fn new(value: u32) -> Colour {
        Colour(value)
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
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Colour {
        let mut uint = r as u32;
        uint = (uint << 8) | (g as u32);
        uint = (uint << 8) | (b as u32);

        Colour(uint)
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
    pub fn r(&self) -> u8 {
        ((self.0 >> 16) & 255) as u8
    }

    /// Returns the green RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).g(), 76);
    /// ```
    pub fn g(&self) -> u8 {
        ((self.0 >> 8) & 255) as u8
    }

    /// Returns the blue RGB component of this Colour.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::utils::Colour;
    ///
    /// assert_eq!(Colour::new(6573123).b(), 67);
    pub fn b(&self) -> u8 {
        (self.0 & 255) as u8
    }

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
    pub fn tuple(&self) -> (u8, u8, u8) {
        (self.r(), self.g(), self.b())
    }

    /// Alias of [`r`].
    ///
    /// [`r`]: #method.r
    #[deprecated(since="0.1.5", note="Use `r` instead.")]
    #[inline]
    pub fn get_r(&self) -> u8 {
        self.r()
    }

    /// Alias of [`g`].
    ///
    /// [`g`]: #method.g
    #[deprecated(since="0.1.5", note="Use `g` instead.")]
    #[inline]
    pub fn get_g(&self) -> u8 {
        self.g()
    }

    /// Alias of [`b`].
    ///
    /// [`b`]: #method.b
    #[deprecated(since="0.1.5", note="Use `b` instead.")]
    #[inline]
    pub fn get_b(&self) -> u8 {
        self.b()
    }

    /// Alias of [`tuple`].
    ///
    /// [`tuple`]: #method.tuple
    #[deprecated(since="0.1.5", note="Use `tuple` instead.")]
    #[inline]
    pub fn get_tuple(&self) -> (u8, u8, u8) {
        self.tuple()
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
    fn from(value: i32) -> Colour {
        Colour(value as u32)
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
    /// assert_eq!(Colour::from(6573123u32).r(), 100);
    /// ```
    fn from(value: u32) -> Colour {
        Colour(value)
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
    /// assert_eq!(Colour::from(6573123u64).r(), 100);
    /// ```
    fn from(value: u64) -> Colour {
        Colour(value as u32)
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
    /// Creates a new `Colour`, setting its RGB value to `(250, 177, 237)`.
    fabled_pink, 0xFAB1ED;
    /// Creates a new `Colour`, setting its RGB value to `(136, 130, 196)`.`
    faded_purple, 0x8882C4;
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
    /// Creates a new `Colour`, setting its RGB value to `(230, 131, 151)`.
    meibe_pink, 0xE68397;
    /// Creates a new `Colour`, setting its RGB value to `(230, 126, 34)`.
    orange, 0xE67E22;
    /// Creates a new `Colour`, setting its RGB value to `(155, 89, 182)`.
    purple, 0x9B59B6;
    /// Creates a new `Colour`, setting its RGB value to `(231, 76, 60)`.
    red, 0xE74C3C;
    /// Creates a new `Colour`, setting its RGB value to `(117, 150, 255)`.
    rohrkatze_blue, 0x7596FF;
    /// Creates a new `Colour`, setting its RGB value to `(246, 219, 216)`.
    rosewater, 0xF6DBD8;
    /// Creates a new `Colour`, setting its RGB value to `(26, 188, 156)`.
    teal, 0x1ABC9C;
}

impl Default for Colour {
    /// Creates a default value for a `Colour`, setting the inner value to `0`.
    fn default() -> Colour {
        Colour(0)
    }
}
