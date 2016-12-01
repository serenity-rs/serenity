use std::default::Default;
use ::internal::prelude::*;

macro_rules! colour {
    ($struct_:ident; $(#[$attr:meta] $name:ident, $val:expr;)*) => {
        impl $struct_ {
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
    Colour;
    /// Creates a new `Colour`, setting its RGB value to `(0, 72, 186)`.
    absolute_zero, 0x0048BA;
    /// Creates a new `Colour`, setting its RGB value to `(176, 191, 26)`.
    acid_green, 0xB0BF1A;
    /// Creates a new `Colour`, setting its RGB value to `(124, 185, 232)`.
    aero, 0x7CB9E8;
    /// Creates a new `Colour`, setting its RGB value to `(201, 255, 229)`.
    aero_blue, 0xC9FFE5;
    /// Creates a new `Colour`, setting its RGB value to `(178, 132, 190)`.
    african_violet, 0xB284BE;
    /// Creates a new `Colour`, setting its RGB value to `(93, 138, 168)`.
    air_force_blue_raf, 0x5D8AA8;
    /// Creates a new `Colour`, setting its RGB value to `(0, 48, 143)`.
    air_force_blue_usaf, 0x00308F;
    /// Creates a new `Colour`, setting its RGB value to `(114, 160, 193)`.
    air_superiority_blue, 0x72A0C1;
    /// Creates a new `Colour`, setting its RGB value to `(175, 0, 42)`.
    alabama_crimson, 0xAF002A;
    /// Creates a new `Colour`, setting its RGB value to `(242, 240, 230)`.
    alabaster, 0xF2F0E6;
    /// Creates a new `Colour`, setting its RGB value to `(240, 248, 255)`.
    alice_blue, 0xF0F8FF;
    /// Creates a new `Colour`, setting its RGB value to `(132, 222, 2)`.
    alien_armpit, 0x84DE02;
    /// Creates a new `Colour`, setting its RGB value to `(227, 38, 54)`.
    alizarin_crimson, 0xE32636;
    /// Creates a new `Colour`, setting its RGB value to `(196, 98, 16)`.
    alloy_orange, 0xC46210;
    /// Creates a new `Colour`, setting its RGB value to `(239, 222, 205)`.
    almond, 0xEFDECD;
    /// Creates a new `Colour`, setting its RGB value to `(229, 43, 80)`.
    amaranth, 0xE52B50;
    /// Creates a new `Colour`, setting its RGB value to `(159, 43, 104)`.
    amaranth_deep_purple, 0x9F2B68;
    /// Creates a new `Colour`, setting its RGB value to `(241, 156, 187)`.
    amaranth_pink, 0xF19CBB;
    /// Creates a new `Colour`, setting its RGB value to `(171, 39, 79)`.
    amaranth_purple, 0xAB274F;
    /// Creates a new `Colour`, setting its RGB value to `(211, 33, 45)`.
    amaranth_red, 0xD3212D;
    /// Creates a new `Colour`, setting its RGB value to `(59, 122, 87)`.
    amazon, 0x3B7A57;
    /// Creates a new `Colour`, setting its RGB value to `(0, 196, 176)`.
    amazonite, 0x00C4B0;
    /// Creates a new `Colour`, setting its RGB value to `(255, 191, 0)`.
    amber, 0xFFBF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 126, 0)`.
    amber_sae_ece, 0xFF7E00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 3, 62)`.
    american_rose, 0xFF033E;
    /// Creates a new `Colour`, setting its RGB value to `(153, 102, 204)`.
    amethyst, 0x9966CC;
    /// Creates a new `Colour`, setting its RGB value to `(164, 198, 57)`.
    android_green, 0xA4C639;
    /// Creates a new `Colour`, setting its RGB value to `(242, 243, 244)`.
    anti_flash_white, 0xF2F3F4;
    /// Creates a new `Colour`, setting its RGB value to `(205, 149, 117)`.
    antique_brass, 0xCD9575;
    /// Creates a new `Colour`, setting its RGB value to `(102, 93, 30)`.
    antique_bronze, 0x665D1E;
    /// Creates a new `Colour`, setting its RGB value to `(145, 92, 131)`.
    antique_fuchsia, 0x915C83;
    /// Creates a new `Colour`, setting its RGB value to `(132, 27, 45)`.
    antique_ruby, 0x841B2D;
    /// Creates a new `Colour`, setting its RGB value to `(250, 235, 215)`.
    antique_white, 0xFAEBD7;
    /// Creates a new `Colour`, setting its RGB value to `(0, 128, 0)`.
    ao_english, 0x008000;
    /// Creates a new `Colour`, setting its RGB value to `(141, 182, 0)`.
    apple_green, 0x8DB600;
    /// Creates a new `Colour`, setting its RGB value to `(251, 206, 177)`.
    apricot, 0xFBCEB1;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 255)`.
    aqua, 0x00FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(127, 255, 212)`.
    aquamarine, 0x7FFFD4;
    /// Creates a new `Colour`, setting its RGB value to `(208, 255, 20)`.
    arctic_lime, 0xD0FF14;
    /// Creates a new `Colour`, setting its RGB value to `(75, 83, 32)`.
    army_green, 0x4B5320;
    /// Creates a new `Colour`, setting its RGB value to `(59, 68, 75)`.
    arsenic, 0x3B444B;
    /// Creates a new `Colour`, setting its RGB value to `(143, 151, 121)`.
    artichoke, 0x8F9779;
    /// Creates a new `Colour`, setting its RGB value to `(233, 214, 107)`.
    arylide_yellow, 0xE9D66B;
    /// Creates a new `Colour`, setting its RGB value to `(178, 190, 181)`.
    ash_grey, 0xB2BEB5;
    /// Creates a new `Colour`, setting its RGB value to `(135, 169, 107)`.
    asparagus, 0x87A96B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 102)`.
    atomic_tangerine, 0xFF9966;
    /// Creates a new `Colour`, setting its RGB value to `(165, 42, 42)`.
    auburn, 0xA52A2A;
    /// Creates a new `Colour`, setting its RGB value to `(253, 238, 0)`.
    aureolin, 0xFDEE00;
    /// Creates a new `Colour`, setting its RGB value to `(110, 127, 128)`.
    aurometalsaurus, 0x6E7F80;
    /// Creates a new `Colour`, setting its RGB value to `(86, 130, 3)`.
    avocado, 0x568203;
    /// Creates a new `Colour`, setting its RGB value to `(255, 32, 82)`.
    awesome, 0xFF2052;
    /// Creates a new `Colour`, setting its RGB value to `(195, 153, 83)`.
    aztec_gold, 0xC39953;
    /// Creates a new `Colour`, setting its RGB value to `(0, 127, 255)`.
    azure, 0x007FFF;
    /// Creates a new `Colour`, setting its RGB value to `(240, 255, 255)`.
    azure_mist, 0xF0FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(240, 255, 255)`.
    azure_web_color, 0xF0FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(219, 233, 244)`.
    azureish_white, 0xDBE9F4;
    /// Creates a new `Colour`, setting its RGB value to `(137, 207, 240)`.
    baby_blue, 0x89CFF0;
    /// Creates a new `Colour`, setting its RGB value to `(161, 202, 241)`.
    baby_blue_eyes, 0xA1CAF1;
    /// Creates a new `Colour`, setting its RGB value to `(244, 194, 194)`.
    baby_pink, 0xF4C2C2;
    /// Creates a new `Colour`, setting its RGB value to `(254, 254, 250)`.
    baby_powder, 0xFEFEFA;
    /// Creates a new `Colour`, setting its RGB value to `(255, 145, 175)`.
    baker_miller_pink, 0xFF91AF;
    /// Creates a new `Colour`, setting its RGB value to `(33, 171, 205)`.
    ball_blue, 0x21ABCD;
    /// Creates a new `Colour`, setting its RGB value to `(250, 231, 181)`.
    banana_mania, 0xFAE7B5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 225, 53)`.
    banana_yellow, 0xFFE135;
    /// Creates a new `Colour`, setting its RGB value to `(0, 106, 78)`.
    bangladesh_green, 0x006A4E;
    /// Creates a new `Colour`, setting its RGB value to `(224, 33, 138)`.
    barbie_pink, 0xE0218A;
    /// Creates a new `Colour`, setting its RGB value to `(124, 10, 2)`.
    barn_red, 0x7C0A02;
    /// Creates a new `Colour`, setting its RGB value to `(29, 172, 214)`.
    battery_charged_blue, 0x1DACD6;
    /// Creates a new `Colour`, setting its RGB value to `(132, 132, 130)`.
    battleship_grey, 0x848482;
    /// Creates a new `Colour`, setting its RGB value to `(152, 119, 123)`.
    bazaar, 0x98777B;
    /// Creates a new `Colour`, setting its RGB value to `(46, 88, 148)`.
    bdazzled_blue, 0x2E5894;
    /// Creates a new `Colour`, setting its RGB value to `(188, 212, 230)`.
    beau_blue, 0xBCD4E6;
    /// Creates a new `Colour`, setting its RGB value to `(159, 129, 112)`.
    beaver, 0x9F8170;
    /// Creates a new `Colour`, setting its RGB value to `(250, 110, 121)`.
    begonia, 0xFA6E79;
    /// Creates a new `Colour`, setting its RGB value to `(245, 245, 220)`.
    beige, 0xF5F5DC;
    /// Creates a new `Colour`, setting its RGB value to `(156, 37, 66)`.
    big_dip_o_ruby, 0x9C2542;
    /// Creates a new `Colour`, setting its RGB value to `(232, 142, 90)`.
    big_foot_feet, 0xE88E5A;
    /// Creates a new `Colour`, setting its RGB value to `(255, 228, 196)`.
    bisque, 0xFFE4C4;
    /// Creates a new `Colour`, setting its RGB value to `(61, 43, 31)`.
    bistre, 0x3D2B1F;
    /// Creates a new `Colour`, setting its RGB value to `(150, 113, 23)`.
    bistre_brown, 0x967117;
    /// Creates a new `Colour`, setting its RGB value to `(202, 224, 13)`.
    bitter_lemon, 0xCAE00D;
    /// Creates a new `Colour`, setting its RGB value to `(191, 255, 0)`.
    bitter_lime, 0xBFFF00;
    /// Creates a new `Colour`, setting its RGB value to `(254, 111, 94)`.
    bittersweet, 0xFE6F5E;
    /// Creates a new `Colour`, setting its RGB value to `(191, 79, 81)`.
    bittersweet_shimmer, 0xBF4F51;
    /// Creates a new `Colour`, setting its RGB value to `(0, 0, 0)`.
    black, 0x000000;
    /// Creates a new `Colour`, setting its RGB value to `(61, 12, 2)`.
    black_bean, 0x3D0C02;
    /// Creates a new `Colour`, setting its RGB value to `(84, 98, 111)`.
    black_coral, 0x54626F;
    /// Creates a new `Colour`, setting its RGB value to `(37, 53, 41)`.
    black_leather_jacket, 0x253529;
    /// Creates a new `Colour`, setting its RGB value to `(59, 60, 54)`.
    black_olive, 0x3B3C36;
    /// Creates a new `Colour`, setting its RGB value to `(191, 175, 178)`.
    black_shadows, 0xBFAFB2;
    /// Creates a new `Colour`, setting its RGB value to `(255, 235, 205)`.
    blanched_almond, 0xFFEBCD;
    /// Creates a new `Colour`, setting its RGB value to `(165, 113, 100)`.
    blast_off_bronze, 0xA57164;
    /// Creates a new `Colour`, setting its RGB value to `(49, 140, 231)`.
    bleu_de_france, 0x318CE7;
    /// Creates a new `Colour`, setting its RGB value to `(111, 198, 226)`.
    blitz_blue, 0x6FC6E2;
    /// Creates a new `Colour`, setting its RGB value to `(172, 229, 238)`.
    blizzard_blue, 0xACE5EE;
    /// Creates a new `Colour`, setting its RGB value to `(250, 240, 190)`.
    blond, 0xFAF0BE;
    /// Creates a new `Colour`, setting its RGB value to `(52, 152, 219)`.
    blue, 0x3498DB;
    /// Creates a new `Colour`, setting its RGB value to `(162, 162, 208)`.
    blue_bell, 0xA2A2D0;
    /// Creates a new `Colour`, setting its RGB value to `(0, 185, 251)`.
    blue_bolt, 0x00B9FB;
    /// Creates a new `Colour`, setting its RGB value to `(31, 117, 254)`.
    blue_crayola, 0x1F75FE;
    /// Creates a new `Colour`, setting its RGB value to `(102, 153, 204)`.
    blue_gray, 0x6699CC;
    /// Creates a new `Colour`, setting its RGB value to `(13, 152, 186)`.
    blue_green, 0x0D98BA;
    /// Creates a new `Colour`, setting its RGB value to `(93, 173, 236)`.
    blue_jeans, 0x5DADEC;
    /// Creates a new `Colour`, setting its RGB value to `(172, 229, 238)`.
    blue_lagoon, 0xACE5EE;
    /// Creates a new `Colour`, setting its RGB value to `(85, 53, 146)`.
    blue_magenta_violet, 0x553592;
    /// Creates a new `Colour`, setting its RGB value to `(0, 147, 175)`.
    blue_munsell, 0x0093AF;
    /// Creates a new `Colour`, setting its RGB value to `(0, 135, 189)`.
    blue_ncs, 0x0087BD;
    /// Creates a new `Colour`, setting its RGB value to `(0, 24, 168)`.
    blue_pantone, 0x0018A8;
    /// Creates a new `Colour`, setting its RGB value to `(51, 51, 153)`.
    blue_pigment, 0x333399;
    /// Creates a new `Colour`, setting its RGB value to `(2, 71, 254)`.
    blue_ryb, 0x0247FE;
    /// Creates a new `Colour`, setting its RGB value to `(18, 97, 128)`.
    blue_sapphire, 0x126180;
    /// Creates a new `Colour`, setting its RGB value to `(138, 43, 226)`.
    blue_violet, 0x8A2BE2;
    /// Creates a new `Colour`, setting its RGB value to `(80, 114, 167)`.
    blue_yonder, 0x5072A7;
    /// Creates a new `Colour`, setting its RGB value to `(79, 134, 247)`.
    blueberry, 0x4F86F7;
    /// Creates a new `Colour`, setting its RGB value to `(28, 28, 240)`.
    bluebonnet, 0x1C1CF0;
    /// Creates a new `Colour`, setting its RGB value to `(114, 137, 218)`.
    blurple, 0x7289DA;
    /// Creates a new `Colour`, setting its RGB value to `(222, 93, 131)`.
    blush, 0xDE5D83;
    /// Creates a new `Colour`, setting its RGB value to `(121, 68, 59)`.
    bole, 0x79443B;
    /// Creates a new `Colour`, setting its RGB value to `(0, 149, 182)`.
    bondi_blue, 0x0095B6;
    /// Creates a new `Colour`, setting its RGB value to `(227, 218, 201)`.
    bone, 0xE3DAC9;
    /// Creates a new `Colour`, setting its RGB value to `(221, 226, 106)`.
    booger_buster, 0xDDE26A;
    /// Creates a new `Colour`, setting its RGB value to `(204, 0, 0)`.
    boston_university_red, 0xCC0000;
    /// Creates a new `Colour`, setting its RGB value to `(0, 106, 78)`.
    bottle_green, 0x006A4E;
    /// Creates a new `Colour`, setting its RGB value to `(135, 50, 96)`.
    boysenberry, 0x873260;
    /// Creates a new `Colour`, setting its RGB value to `(0, 112, 255)`.
    brandeis_blue, 0x0070FF;
    /// Creates a new `Colour`, setting its RGB value to `(181, 166, 66)`.
    brass, 0xB5A642;
    /// Creates a new `Colour`, setting its RGB value to `(203, 65, 84)`.
    brick_red, 0xCB4154;
    /// Creates a new `Colour`, setting its RGB value to `(29, 172, 214)`.
    bright_cerulean, 0x1DACD6;
    /// Creates a new `Colour`, setting its RGB value to `(102, 255, 0)`.
    bright_green, 0x66FF00;
    /// Creates a new `Colour`, setting its RGB value to `(191, 148, 228)`.
    bright_lavender, 0xBF94E4;
    /// Creates a new `Colour`, setting its RGB value to `(216, 145, 239)`.
    bright_lilac, 0xD891EF;
    /// Creates a new `Colour`, setting its RGB value to `(195, 33, 72)`.
    bright_maroon, 0xC32148;
    /// Creates a new `Colour`, setting its RGB value to `(25, 116, 210)`.
    bright_navy_blue, 0x1974D2;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 127)`.
    bright_pink, 0xFF007F;
    /// Creates a new `Colour`, setting its RGB value to `(8, 232, 222)`.
    bright_turquoise, 0x08E8DE;
    /// Creates a new `Colour`, setting its RGB value to `(209, 159, 232)`.
    bright_ube, 0xD19FE8;
    /// Creates a new `Colour`, setting its RGB value to `(255, 170, 29)`.
    bright_yellow_crayola, 0xFFAA1D;
    /// Creates a new `Colour`, setting its RGB value to `(51, 153, 255)`.
    brilliant_azure, 0x3399FF;
    /// Creates a new `Colour`, setting its RGB value to `(244, 187, 255)`.
    brilliant_lavender, 0xF4BBFF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 85, 163)`.
    brilliant_rose, 0xFF55A3;
    /// Creates a new `Colour`, setting its RGB value to `(251, 96, 127)`.
    brink_pink, 0xFB607F;
    /// Creates a new `Colour`, setting its RGB value to `(0, 66, 37)`.
    british_racing_green, 0x004225;
    /// Creates a new `Colour`, setting its RGB value to `(205, 127, 50)`.
    bronze, 0xCD7F32;
    /// Creates a new `Colour`, setting its RGB value to `(115, 112, 0)`.
    bronze_yellow, 0x737000;
    /// Creates a new `Colour`, setting its RGB value to `(107, 68, 35)`.
    brown_nose, 0x6B4423;
    /// Creates a new `Colour`, setting its RGB value to `(175, 110, 77)`.
    brown_sugar, 0xAF6E4D;
    /// Creates a new `Colour`, setting its RGB value to `(150, 75, 0)`.
    brown_traditional, 0x964B00;
    /// Creates a new `Colour`, setting its RGB value to `(165, 42, 42)`.
    brown_web, 0xA52A2A;
    /// Creates a new `Colour`, setting its RGB value to `(204, 153, 102)`.
    brown_yellow, 0xCC9966;
    /// Creates a new `Colour`, setting its RGB value to `(27, 77, 62)`.
    brunswick_green, 0x1B4D3E;
    /// Creates a new `Colour`, setting its RGB value to `(255, 193, 204)`.
    bubble_gum, 0xFFC1CC;
    /// Creates a new `Colour`, setting its RGB value to `(231, 254, 255)`.
    bubbles, 0xE7FEFF;
    /// Creates a new `Colour`, setting its RGB value to `(123, 182, 97)`.
    bud_green, 0x7BB661;
    /// Creates a new `Colour`, setting its RGB value to `(240, 220, 130)`.
    buff, 0xF0DC82;
    /// Creates a new `Colour`, setting its RGB value to `(72, 6, 7)`.
    bulgarian_rose, 0x480607;
    /// Creates a new `Colour`, setting its RGB value to `(128, 0, 32)`.
    burgundy, 0x800020;
    /// Creates a new `Colour`, setting its RGB value to `(222, 184, 135)`.
    burlywood, 0xDEB887;
    /// Creates a new `Colour`, setting its RGB value to `(161, 122, 116)`.
    burnished_brown, 0xA17A74;
    /// Creates a new `Colour`, setting its RGB value to `(204, 85, 0)`.
    burnt_orange, 0xCC5500;
    /// Creates a new `Colour`, setting its RGB value to `(233, 116, 81)`.
    burnt_sienna, 0xE97451;
    /// Creates a new `Colour`, setting its RGB value to `(138, 51, 36)`.
    burnt_umber, 0x8A3324;
    /// Creates a new `Colour`, setting its RGB value to `(36, 160, 237)`.
    button_blue, 0x24A0ED;
    /// Creates a new `Colour`, setting its RGB value to `(189, 51, 164)`.
    byzantine, 0xBD33A4;
    /// Creates a new `Colour`, setting its RGB value to `(112, 41, 99)`.
    byzantium, 0x702963;
    /// Creates a new `Colour`, setting its RGB value to `(83, 104, 114)`.
    cadet, 0x536872;
    /// Creates a new `Colour`, setting its RGB value to `(95, 158, 160)`.
    cadet_blue, 0x5F9EA0;
    /// Creates a new `Colour`, setting its RGB value to `(145, 163, 176)`.
    cadet_grey, 0x91A3B0;
    /// Creates a new `Colour`, setting its RGB value to `(0, 107, 60)`.
    cadmium_green, 0x006B3C;
    /// Creates a new `Colour`, setting its RGB value to `(237, 135, 45)`.
    cadmium_orange, 0xED872D;
    /// Creates a new `Colour`, setting its RGB value to `(227, 0, 34)`.
    cadmium_red, 0xE30022;
    /// Creates a new `Colour`, setting its RGB value to `(255, 246, 0)`.
    cadmium_yellow, 0xFFF600;
    /// Creates a new `Colour`, setting its RGB value to `(166, 123, 91)`.
    caf_au_lait, 0xA67B5B;
    /// Creates a new `Colour`, setting its RGB value to `(75, 54, 33)`.
    caf_noir, 0x4B3621;
    /// Creates a new `Colour`, setting its RGB value to `(30, 77, 43)`.
    cal_poly_pomona_green, 0x1E4D2B;
    /// Creates a new `Colour`, setting its RGB value to `(163, 193, 173)`.
    cambridge_blue, 0xA3C1AD;
    /// Creates a new `Colour`, setting its RGB value to `(193, 154, 107)`.
    camel, 0xC19A6B;
    /// Creates a new `Colour`, setting its RGB value to `(239, 187, 204)`.
    cameo_pink, 0xEFBBCC;
    /// Creates a new `Colour`, setting its RGB value to `(120, 134, 107)`.
    camouflage_green, 0x78866B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 153)`.
    canary, 0xFFFF99;
    /// Creates a new `Colour`, setting its RGB value to `(255, 239, 0)`.
    canary_yellow, 0xFFEF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 8, 0)`.
    candy_apple_red, 0xFF0800;
    /// Creates a new `Colour`, setting its RGB value to `(228, 113, 122)`.
    candy_pink, 0xE4717A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 191, 255)`.
    capri, 0x00BFFF;
    /// Creates a new `Colour`, setting its RGB value to `(89, 39, 32)`.
    caput_mortuum, 0x592720;
    /// Creates a new `Colour`, setting its RGB value to `(196, 30, 58)`.
    cardinal, 0xC41E3A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 204, 153)`.
    caribbean_green, 0x00CC99;
    /// Creates a new `Colour`, setting its RGB value to `(150, 0, 24)`.
    carmine, 0x960018;
    /// Creates a new `Colour`, setting its RGB value to `(215, 0, 64)`.
    carmine_mp, 0xD70040;
    /// Creates a new `Colour`, setting its RGB value to `(235, 76, 66)`.
    carmine_pink, 0xEB4C42;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 56)`.
    carmine_red, 0xFF0038;
    /// Creates a new `Colour`, setting its RGB value to `(255, 166, 201)`.
    carnation_pink, 0xFFA6C9;
    /// Creates a new `Colour`, setting its RGB value to `(179, 27, 27)`.
    carnelian, 0xB31B1B;
    /// Creates a new `Colour`, setting its RGB value to `(86, 160, 211)`.
    carolina_blue, 0x56A0D3;
    /// Creates a new `Colour`, setting its RGB value to `(237, 145, 33)`.
    carrot_orange, 0xED9121;
    /// Creates a new `Colour`, setting its RGB value to `(0, 86, 63)`.
    castleton_green, 0x00563F;
    /// Creates a new `Colour`, setting its RGB value to `(6, 42, 120)`.
    catalina_blue, 0x062A78;
    /// Creates a new `Colour`, setting its RGB value to `(112, 54, 66)`.
    catawba, 0x703642;
    /// Creates a new `Colour`, setting its RGB value to `(201, 90, 73)`.
    cedar_chest, 0xC95A49;
    /// Creates a new `Colour`, setting its RGB value to `(146, 161, 207)`.
    ceil, 0x92A1CF;
    /// Creates a new `Colour`, setting its RGB value to `(172, 225, 175)`.
    celadon, 0xACE1AF;
    /// Creates a new `Colour`, setting its RGB value to `(0, 123, 167)`.
    celadon_blue, 0x007BA7;
    /// Creates a new `Colour`, setting its RGB value to `(47, 132, 124)`.
    celadon_green, 0x2F847C;
    /// Creates a new `Colour`, setting its RGB value to `(178, 255, 255)`.
    celeste, 0xB2FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(73, 151, 208)`.
    celestial_blue, 0x4997D0;
    /// Creates a new `Colour`, setting its RGB value to `(222, 49, 99)`.
    cerise, 0xDE3163;
    /// Creates a new `Colour`, setting its RGB value to `(236, 59, 131)`.
    cerise_pink, 0xEC3B83;
    /// Creates a new `Colour`, setting its RGB value to `(0, 123, 167)`.
    cerulean, 0x007BA7;
    /// Creates a new `Colour`, setting its RGB value to `(42, 82, 190)`.
    cerulean_blue, 0x2A52BE;
    /// Creates a new `Colour`, setting its RGB value to `(109, 155, 195)`.
    cerulean_frost, 0x6D9BC3;
    /// Creates a new `Colour`, setting its RGB value to `(0, 122, 165)`.
    cg_blue, 0x007AA5;
    /// Creates a new `Colour`, setting its RGB value to `(224, 60, 49)`.
    cg_red, 0xE03C31;
    /// Creates a new `Colour`, setting its RGB value to `(160, 120, 90)`.
    chamoisee, 0xA0785A;
    /// Creates a new `Colour`, setting its RGB value to `(247, 231, 206)`.
    champagne, 0xF7E7CE;
    /// Creates a new `Colour`, setting its RGB value to `(241, 221, 207)`.
    champagne_pink, 0xF1DDCF;
    /// Creates a new `Colour`, setting its RGB value to `(54, 69, 79)`.
    charcoal, 0x36454F;
    /// Creates a new `Colour`, setting its RGB value to `(35, 43, 43)`.
    charleston_green, 0x232B2B;
    /// Creates a new `Colour`, setting its RGB value to `(230, 143, 172)`.
    charm_pink, 0xE68FAC;
    /// Creates a new `Colour`, setting its RGB value to `(223, 255, 0)`.
    chartreuse_traditional, 0xDFFF00;
    /// Creates a new `Colour`, setting its RGB value to `(127, 255, 0)`.
    chartreuse_web, 0x7FFF00;
    /// Creates a new `Colour`, setting its RGB value to `(222, 49, 99)`.
    cherry, 0xDE3163;
    /// Creates a new `Colour`, setting its RGB value to `(255, 183, 197)`.
    cherry_blossom_pink, 0xFFB7C5;
    /// Creates a new `Colour`, setting its RGB value to `(149, 69, 53)`.
    chestnut, 0x954535;
    /// Creates a new `Colour`, setting its RGB value to `(222, 111, 161)`.
    china_pink, 0xDE6FA1;
    /// Creates a new `Colour`, setting its RGB value to `(168, 81, 110)`.
    china_rose, 0xA8516E;
    /// Creates a new `Colour`, setting its RGB value to `(170, 56, 30)`.
    chinese_red, 0xAA381E;
    /// Creates a new `Colour`, setting its RGB value to `(133, 96, 136)`.
    chinese_violet, 0x856088;
    /// Creates a new `Colour`, setting its RGB value to `(74, 255, 0)`.
    chlorophyll_green, 0x4AFF00;
    /// Creates a new `Colour`, setting its RGB value to `(123, 63, 0)`.
    chocolate_traditional, 0x7B3F00;
    /// Creates a new `Colour`, setting its RGB value to `(210, 105, 30)`.
    chocolate_web, 0xD2691E;
    /// Creates a new `Colour`, setting its RGB value to `(255, 167, 0)`.
    chrome_yellow, 0xFFA700;
    /// Creates a new `Colour`, setting its RGB value to `(152, 129, 123)`.
    cinereous, 0x98817B;
    /// Creates a new `Colour`, setting its RGB value to `(227, 66, 52)`.
    cinnabar, 0xE34234;
    /// Creates a new `Colour`, setting its RGB value to `(210, 105, 30)`.
    cinnamon, 0xD2691E;
    /// Creates a new `Colour`, setting its RGB value to `(205, 96, 126)`.
    cinnamon_satin, 0xCD607E;
    /// Creates a new `Colour`, setting its RGB value to `(228, 208, 10)`.
    citrine, 0xE4D00A;
    /// Creates a new `Colour`, setting its RGB value to `(159, 169, 31)`.
    citron, 0x9FA91F;
    /// Creates a new `Colour`, setting its RGB value to `(127, 23, 52)`.
    claret, 0x7F1734;
    /// Creates a new `Colour`, setting its RGB value to `(251, 204, 231)`.
    classic_rose, 0xFBCCE7;
    /// Creates a new `Colour`, setting its RGB value to `(0, 71, 171)`.
    cobalt_blue, 0x0047AB;
    /// Creates a new `Colour`, setting its RGB value to `(210, 105, 30)`.
    cocoa_brown, 0xD2691E;
    /// Creates a new `Colour`, setting its RGB value to `(150, 90, 62)`.
    coconut, 0x965A3E;
    /// Creates a new `Colour`, setting its RGB value to `(111, 78, 55)`.
    coffee, 0x6F4E37;
    /// Creates a new `Colour`, setting its RGB value to `(196, 216, 226)`.
    columbia_blue, 0xC4D8E2;
    /// Creates a new `Colour`, setting its RGB value to `(248, 131, 121)`.
    congo_pink, 0xF88379;
    /// Creates a new `Colour`, setting its RGB value to `(0, 46, 99)`.
    cool_black, 0x002E63;
    /// Creates a new `Colour`, setting its RGB value to `(140, 146, 172)`.
    cool_grey, 0x8C92AC;
    /// Creates a new `Colour`, setting its RGB value to `(184, 115, 51)`.
    copper, 0xB87333;
    /// Creates a new `Colour`, setting its RGB value to `(218, 138, 103)`.
    copper_crayola, 0xDA8A67;
    /// Creates a new `Colour`, setting its RGB value to `(173, 111, 105)`.
    copper_penny, 0xAD6F69;
    /// Creates a new `Colour`, setting its RGB value to `(203, 109, 81)`.
    copper_red, 0xCB6D51;
    /// Creates a new `Colour`, setting its RGB value to `(153, 102, 102)`.
    copper_rose, 0x996666;
    /// Creates a new `Colour`, setting its RGB value to `(255, 56, 0)`.
    coquelicot, 0xFF3800;
    /// Creates a new `Colour`, setting its RGB value to `(255, 127, 80)`.
    coral, 0xFF7F50;
    /// Creates a new `Colour`, setting its RGB value to `(248, 131, 121)`.
    coral_pink, 0xF88379;
    /// Creates a new `Colour`, setting its RGB value to `(255, 64, 64)`.
    coral_red, 0xFF4040;
    /// Creates a new `Colour`, setting its RGB value to `(253, 124, 110)`.
    coral_reef, 0xFD7C6E;
    /// Creates a new `Colour`, setting its RGB value to `(137, 63, 69)`.
    cordovan, 0x893F45;
    /// Creates a new `Colour`, setting its RGB value to `(251, 236, 93)`.
    corn, 0xFBEC5D;
    /// Creates a new `Colour`, setting its RGB value to `(179, 27, 27)`.
    cornell_red, 0xB31B1B;
    /// Creates a new `Colour`, setting its RGB value to `(100, 149, 237)`.
    cornflower_blue, 0x6495ED;
    /// Creates a new `Colour`, setting its RGB value to `(255, 248, 220)`.
    cornsilk, 0xFFF8DC;
    /// Creates a new `Colour`, setting its RGB value to `(46, 45, 136)`.
    cosmic_cobalt, 0x2E2D88;
    /// Creates a new `Colour`, setting its RGB value to `(255, 248, 231)`.
    cosmic_latte, 0xFFF8E7;
    /// Creates a new `Colour`, setting its RGB value to `(255, 188, 217)`.
    cotton_candy, 0xFFBCD9;
    /// Creates a new `Colour`, setting its RGB value to `(129, 97, 60)`.
    coyote_brown, 0x81613C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 253, 208)`.
    cream, 0xFFFDD0;
    /// Creates a new `Colour`, setting its RGB value to `(220, 20, 60)`.
    crimson, 0xDC143C;
    /// Creates a new `Colour`, setting its RGB value to `(190, 0, 50)`.
    crimson_glory, 0xBE0032;
    /// Creates a new `Colour`, setting its RGB value to `(153, 0, 0)`.
    crimson_red, 0x990000;
    /// Creates a new `Colour`, setting its RGB value to `(245, 245, 245)`.
    cultured, 0xF5F5F5;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 255)`.
    cyan, 0x00FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(78, 130, 180)`.
    cyan_azure, 0x4E82B4;
    /// Creates a new `Colour`, setting its RGB value to `(70, 130, 191)`.
    cyan_blue_azure, 0x4682BF;
    /// Creates a new `Colour`, setting its RGB value to `(40, 88, 156)`.
    cyan_cobalt_blue, 0x28589C;
    /// Creates a new `Colour`, setting its RGB value to `(24, 139, 194)`.
    cyan_cornflower_blue, 0x188BC2;
    /// Creates a new `Colour`, setting its RGB value to `(0, 183, 235)`.
    cyan_process, 0x00B7EB;
    /// Creates a new `Colour`, setting its RGB value to `(88, 66, 124)`.
    cyber_grape, 0x58427C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 211, 0)`.
    cyber_yellow, 0xFFD300;
    /// Creates a new `Colour`, setting its RGB value to `(245, 111, 161)`.
    cyclamen, 0xF56FA1;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 49)`.
    daffodil, 0xFFFF31;
    /// Creates a new `Colour`, setting its RGB value to `(240, 225, 48)`.
    dandelion, 0xF0E130;
    /// Creates a new `Colour`, setting its RGB value to `(32, 102, 148)`.
    dark_blue, 0x206694;
    /// Creates a new `Colour`, setting its RGB value to `(102, 102, 153)`.
    dark_blue_gray, 0x666699;
    /// Creates a new `Colour`, setting its RGB value to `(101, 67, 33)`.
    dark_brown, 0x654321;
    /// Creates a new `Colour`, setting its RGB value to `(136, 101, 78)`.
    dark_brown_tangelo, 0x88654E;
    /// Creates a new `Colour`, setting its RGB value to `(93, 57, 84)`.
    dark_byzantium, 0x5D3954;
    /// Creates a new `Colour`, setting its RGB value to `(164, 0, 0)`.
    dark_candy_apple_red, 0xA40000;
    /// Creates a new `Colour`, setting its RGB value to `(8, 69, 126)`.
    dark_cerulean, 0x08457E;
    /// Creates a new `Colour`, setting its RGB value to `(152, 105, 96)`.
    dark_chestnut, 0x986960;
    /// Creates a new `Colour`, setting its RGB value to `(205, 91, 69)`.
    dark_coral, 0xCD5B45;
    /// Creates a new `Colour`, setting its RGB value to `(0, 139, 139)`.
    dark_cyan, 0x008B8B;
    /// Creates a new `Colour`, setting its RGB value to `(83, 104, 120)`.
    dark_electric_blue, 0x536878;
    /// Creates a new `Colour`, setting its RGB value to `(194, 124, 14)`.
    dark_gold, 0xC27C0E;
    /// Creates a new `Colour`, setting its RGB value to `(184, 134, 11)`.
    dark_goldenrod, 0xB8860B;
    /// Creates a new `Colour`, setting its RGB value to `(169, 169, 169)`.
    dark_gray_x11, 0xA9A9A9;
    /// Creates a new `Colour`, setting its RGB value to `(31, 139, 76)`.
    dark_green, 0x1F8B4C;
    /// Creates a new `Colour`, setting its RGB value to `(0, 100, 0)`.
    dark_green_x11, 0x006400;
    /// Creates a new `Colour`, setting its RGB value to `(96, 125, 139)`.
    dark_grey, 0x607D8B;
    /// Creates a new `Colour`, setting its RGB value to `(31, 38, 42)`.
    dark_gunmetal, 0x1F262A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 20, 126)`.
    dark_imperial_blue, 0x00147E;
    /// Creates a new `Colour`, setting its RGB value to `(26, 36, 33)`.
    dark_jungle_green, 0x1A2421;
    /// Creates a new `Colour`, setting its RGB value to `(189, 183, 107)`.
    dark_khaki, 0xBDB76B;
    /// Creates a new `Colour`, setting its RGB value to `(72, 60, 50)`.
    dark_lava, 0x483C32;
    /// Creates a new `Colour`, setting its RGB value to `(115, 79, 150)`.
    dark_lavender, 0x734F96;
    /// Creates a new `Colour`, setting its RGB value to `(83, 75, 79)`.
    dark_liver, 0x534B4F;
    /// Creates a new `Colour`, setting its RGB value to `(84, 61, 55)`.
    dark_liver_horses, 0x543D37;
    /// Creates a new `Colour`, setting its RGB value to `(173, 20, 87)`.
    dark_magenta, 0xAD1457;
    /// Creates a new `Colour`, setting its RGB value to `(169, 169, 169)`.
    dark_medium_gray, 0xA9A9A9;
    /// Creates a new `Colour`, setting its RGB value to `(0, 51, 102)`.
    dark_midnight_blue, 0x003366;
    /// Creates a new `Colour`, setting its RGB value to `(74, 93, 35)`.
    dark_moss_green, 0x4A5D23;
    /// Creates a new `Colour`, setting its RGB value to `(85, 107, 47)`.
    dark_olive_green, 0x556B2F;
    /// Creates a new `Colour`, setting its RGB value to `(168, 67, 0)`.
    dark_orange, 0xA84300;
    /// Creates a new `Colour`, setting its RGB value to `(153, 50, 204)`.
    dark_orchid, 0x9932CC;
    /// Creates a new `Colour`, setting its RGB value to `(119, 158, 203)`.
    dark_pastel_blue, 0x779ECB;
    /// Creates a new `Colour`, setting its RGB value to `(3, 192, 60)`.
    dark_pastel_green, 0x03C03C;
    /// Creates a new `Colour`, setting its RGB value to `(150, 111, 214)`.
    dark_pastel_purple, 0x966FD6;
    /// Creates a new `Colour`, setting its RGB value to `(194, 59, 34)`.
    dark_pastel_red, 0xC23B22;
    /// Creates a new `Colour`, setting its RGB value to `(231, 84, 128)`.
    dark_pink, 0xE75480;
    /// Creates a new `Colour`, setting its RGB value to `(0, 51, 153)`.
    dark_powder_blue, 0x003399;
    /// Creates a new `Colour`, setting its RGB value to `(79, 58, 60)`.
    dark_puce, 0x4F3A3C;
    /// Creates a new `Colour`, setting its RGB value to `(113, 54, 138)`.
    dark_purple, 0x71368A;
    /// Creates a new `Colour`, setting its RGB value to `(135, 38, 87)`.
    dark_raspberry, 0x872657;
    /// Creates a new `Colour`, setting its RGB value to `(153, 45, 34)`.
    dark_red, 0x992D22;
    /// Creates a new `Colour`, setting its RGB value to `(233, 150, 122)`.
    dark_salmon, 0xE9967A;
    /// Creates a new `Colour`, setting its RGB value to `(86, 3, 25)`.
    dark_scarlet, 0x560319;
    /// Creates a new `Colour`, setting its RGB value to `(143, 188, 143)`.
    dark_sea_green, 0x8FBC8F;
    /// Creates a new `Colour`, setting its RGB value to `(60, 20, 20)`.
    dark_sienna, 0x3C1414;
    /// Creates a new `Colour`, setting its RGB value to `(140, 190, 214)`.
    dark_sky_blue, 0x8CBED6;
    /// Creates a new `Colour`, setting its RGB value to `(72, 61, 139)`.
    dark_slate_blue, 0x483D8B;
    /// Creates a new `Colour`, setting its RGB value to `(47, 79, 79)`.
    dark_slate_gray, 0x2F4F4F;
    /// Creates a new `Colour`, setting its RGB value to `(23, 114, 69)`.
    dark_spring_green, 0x177245;
    /// Creates a new `Colour`, setting its RGB value to `(145, 129, 81)`.
    dark_tan, 0x918151;
    /// Creates a new `Colour`, setting its RGB value to `(255, 168, 18)`.
    dark_tangerine, 0xFFA812;
    /// Creates a new `Colour`, setting its RGB value to `(72, 60, 50)`.
    dark_taupe, 0x483C32;
    /// Creates a new `Colour`, setting its RGB value to `(17, 128, 106)`.
    dark_teal, 0x11806A;
    /// Creates a new `Colour`, setting its RGB value to `(204, 78, 92)`.
    dark_terra_cotta, 0xCC4E5C;
    /// Creates a new `Colour`, setting its RGB value to `(0, 206, 209)`.
    dark_turquoise, 0x00CED1;
    /// Creates a new `Colour`, setting its RGB value to `(209, 190, 168)`.
    dark_vanilla, 0xD1BEA8;
    /// Creates a new `Colour`, setting its RGB value to `(148, 0, 211)`.
    dark_violet, 0x9400D3;
    /// Creates a new `Colour`, setting its RGB value to `(155, 135, 12)`.
    dark_yellow, 0x9B870C;
    /// Creates a new `Colour`, setting its RGB value to `(84, 110, 122)`.
    darker_grey, 0x546E7A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 112, 60)`.
    dartmouth_green, 0x00703C;
    /// Creates a new `Colour`, setting its RGB value to `(85, 85, 85)`.
    davys_grey, 0x555555;
    /// Creates a new `Colour`, setting its RGB value to `(215, 10, 83)`.
    debian_red, 0xD70A53;
    /// Creates a new `Colour`, setting its RGB value to `(64, 130, 109)`.
    deep_aquamarine, 0x40826D;
    /// Creates a new `Colour`, setting its RGB value to `(169, 32, 62)`.
    deep_carmine, 0xA9203E;
    /// Creates a new `Colour`, setting its RGB value to `(239, 48, 56)`.
    deep_carmine_pink, 0xEF3038;
    /// Creates a new `Colour`, setting its RGB value to `(233, 105, 44)`.
    deep_carrot_orange, 0xE9692C;
    /// Creates a new `Colour`, setting its RGB value to `(218, 50, 135)`.
    deep_cerise, 0xDA3287;
    /// Creates a new `Colour`, setting its RGB value to `(250, 214, 165)`.
    deep_champagne, 0xFAD6A5;
    /// Creates a new `Colour`, setting its RGB value to `(185, 78, 72)`.
    deep_chestnut, 0xB94E48;
    /// Creates a new `Colour`, setting its RGB value to `(112, 66, 65)`.
    deep_coffee, 0x704241;
    /// Creates a new `Colour`, setting its RGB value to `(193, 84, 193)`.
    deep_fuchsia, 0xC154C1;
    /// Creates a new `Colour`, setting its RGB value to `(5, 102, 8)`.
    deep_green, 0x056608;
    /// Creates a new `Colour`, setting its RGB value to `(14, 124, 97)`.
    deep_green_cyan_turquoise, 0x0E7C61;
    /// Creates a new `Colour`, setting its RGB value to `(0, 75, 73)`.
    deep_jungle_green, 0x004B49;
    /// Creates a new `Colour`, setting its RGB value to `(51, 51, 102)`.
    deep_koamaru, 0x333366;
    /// Creates a new `Colour`, setting its RGB value to `(245, 199, 26)`.
    deep_lemon, 0xF5C71A;
    /// Creates a new `Colour`, setting its RGB value to `(153, 85, 187)`.
    deep_lilac, 0x9955BB;
    /// Creates a new `Colour`, setting its RGB value to `(204, 0, 204)`.
    deep_magenta, 0xCC00CC;
    /// Creates a new `Colour`, setting its RGB value to `(130, 0, 0)`.
    deep_maroon, 0x820000;
    /// Creates a new `Colour`, setting its RGB value to `(212, 115, 212)`.
    deep_mauve, 0xD473D4;
    /// Creates a new `Colour`, setting its RGB value to `(53, 94, 59)`.
    deep_moss_green, 0x355E3B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 203, 164)`.
    deep_peach, 0xFFCBA4;
    /// Creates a new `Colour`, setting its RGB value to `(255, 20, 147)`.
    deep_pink, 0xFF1493;
    /// Creates a new `Colour`, setting its RGB value to `(169, 92, 104)`.
    deep_puce, 0xA95C68;
    /// Creates a new `Colour`, setting its RGB value to `(133, 1, 1)`.
    deep_red, 0x850101;
    /// Creates a new `Colour`, setting its RGB value to `(132, 63, 91)`.
    deep_ruby, 0x843F5B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 51)`.
    deep_saffron, 0xFF9933;
    /// Creates a new `Colour`, setting its RGB value to `(0, 191, 255)`.
    deep_sky_blue, 0x00BFFF;
    /// Creates a new `Colour`, setting its RGB value to `(74, 100, 108)`.
    deep_space_sparkle, 0x4A646C;
    /// Creates a new `Colour`, setting its RGB value to `(85, 107, 47)`.
    deep_spring_bud, 0x556B2F;
    /// Creates a new `Colour`, setting its RGB value to `(126, 94, 96)`.
    deep_taupe, 0x7E5E60;
    /// Creates a new `Colour`, setting its RGB value to `(102, 66, 77)`.
    deep_tuscan_red, 0x66424D;
    /// Creates a new `Colour`, setting its RGB value to `(51, 0, 102)`.
    deep_violet, 0x330066;
    /// Creates a new `Colour`, setting its RGB value to `(186, 135, 89)`.
    deer, 0xBA8759;
    /// Creates a new `Colour`, setting its RGB value to `(21, 96, 189)`.
    denim, 0x1560BD;
    /// Creates a new `Colour`, setting its RGB value to `(34, 67, 182)`.
    denim_blue, 0x2243B6;
    /// Creates a new `Colour`, setting its RGB value to `(102, 153, 153)`.
    desaturated_cyan, 0x669999;
    /// Creates a new `Colour`, setting its RGB value to `(193, 154, 107)`.
    desert, 0xC19A6B;
    /// Creates a new `Colour`, setting its RGB value to `(237, 201, 175)`.
    desert_sand, 0xEDC9AF;
    /// Creates a new `Colour`, setting its RGB value to `(234, 60, 83)`.
    desire, 0xEA3C53;
    /// Creates a new `Colour`, setting its RGB value to `(185, 242, 255)`.
    diamond, 0xB9F2FF;
    /// Creates a new `Colour`, setting its RGB value to `(105, 105, 105)`.
    dim_gray, 0x696969;
    /// Creates a new `Colour`, setting its RGB value to `(197, 49, 81)`.
    dingy_dungeon, 0xC53151;
    /// Creates a new `Colour`, setting its RGB value to `(155, 118, 83)`.
    dirt, 0x9B7653;
    /// Creates a new `Colour`, setting its RGB value to `(30, 144, 255)`.
    dodger_blue, 0x1E90FF;
    /// Creates a new `Colour`, setting its RGB value to `(215, 24, 104)`.
    dogwood_rose, 0xD71868;
    /// Creates a new `Colour`, setting its RGB value to `(133, 187, 101)`.
    dollar_bill, 0x85BB65;
    /// Creates a new `Colour`, setting its RGB value to `(130, 142, 132)`.
    dolphin_gray, 0x828E84;
    /// Creates a new `Colour`, setting its RGB value to `(102, 76, 40)`.
    donkey_brown, 0x664C28;
    /// Creates a new `Colour`, setting its RGB value to `(150, 113, 23)`.
    drab, 0x967117;
    /// Creates a new `Colour`, setting its RGB value to `(0, 0, 156)`.
    duke_blue, 0x00009C;
    /// Creates a new `Colour`, setting its RGB value to `(229, 204, 201)`.
    dust_storm, 0xE5CCC9;
    /// Creates a new `Colour`, setting its RGB value to `(239, 223, 187)`.
    dutch_white, 0xEFDFBB;
    /// Creates a new `Colour`, setting its RGB value to `(225, 169, 95)`.
    earth_yellow, 0xE1A95F;
    /// Creates a new `Colour`, setting its RGB value to `(85, 93, 80)`.
    ebony, 0x555D50;
    /// Creates a new `Colour`, setting its RGB value to `(194, 178, 128)`.
    ecru, 0xC2B280;
    /// Creates a new `Colour`, setting its RGB value to `(27, 27, 27)`.
    eerie_black, 0x1B1B1B;
    /// Creates a new `Colour`, setting its RGB value to `(97, 64, 81)`.
    eggplant, 0x614051;
    /// Creates a new `Colour`, setting its RGB value to `(240, 234, 214)`.
    eggshell, 0xF0EAD6;
    /// Creates a new `Colour`, setting its RGB value to `(16, 52, 166)`.
    egyptian_blue, 0x1034A6;
    /// Creates a new `Colour`, setting its RGB value to `(125, 249, 255)`.
    electric_blue, 0x7DF9FF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 63)`.
    electric_crimson, 0xFF003F;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 255)`.
    electric_cyan, 0x00FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 0)`.
    electric_green, 0x00FF00;
    /// Creates a new `Colour`, setting its RGB value to `(111, 0, 255)`.
    electric_indigo, 0x6F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(244, 187, 255)`.
    electric_lavender, 0xF4BBFF;
    /// Creates a new `Colour`, setting its RGB value to `(204, 255, 0)`.
    electric_lime, 0xCCFF00;
    /// Creates a new `Colour`, setting its RGB value to `(191, 0, 255)`.
    electric_purple, 0xBF00FF;
    /// Creates a new `Colour`, setting its RGB value to `(63, 0, 255)`.
    electric_ultramarine, 0x3F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(143, 0, 255)`.
    electric_violet, 0x8F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 51)`.
    electric_yellow, 0xFFFF33;
    /// Creates a new `Colour`, setting its RGB value to `(80, 200, 120)`.
    emerald, 0x50C878;
    /// Creates a new `Colour`, setting its RGB value to `(108, 48, 130)`.
    eminence, 0x6C3082;
    /// Creates a new `Colour`, setting its RGB value to `(27, 77, 62)`.
    english_green, 0x1B4D3E;
    /// Creates a new `Colour`, setting its RGB value to `(180, 131, 149)`.
    english_lavender, 0xB48395;
    /// Creates a new `Colour`, setting its RGB value to `(171, 75, 82)`.
    english_red, 0xAB4B52;
    /// Creates a new `Colour`, setting its RGB value to `(204, 71, 75)`.
    english_vermillion, 0xCC474B;
    /// Creates a new `Colour`, setting its RGB value to `(86, 60, 92)`.
    english_violet, 0x563C5C;
    /// Creates a new `Colour`, setting its RGB value to `(150, 200, 162)`.
    eton_blue, 0x96C8A2;
    /// Creates a new `Colour`, setting its RGB value to `(68, 215, 168)`.
    eucalyptus, 0x44D7A8;
    /// Creates a new `Colour`, setting its RGB value to `(193, 154, 107)`.
    fallow, 0xC19A6B;
    /// Creates a new `Colour`, setting its RGB value to `(128, 24, 24)`.
    falu_red, 0x801818;
    /// Creates a new `Colour`, setting its RGB value to `(181, 51, 137)`.
    fandango, 0xB53389;
    /// Creates a new `Colour`, setting its RGB value to `(222, 82, 133)`.
    fandango_pink, 0xDE5285;
    /// Creates a new `Colour`, setting its RGB value to `(244, 0, 161)`.
    fashion_fuchsia, 0xF400A1;
    /// Creates a new `Colour`, setting its RGB value to `(229, 170, 112)`.
    fawn, 0xE5AA70;
    /// Creates a new `Colour`, setting its RGB value to `(77, 93, 83)`.
    feldgrau, 0x4D5D53;
    /// Creates a new `Colour`, setting its RGB value to `(253, 213, 177)`.
    feldspar, 0xFDD5B1;
    /// Creates a new `Colour`, setting its RGB value to `(79, 121, 66)`.
    fern_green, 0x4F7942;
    /// Creates a new `Colour`, setting its RGB value to `(255, 40, 0)`.
    ferrari_red, 0xFF2800;
    /// Creates a new `Colour`, setting its RGB value to `(108, 84, 30)`.
    field_drab, 0x6C541E;
    /// Creates a new `Colour`, setting its RGB value to `(255, 84, 112)`.
    fiery_rose, 0xFF5470;
    /// Creates a new `Colour`, setting its RGB value to `(206, 32, 41)`.
    fire_engine_red, 0xCE2029;
    /// Creates a new `Colour`, setting its RGB value to `(178, 34, 34)`.
    firebrick, 0xB22222;
    /// Creates a new `Colour`, setting its RGB value to `(226, 88, 34)`.
    flame, 0xE25822;
    /// Creates a new `Colour`, setting its RGB value to `(252, 142, 172)`.
    flamingo_pink, 0xFC8EAC;
    /// Creates a new `Colour`, setting its RGB value to `(107, 68, 35)`.
    flattery, 0x6B4423;
    /// Creates a new `Colour`, setting its RGB value to `(247, 233, 142)`.
    flavescent, 0xF7E98E;
    /// Creates a new `Colour`, setting its RGB value to `(238, 220, 130)`.
    flax, 0xEEDC82;
    /// Creates a new `Colour`, setting its RGB value to `(162, 0, 109)`.
    flirt, 0xA2006D;
    /// Creates a new `Colour`, setting its RGB value to `(255, 250, 240)`.
    floral_white, 0xFFFAF0;
    /// Creates a new `Colour`, setting its RGB value to `(255, 191, 0)`.
    fluorescent_orange, 0xFFBF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 20, 147)`.
    fluorescent_pink, 0xFF1493;
    /// Creates a new `Colour`, setting its RGB value to `(204, 255, 0)`.
    fluorescent_yellow, 0xCCFF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 79)`.
    folly, 0xFF004F;
    /// Creates a new `Colour`, setting its RGB value to `(1, 68, 33)`.
    forest_green_traditional, 0x014421;
    /// Creates a new `Colour`, setting its RGB value to `(34, 139, 34)`.
    forest_green_web, 0x228B22;
    /// Creates a new `Colour`, setting its RGB value to `(166, 123, 91)`.
    french_beige, 0xA67B5B;
    /// Creates a new `Colour`, setting its RGB value to `(133, 109, 77)`.
    french_bistre, 0x856D4D;
    /// Creates a new `Colour`, setting its RGB value to `(0, 114, 187)`.
    french_blue, 0x0072BB;
    /// Creates a new `Colour`, setting its RGB value to `(253, 63, 146)`.
    french_fuchsia, 0xFD3F92;
    /// Creates a new `Colour`, setting its RGB value to `(134, 96, 142)`.
    french_lilac, 0x86608E;
    /// Creates a new `Colour`, setting its RGB value to `(158, 253, 56)`.
    french_lime, 0x9EFD38;
    /// Creates a new `Colour`, setting its RGB value to `(212, 115, 212)`.
    french_mauve, 0xD473D4;
    /// Creates a new `Colour`, setting its RGB value to `(253, 108, 158)`.
    french_pink, 0xFD6C9E;
    /// Creates a new `Colour`, setting its RGB value to `(129, 20, 83)`.
    french_plum, 0x811453;
    /// Creates a new `Colour`, setting its RGB value to `(78, 22, 9)`.
    french_puce, 0x4E1609;
    /// Creates a new `Colour`, setting its RGB value to `(199, 44, 72)`.
    french_raspberry, 0xC72C48;
    /// Creates a new `Colour`, setting its RGB value to `(246, 74, 138)`.
    french_rose, 0xF64A8A;
    /// Creates a new `Colour`, setting its RGB value to `(119, 181, 254)`.
    french_sky_blue, 0x77B5FE;
    /// Creates a new `Colour`, setting its RGB value to `(136, 6, 206)`.
    french_violet, 0x8806CE;
    /// Creates a new `Colour`, setting its RGB value to `(172, 30, 68)`.
    french_wine, 0xAC1E44;
    /// Creates a new `Colour`, setting its RGB value to `(166, 231, 255)`.
    fresh_air, 0xA6E7FF;
    /// Creates a new `Colour`, setting its RGB value to `(233, 54, 167)`.
    frostbite, 0xE936A7;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 255)`.
    fuchsia, 0xFF00FF;
    /// Creates a new `Colour`, setting its RGB value to `(193, 84, 193)`.
    fuchsia_crayola, 0xC154C1;
    /// Creates a new `Colour`, setting its RGB value to `(255, 119, 255)`.
    fuchsia_pink, 0xFF77FF;
    /// Creates a new `Colour`, setting its RGB value to `(204, 57, 123)`.
    fuchsia_purple, 0xCC397B;
    /// Creates a new `Colour`, setting its RGB value to `(199, 67, 117)`.
    fuchsia_rose, 0xC74375;
    /// Creates a new `Colour`, setting its RGB value to `(228, 132, 0)`.
    fulvous, 0xE48400;
    /// Creates a new `Colour`, setting its RGB value to `(204, 102, 102)`.
    fuzzy_wuzzy, 0xCC6666;
    /// Creates a new `Colour`, setting its RGB value to `(220, 220, 220)`.
    gainsboro, 0xDCDCDC;
    /// Creates a new `Colour`, setting its RGB value to `(228, 155, 15)`.
    gamboge, 0xE49B0F;
    /// Creates a new `Colour`, setting its RGB value to `(153, 102, 0)`.
    gamboge_orange_brown, 0x996600;
    /// Creates a new `Colour`, setting its RGB value to `(255, 223, 70)`.
    gargoyle_gas, 0xFFDF46;
    /// Creates a new `Colour`, setting its RGB value to `(0, 127, 102)`.
    generic_viridian, 0x007F66;
    /// Creates a new `Colour`, setting its RGB value to `(248, 248, 255)`.
    ghost_white, 0xF8F8FF;
    /// Creates a new `Colour`, setting its RGB value to `(176, 92, 82)`.
    giants_club, 0xB05C52;
    /// Creates a new `Colour`, setting its RGB value to `(254, 90, 29)`.
    giants_orange, 0xFE5A1D;
    /// Creates a new `Colour`, setting its RGB value to `(176, 101, 0)`.
    ginger, 0xB06500;
    /// Creates a new `Colour`, setting its RGB value to `(96, 130, 182)`.
    glaucous, 0x6082B6;
    /// Creates a new `Colour`, setting its RGB value to `(230, 232, 250)`.
    glitter, 0xE6E8FA;
    /// Creates a new `Colour`, setting its RGB value to `(171, 146, 179)`.
    glossy_grape, 0xAB92B3;
    /// Creates a new `Colour`, setting its RGB value to `(0, 171, 102)`.
    go_green, 0x00AB66;
    /// Creates a new `Colour`, setting its RGB value to `(241, 196, 15)`.
    gold, 0xF1C40F;
    /// Creates a new `Colour`, setting its RGB value to `(133, 117, 78)`.
    gold_fusion, 0x85754E;
    /// Creates a new `Colour`, setting its RGB value to `(212, 175, 55)`.
    gold_metallic, 0xD4AF37;
    /// Creates a new `Colour`, setting its RGB value to `(255, 215, 0)`.
    gold_web_golden, 0xFFD700;
    /// Creates a new `Colour`, setting its RGB value to `(153, 101, 21)`.
    golden_brown, 0x996515;
    /// Creates a new `Colour`, setting its RGB value to `(252, 194, 0)`.
    golden_poppy, 0xFCC200;
    /// Creates a new `Colour`, setting its RGB value to `(255, 223, 0)`.
    golden_yellow, 0xFFDF00;
    /// Creates a new `Colour`, setting its RGB value to `(218, 165, 32)`.
    goldenrod, 0xDAA520;
    /// Creates a new `Colour`, setting its RGB value to `(103, 103, 103)`.
    granite_gray, 0x676767;
    /// Creates a new `Colour`, setting its RGB value to `(168, 228, 160)`.
    granny_smith_apple, 0xA8E4A0;
    /// Creates a new `Colour`, setting its RGB value to `(111, 45, 168)`.
    grape, 0x6F2DA8;
    /// Creates a new `Colour`, setting its RGB value to `(128, 128, 128)`.
    gray, 0x808080;
    /// Creates a new `Colour`, setting its RGB value to `(70, 89, 69)`.
    gray_asparagus, 0x465945;
    /// Creates a new `Colour`, setting its RGB value to `(140, 146, 172)`.
    gray_blue, 0x8C92AC;
    /// Creates a new `Colour`, setting its RGB value to `(128, 128, 128)`.
    gray_html_css_gray, 0x808080;
    /// Creates a new `Colour`, setting its RGB value to `(190, 190, 190)`.
    gray_x11_gray, 0xBEBEBE;
    /// Creates a new `Colour`, setting its RGB value to `(17, 100, 180)`.
    green_blue, 0x1164B4;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 0)`.
    green_color_wheel_x11_green, 0x00FF00;
    /// Creates a new `Colour`, setting its RGB value to `(28, 172, 120)`.
    green_crayola, 0x1CAC78;
    /// Creates a new `Colour`, setting its RGB value to `(0, 153, 102)`.
    green_cyan, 0x009966;
    /// Creates a new `Colour`, setting its RGB value to `(0, 128, 0)`.
    green_html_css_color, 0x008000;
    /// Creates a new `Colour`, setting its RGB value to `(167, 244, 50)`.
    green_lizard, 0xA7F432;
    /// Creates a new `Colour`, setting its RGB value to `(0, 168, 119)`.
    green_munsell, 0x00A877;
    /// Creates a new `Colour`, setting its RGB value to `(0, 159, 107)`.
    green_ncs, 0x009F6B;
    /// Creates a new `Colour`, setting its RGB value to `(0, 173, 67)`.
    green_pantone, 0x00AD43;
    /// Creates a new `Colour`, setting its RGB value to `(0, 165, 80)`.
    green_pigment, 0x00A550;
    /// Creates a new `Colour`, setting its RGB value to `(102, 176, 50)`.
    green_ryb, 0x66B032;
    /// Creates a new `Colour`, setting its RGB value to `(110, 174, 161)`.
    green_sheen, 0x6EAEA1;
    /// Creates a new `Colour`, setting its RGB value to `(173, 255, 47)`.
    green_yellow, 0xADFF2F;
    /// Creates a new `Colour`, setting its RGB value to `(136, 88, 24)`.
    grizzly, 0x885818;
    /// Creates a new `Colour`, setting its RGB value to `(169, 154, 134)`.
    grullo, 0xA99A86;
    /// Creates a new `Colour`, setting its RGB value to `(42, 52, 57)`.
    gunmetal, 0x2A3439;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 127)`.
    guppie_green, 0x00FF7F;
    /// Creates a new `Colour`, setting its RGB value to `(102, 56, 84)`.
    halay_be, 0x663854;
    /// Creates a new `Colour`, setting its RGB value to `(68, 108, 207)`.
    han_blue, 0x446CCF;
    /// Creates a new `Colour`, setting its RGB value to `(82, 24, 250)`.
    han_purple, 0x5218FA;
    /// Creates a new `Colour`, setting its RGB value to `(233, 214, 107)`.
    hansa_yellow, 0xE9D66B;
    /// Creates a new `Colour`, setting its RGB value to `(63, 255, 0)`.
    harlequin, 0x3FFF00;
    /// Creates a new `Colour`, setting its RGB value to `(70, 203, 24)`.
    harlequin_green, 0x46CB18;
    /// Creates a new `Colour`, setting its RGB value to `(201, 0, 22)`.
    harvard_crimson, 0xC90016;
    /// Creates a new `Colour`, setting its RGB value to `(218, 145, 0)`.
    harvest_gold, 0xDA9100;
    /// Creates a new `Colour`, setting its RGB value to `(128, 128, 0)`.
    heart_gold, 0x808000;
    /// Creates a new `Colour`, setting its RGB value to `(255, 122, 0)`.
    heat_wave, 0xFF7A00;
    /// Creates a new `Colour`, setting its RGB value to `(150, 0, 24)`.
    heidelberg_red, 0x960018;
    /// Creates a new `Colour`, setting its RGB value to `(223, 115, 255)`.
    heliotrope, 0xDF73FF;
    /// Creates a new `Colour`, setting its RGB value to `(170, 152, 169)`.
    heliotrope_gray, 0xAA98A9;
    /// Creates a new `Colour`, setting its RGB value to `(170, 0, 187)`.
    heliotrope_magenta, 0xAA00BB;
    /// Creates a new `Colour`, setting its RGB value to `(244, 0, 161)`.
    hollywood_cerise, 0xF400A1;
    /// Creates a new `Colour`, setting its RGB value to `(240, 255, 240)`.
    honeydew, 0xF0FFF0;
    /// Creates a new `Colour`, setting its RGB value to `(0, 109, 176)`.
    honolulu_blue, 0x006DB0;
    /// Creates a new `Colour`, setting its RGB value to `(73, 121, 107)`.
    hookers_green, 0x49796B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 29, 206)`.
    hot_magenta, 0xFF1DCE;
    /// Creates a new `Colour`, setting its RGB value to `(255, 105, 180)`.
    hot_pink, 0xFF69B4;
    /// Creates a new `Colour`, setting its RGB value to `(53, 94, 59)`.
    hunter_green, 0x355E3B;
    /// Creates a new `Colour`, setting its RGB value to `(113, 166, 210)`.
    iceberg, 0x71A6D2;
    /// Creates a new `Colour`, setting its RGB value to `(252, 247, 94)`.
    icterine, 0xFCF75E;
    /// Creates a new `Colour`, setting its RGB value to `(113, 188, 120)`.
    iguana_green, 0x71BC78;
    /// Creates a new `Colour`, setting its RGB value to `(49, 145, 119)`.
    illuminating_emerald, 0x319177;
    /// Creates a new `Colour`, setting its RGB value to `(96, 47, 107)`.
    imperial, 0x602F6B;
    /// Creates a new `Colour`, setting its RGB value to `(0, 35, 149)`.
    imperial_blue, 0x002395;
    /// Creates a new `Colour`, setting its RGB value to `(102, 2, 60)`.
    imperial_purple, 0x66023C;
    /// Creates a new `Colour`, setting its RGB value to `(237, 41, 57)`.
    imperial_red, 0xED2939;
    /// Creates a new `Colour`, setting its RGB value to `(178, 236, 93)`.
    inchworm, 0xB2EC5D;
    /// Creates a new `Colour`, setting its RGB value to `(76, 81, 109)`.
    independence, 0x4C516D;
    /// Creates a new `Colour`, setting its RGB value to `(19, 136, 8)`.
    india_green, 0x138808;
    /// Creates a new `Colour`, setting its RGB value to `(205, 92, 92)`.
    indian_red, 0xCD5C5C;
    /// Creates a new `Colour`, setting its RGB value to `(227, 168, 87)`.
    indian_yellow, 0xE3A857;
    /// Creates a new `Colour`, setting its RGB value to `(75, 0, 130)`.
    indigo, 0x4B0082;
    /// Creates a new `Colour`, setting its RGB value to `(9, 31, 146)`.
    indigo_dye, 0x091F92;
    /// Creates a new `Colour`, setting its RGB value to `(75, 0, 130)`.
    indigo_web, 0x4B0082;
    /// Creates a new `Colour`, setting its RGB value to `(255, 73, 108)`.
    infra_red, 0xFF496C;
    /// Creates a new `Colour`, setting its RGB value to `(54, 12, 204)`.
    interdimensional_blue, 0x360CCC;
    /// Creates a new `Colour`, setting its RGB value to `(0, 47, 167)`.
    international_klein_blue, 0x002FA7;
    /// Creates a new `Colour`, setting its RGB value to `(255, 79, 0)`.
    international_orange_aerospace, 0xFF4F00;
    /// Creates a new `Colour`, setting its RGB value to `(186, 22, 12)`.
    international_orange_engineering, 0xBA160C;
    /// Creates a new `Colour`, setting its RGB value to `(192, 54, 44)`.
    international_orange_golden_gate_bridge, 0xC0362C;
    /// Creates a new `Colour`, setting its RGB value to `(90, 79, 207)`.
    iris, 0x5A4FCF;
    /// Creates a new `Colour`, setting its RGB value to `(179, 68, 108)`.
    irresistible, 0xB3446C;
    /// Creates a new `Colour`, setting its RGB value to `(244, 240, 236)`.
    isabelline, 0xF4F0EC;
    /// Creates a new `Colour`, setting its RGB value to `(0, 144, 0)`.
    islamic_green, 0x009000;
    /// Creates a new `Colour`, setting its RGB value to `(178, 255, 255)`.
    italian_sky_blue, 0xB2FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 240)`.
    ivory, 0xFFFFF0;
    /// Creates a new `Colour`, setting its RGB value to `(0, 168, 107)`.
    jade, 0x00A86B;
    /// Creates a new `Colour`, setting its RGB value to `(157, 41, 51)`.
    japanese_carmine, 0x9D2933;
    /// Creates a new `Colour`, setting its RGB value to `(38, 67, 72)`.
    japanese_indigo, 0x264348;
    /// Creates a new `Colour`, setting its RGB value to `(91, 50, 86)`.
    japanese_violet, 0x5B3256;
    /// Creates a new `Colour`, setting its RGB value to `(248, 222, 126)`.
    jasmine, 0xF8DE7E;
    /// Creates a new `Colour`, setting its RGB value to `(215, 59, 62)`.
    jasper, 0xD73B3E;
    /// Creates a new `Colour`, setting its RGB value to `(165, 11, 94)`.
    jazzberry_jam, 0xA50B5E;
    /// Creates a new `Colour`, setting its RGB value to `(218, 97, 78)`.
    jelly_bean, 0xDA614E;
    /// Creates a new `Colour`, setting its RGB value to `(52, 52, 52)`.
    jet, 0x343434;
    /// Creates a new `Colour`, setting its RGB value to `(244, 202, 22)`.
    jonquil, 0xF4CA16;
    /// Creates a new `Colour`, setting its RGB value to `(138, 185, 241)`.
    jordy_blue, 0x8AB9F1;
    /// Creates a new `Colour`, setting its RGB value to `(189, 218, 87)`.
    june_bud, 0xBDDA57;
    /// Creates a new `Colour`, setting its RGB value to `(41, 171, 135)`.
    jungle_green, 0x29AB87;
    /// Creates a new `Colour`, setting its RGB value to `(76, 187, 23)`.
    kelly_green, 0x4CBB17;
    /// Creates a new `Colour`, setting its RGB value to `(124, 28, 5)`.
    kenyan_copper, 0x7C1C05;
    /// Creates a new `Colour`, setting its RGB value to `(58, 176, 158)`.
    keppel, 0x3AB09E;
    /// Creates a new `Colour`, setting its RGB value to `(186, 218, 85)`.
    kerbal, 0xBADA55;
    /// Creates a new `Colour`, setting its RGB value to `(232, 244, 140)`.
    key_lime, 0xE8F48C;
    /// Creates a new `Colour`, setting its RGB value to `(195, 176, 145)`.
    khaki_html_css_khaki, 0xC3B091;
    /// Creates a new `Colour`, setting its RGB value to `(240, 230, 140)`.
    khaki_x11_light_khaki, 0xF0E68C;
    /// Creates a new `Colour`, setting its RGB value to `(142, 229, 63)`.
    kiwi, 0x8EE53F;
    /// Creates a new `Colour`, setting its RGB value to `(136, 45, 23)`.
    kobe, 0x882D17;
    /// Creates a new `Colour`, setting its RGB value to `(231, 159, 196)`.
    kobi, 0xE79FC4;
    /// Creates a new `Colour`, setting its RGB value to `(107, 68, 35)`.
    kobicha, 0x6B4423;
    /// Creates a new `Colour`, setting its RGB value to `(53, 66, 48)`.
    kombu_green, 0x354230;
    /// Creates a new `Colour`, setting its RGB value to `(81, 40, 136)`.
    ksu_purple, 0x512888;
    /// Creates a new `Colour`, setting its RGB value to `(232, 0, 13)`.
    ku_crimson, 0xE8000D;
    /// Creates a new `Colour`, setting its RGB value to `(8, 120, 48)`.
    la_salle_green, 0x087830;
    /// Creates a new `Colour`, setting its RGB value to `(214, 202, 221)`.
    languid_lavender, 0xD6CADD;
    /// Creates a new `Colour`, setting its RGB value to `(38, 97, 156)`.
    lapis_lazuli, 0x26619C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 102)`.
    laser_lemon, 0xFFFF66;
    /// Creates a new `Colour`, setting its RGB value to `(169, 186, 157)`.
    laurel_green, 0xA9BA9D;
    /// Creates a new `Colour`, setting its RGB value to `(207, 16, 32)`.
    lava, 0xCF1020;
    /// Creates a new `Colour`, setting its RGB value to `(204, 204, 255)`.
    lavender_blue, 0xCCCCFF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 240, 245)`.
    lavender_blush, 0xFFF0F5;
    /// Creates a new `Colour`, setting its RGB value to `(181, 126, 220)`.
    lavender_floral, 0xB57EDC;
    /// Creates a new `Colour`, setting its RGB value to `(196, 195, 208)`.
    lavender_gray, 0xC4C3D0;
    /// Creates a new `Colour`, setting its RGB value to `(148, 87, 235)`.
    lavender_indigo, 0x9457EB;
    /// Creates a new `Colour`, setting its RGB value to `(238, 130, 238)`.
    lavender_magenta, 0xEE82EE;
    /// Creates a new `Colour`, setting its RGB value to `(230, 230, 250)`.
    lavender_mist, 0xE6E6FA;
    /// Creates a new `Colour`, setting its RGB value to `(251, 174, 210)`.
    lavender_pink, 0xFBAED2;
    /// Creates a new `Colour`, setting its RGB value to `(150, 123, 182)`.
    lavender_purple, 0x967BB6;
    /// Creates a new `Colour`, setting its RGB value to `(251, 160, 227)`.
    lavender_rose, 0xFBA0E3;
    /// Creates a new `Colour`, setting its RGB value to `(230, 230, 250)`.
    lavender_web, 0xE6E6FA;
    /// Creates a new `Colour`, setting its RGB value to `(124, 252, 0)`.
    lawn_green, 0x7CFC00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 247, 0)`.
    lemon, 0xFFF700;
    /// Creates a new `Colour`, setting its RGB value to `(255, 250, 205)`.
    lemon_chiffon, 0xFFFACD;
    /// Creates a new `Colour`, setting its RGB value to `(204, 160, 29)`.
    lemon_curry, 0xCCA01D;
    /// Creates a new `Colour`, setting its RGB value to `(253, 255, 0)`.
    lemon_glacier, 0xFDFF00;
    /// Creates a new `Colour`, setting its RGB value to `(227, 255, 0)`.
    lemon_lime, 0xE3FF00;
    /// Creates a new `Colour`, setting its RGB value to `(246, 234, 190)`.
    lemon_meringue, 0xF6EABE;
    /// Creates a new `Colour`, setting its RGB value to `(255, 244, 79)`.
    lemon_yellow, 0xFFF44F;
    /// Creates a new `Colour`, setting its RGB value to `(84, 90, 167)`.
    liberty, 0x545AA7;
    /// Creates a new `Colour`, setting its RGB value to `(26, 17, 16)`.
    licorice, 0x1A1110;
    /// Creates a new `Colour`, setting its RGB value to `(253, 213, 177)`.
    light_apricot, 0xFDD5B1;
    /// Creates a new `Colour`, setting its RGB value to `(173, 216, 230)`.
    light_blue, 0xADD8E6;
    /// Creates a new `Colour`, setting its RGB value to `(181, 101, 29)`.
    light_brown, 0xB5651D;
    /// Creates a new `Colour`, setting its RGB value to `(230, 103, 113)`.
    light_carmine_pink, 0xE66771;
    /// Creates a new `Colour`, setting its RGB value to `(136, 172, 224)`.
    light_cobalt_blue, 0x88ACE0;
    /// Creates a new `Colour`, setting its RGB value to `(240, 128, 128)`.
    light_coral, 0xF08080;
    /// Creates a new `Colour`, setting its RGB value to `(147, 204, 234)`.
    light_cornflower_blue, 0x93CCEA;
    /// Creates a new `Colour`, setting its RGB value to `(245, 105, 145)`.
    light_crimson, 0xF56991;
    /// Creates a new `Colour`, setting its RGB value to `(224, 255, 255)`.
    light_cyan, 0xE0FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 92, 205)`.
    light_deep_pink, 0xFF5CCD;
    /// Creates a new `Colour`, setting its RGB value to `(200, 173, 127)`.
    light_french_beige, 0xC8AD7F;
    /// Creates a new `Colour`, setting its RGB value to `(249, 132, 239)`.
    light_fuchsia_pink, 0xF984EF;
    /// Creates a new `Colour`, setting its RGB value to `(250, 250, 210)`.
    light_goldenrod_yellow, 0xFAFAD2;
    /// Creates a new `Colour`, setting its RGB value to `(211, 211, 211)`.
    light_gray, 0xD3D3D3;
    /// Creates a new `Colour`, setting its RGB value to `(204, 153, 204)`.
    light_grayish_magenta, 0xCC99CC;
    /// Creates a new `Colour`, setting its RGB value to `(144, 238, 144)`.
    light_green, 0x90EE90;
    /// Creates a new `Colour`, setting its RGB value to `(151, 156, 159)`.
    light_grey, 0x979C9F;
    /// Creates a new `Colour`, setting its RGB value to `(255, 179, 222)`.
    light_hot_pink, 0xFFB3DE;
    /// Creates a new `Colour`, setting its RGB value to `(240, 230, 140)`.
    light_khaki, 0xF0E68C;
    /// Creates a new `Colour`, setting its RGB value to `(211, 155, 203)`.
    light_medium_orchid, 0xD39BCB;
    /// Creates a new `Colour`, setting its RGB value to `(173, 223, 173)`.
    light_moss_green, 0xADDFAD;
    /// Creates a new `Colour`, setting its RGB value to `(230, 168, 215)`.
    light_orchid, 0xE6A8D7;
    /// Creates a new `Colour`, setting its RGB value to `(177, 156, 217)`.
    light_pastel_purple, 0xB19CD9;
    /// Creates a new `Colour`, setting its RGB value to `(255, 182, 193)`.
    light_pink, 0xFFB6C1;
    /// Creates a new `Colour`, setting its RGB value to `(233, 116, 81)`.
    light_red_ochre, 0xE97451;
    /// Creates a new `Colour`, setting its RGB value to `(255, 160, 122)`.
    light_salmon, 0xFFA07A;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 153)`.
    light_salmon_pink, 0xFF9999;
    /// Creates a new `Colour`, setting its RGB value to `(32, 178, 170)`.
    light_sea_green, 0x20B2AA;
    /// Creates a new `Colour`, setting its RGB value to `(135, 206, 250)`.
    light_sky_blue, 0x87CEFA;
    /// Creates a new `Colour`, setting its RGB value to `(119, 136, 153)`.
    light_slate_gray, 0x778899;
    /// Creates a new `Colour`, setting its RGB value to `(176, 196, 222)`.
    light_steel_blue, 0xB0C4DE;
    /// Creates a new `Colour`, setting its RGB value to `(179, 139, 109)`.
    light_taupe, 0xB38B6D;
    /// Creates a new `Colour`, setting its RGB value to `(230, 143, 172)`.
    light_thulian_pink, 0xE68FAC;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 224)`.
    light_yellow, 0xFFFFE0;
    /// Creates a new `Colour`, setting its RGB value to `(149, 165, 166)`.
    lighter_grey, 0x95A5A6;
    /// Creates a new `Colour`, setting its RGB value to `(200, 162, 200)`.
    lilac, 0xC8A2C8;
    /// Creates a new `Colour`, setting its RGB value to `(174, 152, 170)`.
    lilac_luster, 0xAE98AA;
    /// Creates a new `Colour`, setting its RGB value to `(191, 255, 0)`.
    lime_color_wheel, 0xBFFF00;
    /// Creates a new `Colour`, setting its RGB value to `(50, 205, 50)`.
    lime_green, 0x32CD32;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 0)`.
    lime_web_x11_green, 0x00FF00;
    /// Creates a new `Colour`, setting its RGB value to `(157, 194, 9)`.
    limerick, 0x9DC209;
    /// Creates a new `Colour`, setting its RGB value to `(25, 89, 5)`.
    lincoln_green, 0x195905;
    /// Creates a new `Colour`, setting its RGB value to `(250, 240, 230)`.
    linen, 0xFAF0E6;
    /// Creates a new `Colour`, setting its RGB value to `(222, 111, 161)`.
    liseran_purple, 0xDE6FA1;
    /// Creates a new `Colour`, setting its RGB value to `(108, 160, 220)`.
    little_boy_blue, 0x6CA0DC;
    /// Creates a new `Colour`, setting its RGB value to `(103, 76, 71)`.
    liver, 0x674C47;
    /// Creates a new `Colour`, setting its RGB value to `(152, 116, 86)`.
    liver_chestnut, 0x987456;
    /// Creates a new `Colour`, setting its RGB value to `(184, 109, 41)`.
    liver_dogs, 0xB86D29;
    /// Creates a new `Colour`, setting its RGB value to `(108, 46, 31)`.
    liver_organ, 0x6C2E1F;
    /// Creates a new `Colour`, setting its RGB value to `(102, 153, 204)`.
    livid, 0x6699CC;
    /// Creates a new `Colour`, setting its RGB value to `(21, 242, 253)`.
    loeenlopenlook_vomit_indogo_lopen_gabriel, 0x15F2FD;
    /// Creates a new `Colour`, setting its RGB value to `(255, 228, 205)`.
    lumber, 0xFFE4CD;
    /// Creates a new `Colour`, setting its RGB value to `(230, 32, 32)`.
    lust, 0xE62020;
    /// Creates a new `Colour`, setting its RGB value to `(0, 28, 61)`.
    maastricht_blue, 0x001C3D;
    /// Creates a new `Colour`, setting its RGB value to `(255, 189, 136)`.
    macaroni_and_cheese, 0xFFBD88;
    /// Creates a new `Colour`, setting its RGB value to `(204, 51, 54)`.
    madder_lake, 0xCC3336;
    /// Creates a new `Colour`, setting its RGB value to `(233, 30, 99)`.
    magenta, 0xE91E63;
    /// Creates a new `Colour`, setting its RGB value to `(255, 85, 163)`.
    magenta_crayola, 0xFF55A3;
    /// Creates a new `Colour`, setting its RGB value to `(202, 31, 123)`.
    magenta_dye, 0xCA1F7B;
    /// Creates a new `Colour`, setting its RGB value to `(159, 69, 118)`.
    magenta_haze, 0x9F4576;
    /// Creates a new `Colour`, setting its RGB value to `(208, 65, 126)`.
    magenta_pantone, 0xD0417E;
    /// Creates a new `Colour`, setting its RGB value to `(204, 51, 139)`.
    magenta_pink, 0xCC338B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 144)`.
    magenta_process, 0xFF0090;
    /// Creates a new `Colour`, setting its RGB value to `(170, 240, 209)`.
    magic_mint, 0xAAF0D1;
    /// Creates a new `Colour`, setting its RGB value to `(255, 68, 102)`.
    magic_potion, 0xFF4466;
    /// Creates a new `Colour`, setting its RGB value to `(248, 244, 255)`.
    magnolia, 0xF8F4FF;
    /// Creates a new `Colour`, setting its RGB value to `(192, 64, 0)`.
    mahogany, 0xC04000;
    /// Creates a new `Colour`, setting its RGB value to `(251, 236, 93)`.
    maize, 0xFBEC5D;
    /// Creates a new `Colour`, setting its RGB value to `(96, 80, 220)`.
    majorelle_blue, 0x6050DC;
    /// Creates a new `Colour`, setting its RGB value to `(11, 218, 81)`.
    malachite, 0x0BDA51;
    /// Creates a new `Colour`, setting its RGB value to `(151, 154, 170)`.
    manatee, 0x979AAA;
    /// Creates a new `Colour`, setting its RGB value to `(243, 122, 72)`.
    mandarin, 0xF37A48;
    /// Creates a new `Colour`, setting its RGB value to `(255, 130, 67)`.
    mango_tango, 0xFF8243;
    /// Creates a new `Colour`, setting its RGB value to `(116, 195, 101)`.
    mantis, 0x74C365;
    /// Creates a new `Colour`, setting its RGB value to `(136, 0, 133)`.
    mardi_gras, 0x880085;
    /// Creates a new `Colour`, setting its RGB value to `(234, 162, 33)`.
    marigold, 0xEAA221;
    /// Creates a new `Colour`, setting its RGB value to `(195, 33, 72)`.
    maroon_crayola, 0xC32148;
    /// Creates a new `Colour`, setting its RGB value to `(128, 0, 0)`.
    maroon_html_css, 0x800000;
    /// Creates a new `Colour`, setting its RGB value to `(176, 48, 96)`.
    maroon_x11, 0xB03060;
    /// Creates a new `Colour`, setting its RGB value to `(224, 176, 255)`.
    mauve, 0xE0B0FF;
    /// Creates a new `Colour`, setting its RGB value to `(145, 95, 109)`.
    mauve_taupe, 0x915F6D;
    /// Creates a new `Colour`, setting its RGB value to `(239, 152, 170)`.
    mauvelous, 0xEF98AA;
    /// Creates a new `Colour`, setting its RGB value to `(71, 171, 204)`.
    maximum_blue, 0x47ABCC;
    /// Creates a new `Colour`, setting its RGB value to `(48, 191, 191)`.
    maximum_blue_green, 0x30BFBF;
    /// Creates a new `Colour`, setting its RGB value to `(172, 172, 230)`.
    maximum_blue_purple, 0xACACE6;
    /// Creates a new `Colour`, setting its RGB value to `(94, 140, 49)`.
    maximum_green, 0x5E8C31;
    /// Creates a new `Colour`, setting its RGB value to `(217, 230, 80)`.
    maximum_green_yellow, 0xD9E650;
    /// Creates a new `Colour`, setting its RGB value to `(115, 51, 128)`.
    maximum_purple, 0x733380;
    /// Creates a new `Colour`, setting its RGB value to `(217, 33, 33)`.
    maximum_red, 0xD92121;
    /// Creates a new `Colour`, setting its RGB value to `(166, 58, 121)`.
    maximum_red_purple, 0xA63A79;
    /// Creates a new `Colour`, setting its RGB value to `(250, 250, 55)`.
    maximum_yellow, 0xFAFA37;
    /// Creates a new `Colour`, setting its RGB value to `(242, 186, 73)`.
    maximum_yellow_red, 0xF2BA49;
    /// Creates a new `Colour`, setting its RGB value to `(76, 145, 65)`.
    may_green, 0x4C9141;
    /// Creates a new `Colour`, setting its RGB value to `(115, 194, 251)`.
    maya_blue, 0x73C2FB;
    /// Creates a new `Colour`, setting its RGB value to `(229, 183, 59)`.
    meat_brown, 0xE5B73B;
    /// Creates a new `Colour`, setting its RGB value to `(102, 221, 170)`.
    medium_aquamarine, 0x66DDAA;
    /// Creates a new `Colour`, setting its RGB value to `(0, 0, 205)`.
    medium_blue, 0x0000CD;
    /// Creates a new `Colour`, setting its RGB value to `(226, 6, 44)`.
    medium_candy_apple_red, 0xE2062C;
    /// Creates a new `Colour`, setting its RGB value to `(175, 64, 53)`.
    medium_carmine, 0xAF4035;
    /// Creates a new `Colour`, setting its RGB value to `(243, 229, 171)`.
    medium_champagne, 0xF3E5AB;
    /// Creates a new `Colour`, setting its RGB value to `(3, 80, 150)`.
    medium_electric_blue, 0x035096;
    /// Creates a new `Colour`, setting its RGB value to `(28, 53, 45)`.
    medium_jungle_green, 0x1C352D;
    /// Creates a new `Colour`, setting its RGB value to `(221, 160, 221)`.
    medium_lavender_magenta, 0xDDA0DD;
    /// Creates a new `Colour`, setting its RGB value to `(186, 85, 211)`.
    medium_orchid, 0xBA55D3;
    /// Creates a new `Colour`, setting its RGB value to `(0, 103, 165)`.
    medium_persian_blue, 0x0067A5;
    /// Creates a new `Colour`, setting its RGB value to `(147, 112, 219)`.
    medium_purple, 0x9370DB;
    /// Creates a new `Colour`, setting its RGB value to `(187, 51, 133)`.
    medium_red_violet, 0xBB3385;
    /// Creates a new `Colour`, setting its RGB value to `(170, 64, 105)`.
    medium_ruby, 0xAA4069;
    /// Creates a new `Colour`, setting its RGB value to `(60, 179, 113)`.
    medium_sea_green, 0x3CB371;
    /// Creates a new `Colour`, setting its RGB value to `(128, 218, 235)`.
    medium_sky_blue, 0x80DAEB;
    /// Creates a new `Colour`, setting its RGB value to `(123, 104, 238)`.
    medium_slate_blue, 0x7B68EE;
    /// Creates a new `Colour`, setting its RGB value to `(201, 220, 135)`.
    medium_spring_bud, 0xC9DC87;
    /// Creates a new `Colour`, setting its RGB value to `(0, 250, 154)`.
    medium_spring_green, 0x00FA9A;
    /// Creates a new `Colour`, setting its RGB value to `(103, 76, 71)`.
    medium_taupe, 0x674C47;
    /// Creates a new `Colour`, setting its RGB value to `(72, 209, 204)`.
    medium_turquoise, 0x48D1CC;
    /// Creates a new `Colour`, setting its RGB value to `(121, 68, 59)`.
    medium_tuscan_red, 0x79443B;
    /// Creates a new `Colour`, setting its RGB value to `(217, 96, 59)`.
    medium_vermilion, 0xD9603B;
    /// Creates a new `Colour`, setting its RGB value to `(199, 21, 133)`.
    medium_violet_red, 0xC71585;
    /// Creates a new `Colour`, setting its RGB value to `(248, 184, 120)`.
    mellow_apricot, 0xF8B878;
    /// Creates a new `Colour`, setting its RGB value to `(248, 222, 126)`.
    mellow_yellow, 0xF8DE7E;
    /// Creates a new `Colour`, setting its RGB value to `(253, 188, 180)`.
    melon, 0xFDBCB4;
    /// Creates a new `Colour`, setting its RGB value to `(10, 126, 140)`.
    metallic_seaweed, 0x0A7E8C;
    /// Creates a new `Colour`, setting its RGB value to `(156, 124, 56)`.
    metallic_sunburst, 0x9C7C38;
    /// Creates a new `Colour`, setting its RGB value to `(228, 0, 124)`.
    mexican_pink, 0xE4007C;
    /// Creates a new `Colour`, setting its RGB value to `(126, 212, 230)`.
    middle_blue, 0x7ED4E6;
    /// Creates a new `Colour`, setting its RGB value to `(141, 217, 204)`.
    middle_blue_green, 0x8DD9CC;
    /// Creates a new `Colour`, setting its RGB value to `(139, 114, 190)`.
    middle_blue_purple, 0x8B72BE;
    /// Creates a new `Colour`, setting its RGB value to `(77, 140, 87)`.
    middle_green, 0x4D8C57;
    /// Creates a new `Colour`, setting its RGB value to `(172, 191, 96)`.
    middle_green_yellow, 0xACBF60;
    /// Creates a new `Colour`, setting its RGB value to `(217, 130, 181)`.
    middle_purple, 0xD982B5;
    /// Creates a new `Colour`, setting its RGB value to `(229, 142, 115)`.
    middle_red, 0xE58E73;
    /// Creates a new `Colour`, setting its RGB value to `(165, 83, 83)`.
    middle_red_purple, 0xA55353;
    /// Creates a new `Colour`, setting its RGB value to `(255, 235, 0)`.
    middle_yellow, 0xFFEB00;
    /// Creates a new `Colour`, setting its RGB value to `(236, 177, 118)`.
    middle_yellow_red, 0xECB176;
    /// Creates a new `Colour`, setting its RGB value to `(112, 38, 112)`.
    midnight, 0x702670;
    /// Creates a new `Colour`, setting its RGB value to `(25, 25, 112)`.
    midnight_blue, 0x191970;
    /// Creates a new `Colour`, setting its RGB value to `(0, 73, 83)`.
    midnight_green_eagle_green, 0x004953;
    /// Creates a new `Colour`, setting its RGB value to `(255, 196, 12)`.
    mikado_yellow, 0xFFC40C;
    /// Creates a new `Colour`, setting its RGB value to `(253, 255, 245)`.
    milk, 0xFDFFF5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 218, 233)`.
    mimi_pink, 0xFFDAE9;
    /// Creates a new `Colour`, setting its RGB value to `(227, 249, 136)`.
    mindaro, 0xE3F988;
    /// Creates a new `Colour`, setting its RGB value to `(54, 116, 125)`.
    ming, 0x36747D;
    /// Creates a new `Colour`, setting its RGB value to `(245, 224, 80)`.
    minion_yellow, 0xF5E050;
    /// Creates a new `Colour`, setting its RGB value to `(62, 180, 137)`.
    mint, 0x3EB489;
    /// Creates a new `Colour`, setting its RGB value to `(245, 255, 250)`.
    mint_cream, 0xF5FFFA;
    /// Creates a new `Colour`, setting its RGB value to `(152, 255, 152)`.
    mint_green, 0x98FF98;
    /// Creates a new `Colour`, setting its RGB value to `(187, 180, 119)`.
    misty_moss, 0xBBB477;
    /// Creates a new `Colour`, setting its RGB value to `(255, 228, 225)`.
    misty_rose, 0xFFE4E1;
    /// Creates a new `Colour`, setting its RGB value to `(250, 235, 215)`.
    moccasin, 0xFAEBD7;
    /// Creates a new `Colour`, setting its RGB value to `(150, 113, 23)`.
    mode_beige, 0x967117;
    /// Creates a new `Colour`, setting its RGB value to `(115, 169, 194)`.
    moonstone_blue, 0x73A9C2;
    /// Creates a new `Colour`, setting its RGB value to `(174, 12, 0)`.
    mordant_red_19, 0xAE0C00;
    /// Creates a new `Colour`, setting its RGB value to `(141, 163, 153)`.
    morning_blue, 0x8DA399;
    /// Creates a new `Colour`, setting its RGB value to `(138, 154, 91)`.
    moss_green, 0x8A9A5B;
    /// Creates a new `Colour`, setting its RGB value to `(48, 186, 143)`.
    mountain_meadow, 0x30BA8F;
    /// Creates a new `Colour`, setting its RGB value to `(153, 122, 141)`.
    mountbatten_pink, 0x997A8D;
    /// Creates a new `Colour`, setting its RGB value to `(24, 69, 59)`.
    msu_green, 0x18453B;
    /// Creates a new `Colour`, setting its RGB value to `(48, 96, 48)`.
    mughal_green, 0x306030;
    /// Creates a new `Colour`, setting its RGB value to `(197, 75, 140)`.
    mulberry, 0xC54B8C;
    /// Creates a new `Colour`, setting its RGB value to `(130, 142, 132)`.
    mummys_tomb, 0x828E84;
    /// Creates a new `Colour`, setting its RGB value to `(255, 219, 88)`.
    mustard, 0xFFDB58;
    /// Creates a new `Colour`, setting its RGB value to `(49, 120, 115)`.
    myrtle_green, 0x317873;
    /// Creates a new `Colour`, setting its RGB value to `(214, 82, 130)`.
    mystic, 0xD65282;
    /// Creates a new `Colour`, setting its RGB value to `(173, 67, 121)`.
    mystic_maroon, 0xAD4379;
    /// Creates a new `Colour`, setting its RGB value to `(246, 173, 198)`.
    nadeshiko_pink, 0xF6ADC6;
    /// Creates a new `Colour`, setting its RGB value to `(42, 128, 0)`.
    napier_green, 0x2A8000;
    /// Creates a new `Colour`, setting its RGB value to `(250, 218, 94)`.
    naples_yellow, 0xFADA5E;
    /// Creates a new `Colour`, setting its RGB value to `(255, 222, 173)`.
    navajo_white, 0xFFDEAD;
    /// Creates a new `Colour`, setting its RGB value to `(0, 0, 128)`.
    navy, 0x000080;
    /// Creates a new `Colour`, setting its RGB value to `(148, 87, 235)`.
    navy_purple, 0x9457EB;
    /// Creates a new `Colour`, setting its RGB value to `(255, 163, 67)`.
    neon_carrot, 0xFFA343;
    /// Creates a new `Colour`, setting its RGB value to `(254, 65, 100)`.
    neon_fuchsia, 0xFE4164;
    /// Creates a new `Colour`, setting its RGB value to `(57, 255, 20)`.
    neon_green, 0x39FF14;
    /// Creates a new `Colour`, setting its RGB value to `(33, 79, 198)`.
    new_car, 0x214FC6;
    /// Creates a new `Colour`, setting its RGB value to `(215, 131, 127)`.
    new_york_pink, 0xD7837F;
    /// Creates a new `Colour`, setting its RGB value to `(114, 116, 114)`.
    nickel, 0x727472;
    /// Creates a new `Colour`, setting its RGB value to `(164, 221, 237)`.
    non_photo_blue, 0xA4DDED;
    /// Creates a new `Colour`, setting its RGB value to `(5, 144, 51)`.
    north_texas_green, 0x059033;
    /// Creates a new `Colour`, setting its RGB value to `(233, 255, 219)`.
    nyanza, 0xE9FFDB;
    /// Creates a new `Colour`, setting its RGB value to `(79, 66, 181)`.
    ocean_blue, 0x4F42B5;
    /// Creates a new `Colour`, setting its RGB value to `(0, 119, 190)`.
    ocean_boat_blue, 0x0077BE;
    /// Creates a new `Colour`, setting its RGB value to `(72, 191, 145)`.
    ocean_green, 0x48BF91;
    /// Creates a new `Colour`, setting its RGB value to `(204, 119, 34)`.
    ochre, 0xCC7722;
    /// Creates a new `Colour`, setting its RGB value to `(0, 128, 0)`.
    office_green, 0x008000;
    /// Creates a new `Colour`, setting its RGB value to `(253, 82, 64)`.
    ogre_odor, 0xFD5240;
    /// Creates a new `Colour`, setting its RGB value to `(67, 48, 46)`.
    old_burgundy, 0x43302E;
    /// Creates a new `Colour`, setting its RGB value to `(207, 181, 59)`.
    old_gold, 0xCFB53B;
    /// Creates a new `Colour`, setting its RGB value to `(86, 60, 92)`.
    old_heliotrope, 0x563C5C;
    /// Creates a new `Colour`, setting its RGB value to `(253, 245, 230)`.
    old_lace, 0xFDF5E6;
    /// Creates a new `Colour`, setting its RGB value to `(121, 104, 120)`.
    old_lavender, 0x796878;
    /// Creates a new `Colour`, setting its RGB value to `(103, 49, 71)`.
    old_mauve, 0x673147;
    /// Creates a new `Colour`, setting its RGB value to `(134, 126, 54)`.
    old_moss_green, 0x867E36;
    /// Creates a new `Colour`, setting its RGB value to `(192, 128, 129)`.
    old_rose, 0xC08081;
    /// Creates a new `Colour`, setting its RGB value to `(132, 132, 130)`.
    old_silver, 0x848482;
    /// Creates a new `Colour`, setting its RGB value to `(128, 128, 0)`.
    olive, 0x808000;
    /// Creates a new `Colour`, setting its RGB value to `(107, 142, 35)`.
    olive_drab_3, 0x6B8E23;
    /// Creates a new `Colour`, setting its RGB value to `(60, 52, 31)`.
    olive_drab_7, 0x3C341F;
    /// Creates a new `Colour`, setting its RGB value to `(154, 185, 115)`.
    olivine, 0x9AB973;
    /// Creates a new `Colour`, setting its RGB value to `(53, 56, 57)`.
    onyx, 0x353839;
    /// Creates a new `Colour`, setting its RGB value to `(183, 132, 167)`.
    opera_mauve, 0xB784A7;
    /// Creates a new `Colour`, setting its RGB value to `(230, 126, 34)`.
    orange, 0xE67E22;
    /// Creates a new `Colour`, setting its RGB value to `(255, 127, 0)`.
    orange_color_wheel, 0xFF7F00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 117, 56)`.
    orange_crayola, 0xFF7538;
    /// Creates a new `Colour`, setting its RGB value to `(255, 88, 0)`.
    orange_pantone, 0xFF5800;
    /// Creates a new `Colour`, setting its RGB value to `(255, 159, 0)`.
    orange_peel, 0xFF9F00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 69, 0)`.
    orange_red, 0xFF4500;
    /// Creates a new `Colour`, setting its RGB value to `(251, 153, 2)`.
    orange_ryb, 0xFB9902;
    /// Creates a new `Colour`, setting its RGB value to `(250, 91, 61)`.
    orange_soda, 0xFA5B3D;
    /// Creates a new `Colour`, setting its RGB value to `(255, 165, 0)`.
    orange_web, 0xFFA500;
    /// Creates a new `Colour`, setting its RGB value to `(248, 213, 104)`.
    orange_yellow, 0xF8D568;
    /// Creates a new `Colour`, setting its RGB value to `(218, 112, 214)`.
    orchid, 0xDA70D6;
    /// Creates a new `Colour`, setting its RGB value to `(242, 189, 205)`.
    orchid_pink, 0xF2BDCD;
    /// Creates a new `Colour`, setting its RGB value to `(251, 79, 20)`.
    orioles_orange, 0xFB4F14;
    /// Creates a new `Colour`, setting its RGB value to `(101, 67, 33)`.
    otter_brown, 0x654321;
    /// Creates a new `Colour`, setting its RGB value to `(153, 0, 0)`.
    ou_crimson_red, 0x990000;
    /// Creates a new `Colour`, setting its RGB value to `(65, 74, 76)`.
    outer_space, 0x414A4C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 110, 74)`.
    outrageous_orange, 0xFF6E4A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 33, 71)`.
    oxford_blue, 0x002147;
    /// Creates a new `Colour`, setting its RGB value to `(28, 169, 201)`.
    pacific_blue, 0x1CA9C9;
    /// Creates a new `Colour`, setting its RGB value to `(0, 102, 0)`.
    pakistan_green, 0x006600;
    /// Creates a new `Colour`, setting its RGB value to `(39, 59, 226)`.
    palatinate_blue, 0x273BE2;
    /// Creates a new `Colour`, setting its RGB value to `(104, 40, 96)`.
    palatinate_purple, 0x682860;
    /// Creates a new `Colour`, setting its RGB value to `(188, 212, 230)`.
    pale_aqua, 0xBCD4E6;
    /// Creates a new `Colour`, setting its RGB value to `(175, 238, 238)`.
    pale_blue, 0xAFEEEE;
    /// Creates a new `Colour`, setting its RGB value to `(152, 118, 84)`.
    pale_brown, 0x987654;
    /// Creates a new `Colour`, setting its RGB value to `(175, 64, 53)`.
    pale_carmine, 0xAF4035;
    /// Creates a new `Colour`, setting its RGB value to `(155, 196, 226)`.
    pale_cerulean, 0x9BC4E2;
    /// Creates a new `Colour`, setting its RGB value to `(221, 173, 175)`.
    pale_chestnut, 0xDDADAF;
    /// Creates a new `Colour`, setting its RGB value to `(218, 138, 103)`.
    pale_copper, 0xDA8A67;
    /// Creates a new `Colour`, setting its RGB value to `(171, 205, 239)`.
    pale_cornflower_blue, 0xABCDEF;
    /// Creates a new `Colour`, setting its RGB value to `(135, 211, 248)`.
    pale_cyan, 0x87D3F8;
    /// Creates a new `Colour`, setting its RGB value to `(230, 190, 138)`.
    pale_gold, 0xE6BE8A;
    /// Creates a new `Colour`, setting its RGB value to `(238, 232, 170)`.
    pale_goldenrod, 0xEEE8AA;
    /// Creates a new `Colour`, setting its RGB value to `(152, 251, 152)`.
    pale_green, 0x98FB98;
    /// Creates a new `Colour`, setting its RGB value to `(220, 208, 255)`.
    pale_lavender, 0xDCD0FF;
    /// Creates a new `Colour`, setting its RGB value to `(249, 132, 229)`.
    pale_magenta, 0xF984E5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 204)`.
    pale_magenta_pink, 0xFF99CC;
    /// Creates a new `Colour`, setting its RGB value to `(250, 218, 221)`.
    pale_pink, 0xFADADD;
    /// Creates a new `Colour`, setting its RGB value to `(221, 160, 221)`.
    pale_plum, 0xDDA0DD;
    /// Creates a new `Colour`, setting its RGB value to `(219, 112, 147)`.
    pale_red_violet, 0xDB7093;
    /// Creates a new `Colour`, setting its RGB value to `(150, 222, 209)`.
    pale_robin_egg_blue, 0x96DED1;
    /// Creates a new `Colour`, setting its RGB value to `(201, 192, 187)`.
    pale_silver, 0xC9C0BB;
    /// Creates a new `Colour`, setting its RGB value to `(236, 235, 189)`.
    pale_spring_bud, 0xECEBBD;
    /// Creates a new `Colour`, setting its RGB value to `(188, 152, 126)`.
    pale_taupe, 0xBC987E;
    /// Creates a new `Colour`, setting its RGB value to `(175, 238, 238)`.
    pale_turquoise, 0xAFEEEE;
    /// Creates a new `Colour`, setting its RGB value to `(204, 153, 255)`.
    pale_violet, 0xCC99FF;
    /// Creates a new `Colour`, setting its RGB value to `(219, 112, 147)`.
    pale_violet_red, 0xDB7093;
    /// Creates a new `Colour`, setting its RGB value to `(111, 153, 64)`.
    palm_leaf, 0x6F9940;
    /// Creates a new `Colour`, setting its RGB value to `(120, 24, 74)`.
    pansy_purple, 0x78184A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 155, 125)`.
    paolo_veronese_green, 0x009B7D;
    /// Creates a new `Colour`, setting its RGB value to `(255, 239, 213)`.
    papaya_whip, 0xFFEFD5;
    /// Creates a new `Colour`, setting its RGB value to `(230, 62, 98)`.
    paradise_pink, 0xE63E62;
    /// Creates a new `Colour`, setting its RGB value to `(80, 200, 120)`.
    paris_green, 0x50C878;
    /// Creates a new `Colour`, setting its RGB value to `(217, 152, 160)`.
    parrot_pink, 0xD998A0;
    /// Creates a new `Colour`, setting its RGB value to `(174, 198, 207)`.
    pastel_blue, 0xAEC6CF;
    /// Creates a new `Colour`, setting its RGB value to `(131, 105, 83)`.
    pastel_brown, 0x836953;
    /// Creates a new `Colour`, setting its RGB value to `(207, 207, 196)`.
    pastel_gray, 0xCFCFC4;
    /// Creates a new `Colour`, setting its RGB value to `(119, 221, 119)`.
    pastel_green, 0x77DD77;
    /// Creates a new `Colour`, setting its RGB value to `(244, 154, 194)`.
    pastel_magenta, 0xF49AC2;
    /// Creates a new `Colour`, setting its RGB value to `(255, 179, 71)`.
    pastel_orange, 0xFFB347;
    /// Creates a new `Colour`, setting its RGB value to `(222, 165, 164)`.
    pastel_pink, 0xDEA5A4;
    /// Creates a new `Colour`, setting its RGB value to `(179, 158, 181)`.
    pastel_purple, 0xB39EB5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 105, 97)`.
    pastel_red, 0xFF6961;
    /// Creates a new `Colour`, setting its RGB value to `(203, 153, 201)`.
    pastel_violet, 0xCB99C9;
    /// Creates a new `Colour`, setting its RGB value to `(253, 253, 150)`.
    pastel_yellow, 0xFDFD96;
    /// Creates a new `Colour`, setting its RGB value to `(128, 0, 128)`.
    patriarch, 0x800080;
    /// Creates a new `Colour`, setting its RGB value to `(83, 104, 120)`.
    paynes_grey, 0x536878;
    /// Creates a new `Colour`, setting its RGB value to `(255, 203, 164)`.
    peach, 0xFFCBA4;
    /// Creates a new `Colour`, setting its RGB value to `(255, 204, 153)`.
    peach_orange, 0xFFCC99;
    /// Creates a new `Colour`, setting its RGB value to `(255, 218, 185)`.
    peach_puff, 0xFFDAB9;
    /// Creates a new `Colour`, setting its RGB value to `(250, 223, 173)`.
    peach_yellow, 0xFADFAD;
    /// Creates a new `Colour`, setting its RGB value to `(209, 226, 49)`.
    pear, 0xD1E231;
    /// Creates a new `Colour`, setting its RGB value to `(234, 224, 200)`.
    pearl, 0xEAE0C8;
    /// Creates a new `Colour`, setting its RGB value to `(136, 216, 192)`.
    pearl_aqua, 0x88D8C0;
    /// Creates a new `Colour`, setting its RGB value to `(183, 104, 162)`.
    pearly_purple, 0xB768A2;
    /// Creates a new `Colour`, setting its RGB value to `(230, 226, 0)`.
    peridot, 0xE6E200;
    /// Creates a new `Colour`, setting its RGB value to `(204, 204, 255)`.
    periwinkle, 0xCCCCFF;
    /// Creates a new `Colour`, setting its RGB value to `(225, 44, 44)`.
    permanent_geranium_lake, 0xE12C2C;
    /// Creates a new `Colour`, setting its RGB value to `(28, 57, 187)`.
    persian_blue, 0x1C39BB;
    /// Creates a new `Colour`, setting its RGB value to `(0, 166, 147)`.
    persian_green, 0x00A693;
    /// Creates a new `Colour`, setting its RGB value to `(50, 18, 122)`.
    persian_indigo, 0x32127A;
    /// Creates a new `Colour`, setting its RGB value to `(217, 144, 88)`.
    persian_orange, 0xD99058;
    /// Creates a new `Colour`, setting its RGB value to `(247, 127, 190)`.
    persian_pink, 0xF77FBE;
    /// Creates a new `Colour`, setting its RGB value to `(112, 28, 28)`.
    persian_plum, 0x701C1C;
    /// Creates a new `Colour`, setting its RGB value to `(204, 51, 51)`.
    persian_red, 0xCC3333;
    /// Creates a new `Colour`, setting its RGB value to `(254, 40, 162)`.
    persian_rose, 0xFE28A2;
    /// Creates a new `Colour`, setting its RGB value to `(236, 88, 0)`.
    persimmon, 0xEC5800;
    /// Creates a new `Colour`, setting its RGB value to `(205, 133, 63)`.
    peru, 0xCD853F;
    /// Creates a new `Colour`, setting its RGB value to `(139, 168, 183)`.
    pewter_blue, 0x8BA8B7;
    /// Creates a new `Colour`, setting its RGB value to `(223, 0, 255)`.
    phlox, 0xDF00FF;
    /// Creates a new `Colour`, setting its RGB value to `(0, 15, 137)`.
    phthalo_blue, 0x000F89;
    /// Creates a new `Colour`, setting its RGB value to `(18, 53, 36)`.
    phthalo_green, 0x123524;
    /// Creates a new `Colour`, setting its RGB value to `(69, 177, 232)`.
    picton_blue, 0x45B1E8;
    /// Creates a new `Colour`, setting its RGB value to `(195, 11, 78)`.
    pictorial_carmine, 0xC30B4E;
    /// Creates a new `Colour`, setting its RGB value to `(253, 221, 230)`.
    piggy_pink, 0xFDDDE6;
    /// Creates a new `Colour`, setting its RGB value to `(1, 121, 111)`.
    pine_green, 0x01796F;
    /// Creates a new `Colour`, setting its RGB value to `(86, 60, 92)`.
    pineapple, 0x563C5C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 192, 203)`.
    pink, 0xFFC0CB;
    /// Creates a new `Colour`, setting its RGB value to `(252, 116, 253)`.
    pink_flamingo, 0xFC74FD;
    /// Creates a new `Colour`, setting its RGB value to `(255, 221, 244)`.
    pink_lace, 0xFFDDF4;
    /// Creates a new `Colour`, setting its RGB value to `(216, 178, 209)`.
    pink_lavender, 0xD8B2D1;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 102)`.
    pink_orange, 0xFF9966;
    /// Creates a new `Colour`, setting its RGB value to `(215, 72, 148)`.
    pink_pantone, 0xD74894;
    /// Creates a new `Colour`, setting its RGB value to `(231, 172, 207)`.
    pink_pearl, 0xE7ACCF;
    /// Creates a new `Colour`, setting its RGB value to `(152, 0, 54)`.
    pink_raspberry, 0x980036;
    /// Creates a new `Colour`, setting its RGB value to `(247, 143, 167)`.
    pink_sherbet, 0xF78FA7;
    /// Creates a new `Colour`, setting its RGB value to `(147, 197, 114)`.
    pistachio, 0x93C572;
    /// Creates a new `Colour`, setting its RGB value to `(57, 18, 133)`.
    pixie_powder, 0x391285;
    /// Creates a new `Colour`, setting its RGB value to `(229, 228, 226)`.
    platinum, 0xE5E4E2;
    /// Creates a new `Colour`, setting its RGB value to `(142, 69, 133)`.
    plum, 0x8E4585;
    /// Creates a new `Colour`, setting its RGB value to `(221, 160, 221)`.
    plum_web, 0xDDA0DD;
    /// Creates a new `Colour`, setting its RGB value to `(89, 70, 178)`.
    plump_purple, 0x5946B2;
    /// Creates a new `Colour`, setting its RGB value to `(93, 164, 147)`.
    polished_pine, 0x5DA493;
    /// Creates a new `Colour`, setting its RGB value to `(134, 96, 142)`.
    pomp_and_power, 0x86608E;
    /// Creates a new `Colour`, setting its RGB value to `(190, 79, 98)`.
    popstar, 0xBE4F62;
    /// Creates a new `Colour`, setting its RGB value to `(255, 90, 54)`.
    portland_orange, 0xFF5A36;
    /// Creates a new `Colour`, setting its RGB value to `(176, 224, 230)`.
    powder_blue, 0xB0E0E6;
    /// Creates a new `Colour`, setting its RGB value to `(255, 133, 207)`.
    princess_perfume, 0xFF85CF;
    /// Creates a new `Colour`, setting its RGB value to `(245, 128, 37)`.
    princeton_orange, 0xF58025;
    /// Creates a new `Colour`, setting its RGB value to `(112, 28, 28)`.
    prune, 0x701C1C;
    /// Creates a new `Colour`, setting its RGB value to `(0, 49, 83)`.
    prussian_blue, 0x003153;
    /// Creates a new `Colour`, setting its RGB value to `(223, 0, 255)`.
    psychedelic_purple, 0xDF00FF;
    /// Creates a new `Colour`, setting its RGB value to `(204, 136, 153)`.
    puce, 0xCC8899;
    /// Creates a new `Colour`, setting its RGB value to `(114, 47, 55)`.
    puce_red, 0x722F37;
    /// Creates a new `Colour`, setting its RGB value to `(100, 65, 23)`.
    pullman_brown_ups_brown, 0x644117;
    /// Creates a new `Colour`, setting its RGB value to `(59, 51, 28)`.
    pullman_green, 0x3B331C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 117, 24)`.
    pumpkin, 0xFF7518;
    /// Creates a new `Colour`, setting its RGB value to `(155, 89, 182)`.
    purple, 0x9B59B6;
    /// Creates a new `Colour`, setting its RGB value to `(105, 53, 156)`.
    purple_heart, 0x69359C;
    /// Creates a new `Colour`, setting its RGB value to `(128, 0, 128)`.
    purple_html, 0x800080;
    /// Creates a new `Colour`, setting its RGB value to `(150, 120, 182)`.
    purple_mountain_majesty, 0x9678B6;
    /// Creates a new `Colour`, setting its RGB value to `(159, 0, 197)`.
    purple_munsell, 0x9F00C5;
    /// Creates a new `Colour`, setting its RGB value to `(78, 81, 128)`.
    purple_navy, 0x4E5180;
    /// Creates a new `Colour`, setting its RGB value to `(254, 78, 218)`.
    purple_pizzazz, 0xFE4EDA;
    /// Creates a new `Colour`, setting its RGB value to `(156, 81, 182)`.
    purple_plum, 0x9C51B6;
    /// Creates a new `Colour`, setting its RGB value to `(80, 64, 77)`.
    purple_taupe, 0x50404D;
    /// Creates a new `Colour`, setting its RGB value to `(160, 32, 240)`.
    purple_x11, 0xA020F0;
    /// Creates a new `Colour`, setting its RGB value to `(154, 78, 174)`.
    purpureus, 0x9A4EAE;
    /// Creates a new `Colour`, setting its RGB value to `(81, 72, 79)`.
    quartz, 0x51484F;
    /// Creates a new `Colour`, setting its RGB value to `(67, 107, 149)`.
    queen_blue, 0x436B95;
    /// Creates a new `Colour`, setting its RGB value to `(232, 204, 215)`.
    queen_pink, 0xE8CCD7;
    /// Creates a new `Colour`, setting its RGB value to `(166, 166, 166)`.
    quick_silver, 0xA6A6A6;
    /// Creates a new `Colour`, setting its RGB value to `(142, 58, 89)`.
    quinacridone_magenta, 0x8E3A59;
    /// Creates a new `Colour`, setting its RGB value to `(93, 138, 168)`.
    rackley, 0x5D8AA8;
    /// Creates a new `Colour`, setting its RGB value to `(255, 53, 94)`.
    radical_red, 0xFF355E;
    /// Creates a new `Colour`, setting its RGB value to `(36, 33, 36)`.
    raisin_black, 0x242124;
    /// Creates a new `Colour`, setting its RGB value to `(251, 171, 96)`.
    rajah, 0xFBAB60;
    /// Creates a new `Colour`, setting its RGB value to `(227, 11, 93)`.
    raspberry, 0xE30B5D;
    /// Creates a new `Colour`, setting its RGB value to `(145, 95, 109)`.
    raspberry_glace, 0x915F6D;
    /// Creates a new `Colour`, setting its RGB value to `(226, 80, 152)`.
    raspberry_pink, 0xE25098;
    /// Creates a new `Colour`, setting its RGB value to `(179, 68, 108)`.
    raspberry_rose, 0xB3446C;
    /// Creates a new `Colour`, setting its RGB value to `(214, 138, 89)`.
    raw_sienna, 0xD68A59;
    /// Creates a new `Colour`, setting its RGB value to `(130, 102, 68)`.
    raw_umber, 0x826644;
    /// Creates a new `Colour`, setting its RGB value to `(255, 51, 204)`.
    razzle_dazzle_rose, 0xFF33CC;
    /// Creates a new `Colour`, setting its RGB value to `(227, 37, 107)`.
    razzmatazz, 0xE3256B;
    /// Creates a new `Colour`, setting its RGB value to `(141, 78, 133)`.
    razzmic_berry, 0x8D4E85;
    /// Creates a new `Colour`, setting its RGB value to `(102, 51, 153)`.
    rebecca_purple, 0x663399;
    /// Creates a new `Colour`, setting its RGB value to `(231, 76, 60)`.
    red, 0xE74C3C;
    /// Creates a new `Colour`, setting its RGB value to `(165, 42, 42)`.
    red_brown, 0xA52A2A;
    /// Creates a new `Colour`, setting its RGB value to `(238, 32, 77)`.
    red_crayola, 0xEE204D;
    /// Creates a new `Colour`, setting its RGB value to `(134, 1, 17)`.
    red_devil, 0x860111;
    /// Creates a new `Colour`, setting its RGB value to `(242, 0, 60)`.
    red_munsell, 0xF2003C;
    /// Creates a new `Colour`, setting its RGB value to `(196, 2, 51)`.
    red_ncs, 0xC40233;
    /// Creates a new `Colour`, setting its RGB value to `(255, 83, 73)`.
    red_orange, 0xFF5349;
    /// Creates a new `Colour`, setting its RGB value to `(237, 41, 57)`.
    red_pantone, 0xED2939;
    /// Creates a new `Colour`, setting its RGB value to `(237, 28, 36)`.
    red_pigment, 0xED1C24;
    /// Creates a new `Colour`, setting its RGB value to `(228, 0, 120)`.
    red_purple, 0xE40078;
    /// Creates a new `Colour`, setting its RGB value to `(254, 39, 18)`.
    red_ryb, 0xFE2712;
    /// Creates a new `Colour`, setting its RGB value to `(253, 58, 74)`.
    red_salsa, 0xFD3A4A;
    /// Creates a new `Colour`, setting its RGB value to `(199, 21, 133)`.
    red_violet, 0xC71585;
    /// Creates a new `Colour`, setting its RGB value to `(164, 90, 82)`.
    redwood, 0xA45A52;
    /// Creates a new `Colour`, setting its RGB value to `(82, 45, 128)`.
    regalia, 0x522D80;
    /// Creates a new `Colour`, setting its RGB value to `(0, 0, 0)`.
    registration_black, 0x000000;
    /// Creates a new `Colour`, setting its RGB value to `(0, 35, 135)`.
    resolution_blue, 0x002387;
    /// Creates a new `Colour`, setting its RGB value to `(119, 118, 150)`.
    rhythm, 0x777696;
    /// Creates a new `Colour`, setting its RGB value to `(0, 64, 64)`.
    rich_black, 0x004040;
    /// Creates a new `Colour`, setting its RGB value to `(1, 11, 19)`.
    rich_black_fogra29, 0x010B13;
    /// Creates a new `Colour`, setting its RGB value to `(1, 2, 3)`.
    rich_black_fogra39, 0x010203;
    /// Creates a new `Colour`, setting its RGB value to `(241, 167, 254)`.
    rich_brilliant_lavender, 0xF1A7FE;
    /// Creates a new `Colour`, setting its RGB value to `(215, 0, 64)`.
    rich_carmine, 0xD70040;
    /// Creates a new `Colour`, setting its RGB value to `(8, 146, 208)`.
    rich_electric_blue, 0x0892D0;
    /// Creates a new `Colour`, setting its RGB value to `(167, 107, 207)`.
    rich_lavender, 0xA76BCF;
    /// Creates a new `Colour`, setting its RGB value to `(182, 102, 210)`.
    rich_lilac, 0xB666D2;
    /// Creates a new `Colour`, setting its RGB value to `(176, 48, 96)`.
    rich_maroon, 0xB03060;
    /// Creates a new `Colour`, setting its RGB value to `(68, 76, 56)`.
    rifle_green, 0x444C38;
    /// Creates a new `Colour`, setting its RGB value to `(112, 66, 65)`.
    roast_coffee, 0x704241;
    /// Creates a new `Colour`, setting its RGB value to `(0, 204, 204)`.
    robin_egg_blue, 0x00CCCC;
    /// Creates a new `Colour`, setting its RGB value to `(138, 127, 128)`.
    rocket_metallic, 0x8A7F80;
    /// Creates a new `Colour`, setting its RGB value to `(131, 137, 150)`.
    roman_silver, 0x838996;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 127)`.
    rose, 0xFF007F;
    /// Creates a new `Colour`, setting its RGB value to `(249, 66, 158)`.
    rose_bonbon, 0xF9429E;
    /// Creates a new `Colour`, setting its RGB value to `(158, 94, 111)`.
    rose_dust, 0x9E5E6F;
    /// Creates a new `Colour`, setting its RGB value to `(103, 72, 70)`.
    rose_ebony, 0x674846;
    /// Creates a new `Colour`, setting its RGB value to `(183, 110, 121)`.
    rose_gold, 0xB76E79;
    /// Creates a new `Colour`, setting its RGB value to `(227, 38, 54)`.
    rose_madder, 0xE32636;
    /// Creates a new `Colour`, setting its RGB value to `(255, 102, 204)`.
    rose_pink, 0xFF66CC;
    /// Creates a new `Colour`, setting its RGB value to `(170, 152, 169)`.
    rose_quartz, 0xAA98A9;
    /// Creates a new `Colour`, setting its RGB value to `(194, 30, 86)`.
    rose_red, 0xC21E56;
    /// Creates a new `Colour`, setting its RGB value to `(144, 93, 93)`.
    rose_taupe, 0x905D5D;
    /// Creates a new `Colour`, setting its RGB value to `(171, 78, 82)`.
    rose_vale, 0xAB4E52;
    /// Creates a new `Colour`, setting its RGB value to `(101, 0, 11)`.
    rosewood, 0x65000B;
    /// Creates a new `Colour`, setting its RGB value to `(212, 0, 0)`.
    rosso_corsa, 0xD40000;
    /// Creates a new `Colour`, setting its RGB value to `(188, 143, 143)`.
    rosy_brown, 0xBC8F8F;
    /// Creates a new `Colour`, setting its RGB value to `(0, 56, 168)`.
    royal_azure, 0x0038A8;
    /// Creates a new `Colour`, setting its RGB value to `(65, 105, 225)`.
    royal_blue, 0x4169E1;
    /// Creates a new `Colour`, setting its RGB value to `(202, 44, 146)`.
    royal_fuchsia, 0xCA2C92;
    /// Creates a new `Colour`, setting its RGB value to `(120, 81, 169)`.
    royal_purple, 0x7851A9;
    /// Creates a new `Colour`, setting its RGB value to `(250, 218, 94)`.
    royal_yellow, 0xFADA5E;
    /// Creates a new `Colour`, setting its RGB value to `(206, 70, 118)`.
    ruber, 0xCE4676;
    /// Creates a new `Colour`, setting its RGB value to `(209, 0, 86)`.
    rubine_red, 0xD10056;
    /// Creates a new `Colour`, setting its RGB value to `(224, 17, 95)`.
    ruby, 0xE0115F;
    /// Creates a new `Colour`, setting its RGB value to `(155, 17, 30)`.
    ruby_red, 0x9B111E;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 40)`.
    ruddy, 0xFF0028;
    /// Creates a new `Colour`, setting its RGB value to `(187, 101, 40)`.
    ruddy_brown, 0xBB6528;
    /// Creates a new `Colour`, setting its RGB value to `(225, 142, 150)`.
    ruddy_pink, 0xE18E96;
    /// Creates a new `Colour`, setting its RGB value to `(168, 28, 7)`.
    rufous, 0xA81C07;
    /// Creates a new `Colour`, setting its RGB value to `(128, 70, 27)`.
    russet, 0x80461B;
    /// Creates a new `Colour`, setting its RGB value to `(103, 146, 103)`.
    russian_green, 0x679267;
    /// Creates a new `Colour`, setting its RGB value to `(50, 23, 77)`.
    russian_violet, 0x32174D;
    /// Creates a new `Colour`, setting its RGB value to `(183, 65, 14)`.
    rust, 0xB7410E;
    /// Creates a new `Colour`, setting its RGB value to `(218, 44, 67)`.
    rusty_red, 0xDA2C43;
    /// Creates a new `Colour`, setting its RGB value to `(0, 86, 63)`.
    sacramento_state_green, 0x00563F;
    /// Creates a new `Colour`, setting its RGB value to `(139, 69, 19)`.
    saddle_brown, 0x8B4513;
    /// Creates a new `Colour`, setting its RGB value to `(255, 120, 0)`.
    safety_orange, 0xFF7800;
    /// Creates a new `Colour`, setting its RGB value to `(255, 103, 0)`.
    safety_orange_blaze_orange, 0xFF6700;
    /// Creates a new `Colour`, setting its RGB value to `(238, 210, 2)`.
    safety_yellow, 0xEED202;
    /// Creates a new `Colour`, setting its RGB value to `(244, 196, 48)`.
    saffron, 0xF4C430;
    /// Creates a new `Colour`, setting its RGB value to `(188, 184, 138)`.
    sage, 0xBCB88A;
    /// Creates a new `Colour`, setting its RGB value to `(250, 128, 114)`.
    salmon, 0xFA8072;
    /// Creates a new `Colour`, setting its RGB value to `(255, 145, 164)`.
    salmon_pink, 0xFF91A4;
    /// Creates a new `Colour`, setting its RGB value to `(194, 178, 128)`.
    sand, 0xC2B280;
    /// Creates a new `Colour`, setting its RGB value to `(150, 113, 23)`.
    sand_dune, 0x967117;
    /// Creates a new `Colour`, setting its RGB value to `(236, 213, 64)`.
    sandstorm, 0xECD540;
    /// Creates a new `Colour`, setting its RGB value to `(244, 164, 96)`.
    sandy_brown, 0xF4A460;
    /// Creates a new `Colour`, setting its RGB value to `(253, 217, 181)`.
    sandy_tan, 0xFDD9B5;
    /// Creates a new `Colour`, setting its RGB value to `(150, 113, 23)`.
    sandy_taupe, 0x967117;
    /// Creates a new `Colour`, setting its RGB value to `(146, 0, 10)`.
    sangria, 0x92000A;
    /// Creates a new `Colour`, setting its RGB value to `(80, 125, 42)`.
    sap_green, 0x507D2A;
    /// Creates a new `Colour`, setting its RGB value to `(15, 82, 186)`.
    sapphire, 0x0F52BA;
    /// Creates a new `Colour`, setting its RGB value to `(0, 103, 165)`.
    sapphire_blue, 0x0067A5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 70, 129)`.
    sasquatch_socks, 0xFF4681;
    /// Creates a new `Colour`, setting its RGB value to `(203, 161, 53)`.
    satin_sheen_gold, 0xCBA135;
    /// Creates a new `Colour`, setting its RGB value to `(253, 14, 53)`.
    scarlet, 0xFD0E35;
    /// Creates a new `Colour`, setting its RGB value to `(255, 145, 175)`.
    schauss_pink, 0xFF91AF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 216, 0)`.
    school_bus_yellow, 0xFFD800;
    /// Creates a new `Colour`, setting its RGB value to `(102, 255, 102)`.
    screamin_green, 0x66FF66;
    /// Creates a new `Colour`, setting its RGB value to `(0, 105, 148)`.
    sea_blue, 0x006994;
    /// Creates a new `Colour`, setting its RGB value to `(159, 226, 191)`.
    sea_foam_green, 0x9FE2BF;
    /// Creates a new `Colour`, setting its RGB value to `(46, 139, 87)`.
    sea_green, 0x2E8B57;
    /// Creates a new `Colour`, setting its RGB value to `(75, 199, 207)`.
    sea_serpent, 0x4BC7CF;
    /// Creates a new `Colour`, setting its RGB value to `(89, 38, 11)`.
    seal_brown, 0x59260B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 245, 238)`.
    seashell, 0xFFF5EE;
    /// Creates a new `Colour`, setting its RGB value to `(255, 186, 0)`.
    selective_yellow, 0xFFBA00;
    /// Creates a new `Colour`, setting its RGB value to `(112, 66, 20)`.
    sepia, 0x704214;
    /// Creates a new `Colour`, setting its RGB value to `(138, 121, 93)`.
    shadow, 0x8A795D;
    /// Creates a new `Colour`, setting its RGB value to `(119, 139, 165)`.
    shadow_blue, 0x778BA5;
    /// Creates a new `Colour`, setting its RGB value to `(255, 207, 241)`.
    shampoo, 0xFFCFF1;
    /// Creates a new `Colour`, setting its RGB value to `(0, 158, 96)`.
    shamrock_green, 0x009E60;
    /// Creates a new `Colour`, setting its RGB value to `(143, 212, 0)`.
    sheen_green, 0x8FD400;
    /// Creates a new `Colour`, setting its RGB value to `(217, 134, 149)`.
    shimmering_blush, 0xD98695;
    /// Creates a new `Colour`, setting its RGB value to `(95, 167, 120)`.
    shiny_shamrock, 0x5FA778;
    /// Creates a new `Colour`, setting its RGB value to `(252, 15, 192)`.
    shocking_pink, 0xFC0FC0;
    /// Creates a new `Colour`, setting its RGB value to `(255, 111, 255)`.
    shocking_pink_crayola, 0xFF6FFF;
    /// Creates a new `Colour`, setting its RGB value to `(136, 45, 23)`.
    sienna, 0x882D17;
    /// Creates a new `Colour`, setting its RGB value to `(192, 192, 192)`.
    silver, 0xC0C0C0;
    /// Creates a new `Colour`, setting its RGB value to `(172, 172, 172)`.
    silver_chalice, 0xACACAC;
    /// Creates a new `Colour`, setting its RGB value to `(93, 137, 186)`.
    silver_lake_blue, 0x5D89BA;
    /// Creates a new `Colour`, setting its RGB value to `(196, 174, 173)`.
    silver_pink, 0xC4AEAD;
    /// Creates a new `Colour`, setting its RGB value to `(191, 193, 194)`.
    silver_sand, 0xBFC1C2;
    /// Creates a new `Colour`, setting its RGB value to `(203, 65, 11)`.
    sinopia, 0xCB410B;
    /// Creates a new `Colour`, setting its RGB value to `(255, 56, 85)`.
    sizzling_red, 0xFF3855;
    /// Creates a new `Colour`, setting its RGB value to `(255, 219, 0)`.
    sizzling_sunrise, 0xFFDB00;
    /// Creates a new `Colour`, setting its RGB value to `(0, 116, 116)`.
    skobeloff, 0x007474;
    /// Creates a new `Colour`, setting its RGB value to `(135, 206, 235)`.
    sky_blue, 0x87CEEB;
    /// Creates a new `Colour`, setting its RGB value to `(207, 113, 175)`.
    sky_magenta, 0xCF71AF;
    /// Creates a new `Colour`, setting its RGB value to `(106, 90, 205)`.
    slate_blue, 0x6A5ACD;
    /// Creates a new `Colour`, setting its RGB value to `(112, 128, 144)`.
    slate_gray, 0x708090;
    /// Creates a new `Colour`, setting its RGB value to `(41, 150, 23)`.
    slimy_green, 0x299617;
    /// Creates a new `Colour`, setting its RGB value to `(0, 51, 153)`.
    smalt_dark_powder_blue, 0x003399;
    /// Creates a new `Colour`, setting its RGB value to `(255, 109, 58)`.
    smashed_pumpkin, 0xFF6D3A;
    /// Creates a new `Colour`, setting its RGB value to `(200, 65, 134)`.
    smitten, 0xC84186;
    /// Creates a new `Colour`, setting its RGB value to `(115, 130, 118)`.
    smoke, 0x738276;
    /// Creates a new `Colour`, setting its RGB value to `(131, 42, 13)`.
    smokey_topaz, 0x832A0D;
    /// Creates a new `Colour`, setting its RGB value to `(16, 12, 8)`.
    smoky_black, 0x100C08;
    /// Creates a new `Colour`, setting its RGB value to `(147, 61, 65)`.
    smoky_topaz, 0x933D41;
    /// Creates a new `Colour`, setting its RGB value to `(255, 250, 250)`.
    snow, 0xFFFAFA;
    /// Creates a new `Colour`, setting its RGB value to `(206, 200, 239)`.
    soap, 0xCEC8EF;
    /// Creates a new `Colour`, setting its RGB value to `(137, 56, 67)`.
    solid_pink, 0x893843;
    /// Creates a new `Colour`, setting its RGB value to `(117, 117, 117)`.
    sonic_silver, 0x757575;
    /// Creates a new `Colour`, setting its RGB value to `(29, 41, 81)`.
    space_cadet, 0x1D2951;
    /// Creates a new `Colour`, setting its RGB value to `(128, 117, 50)`.
    spanish_bistre, 0x807532;
    /// Creates a new `Colour`, setting its RGB value to `(0, 112, 184)`.
    spanish_blue, 0x0070B8;
    /// Creates a new `Colour`, setting its RGB value to `(209, 0, 71)`.
    spanish_carmine, 0xD10047;
    /// Creates a new `Colour`, setting its RGB value to `(229, 26, 76)`.
    spanish_crimson, 0xE51A4C;
    /// Creates a new `Colour`, setting its RGB value to `(152, 152, 152)`.
    spanish_gray, 0x989898;
    /// Creates a new `Colour`, setting its RGB value to `(0, 145, 80)`.
    spanish_green, 0x009150;
    /// Creates a new `Colour`, setting its RGB value to `(232, 97, 0)`.
    spanish_orange, 0xE86100;
    /// Creates a new `Colour`, setting its RGB value to `(247, 191, 190)`.
    spanish_pink, 0xF7BFBE;
    /// Creates a new `Colour`, setting its RGB value to `(230, 0, 38)`.
    spanish_red, 0xE60026;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 255)`.
    spanish_sky_blue, 0x00FFFF;
    /// Creates a new `Colour`, setting its RGB value to `(76, 40, 130)`.
    spanish_violet, 0x4C2882;
    /// Creates a new `Colour`, setting its RGB value to `(0, 127, 92)`.
    spanish_viridian, 0x007F5C;
    /// Creates a new `Colour`, setting its RGB value to `(158, 19, 22)`.
    spartan_crimson, 0x9E1316;
    /// Creates a new `Colour`, setting its RGB value to `(139, 95, 77)`.
    spicy_mix, 0x8B5F4D;
    /// Creates a new `Colour`, setting its RGB value to `(15, 192, 252)`.
    spiro_disco_ball, 0x0FC0FC;
    /// Creates a new `Colour`, setting its RGB value to `(167, 252, 0)`.
    spring_bud, 0xA7FC00;
    /// Creates a new `Colour`, setting its RGB value to `(135, 255, 42)`.
    spring_frost, 0x87FF2A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 127)`.
    spring_green, 0x00FF7F;
    /// Creates a new `Colour`, setting its RGB value to `(35, 41, 122)`.
    st_patricks_blue, 0x23297A;
    /// Creates a new `Colour`, setting its RGB value to `(0, 123, 184)`.
    star_command_blue, 0x007BB8;
    /// Creates a new `Colour`, setting its RGB value to `(70, 130, 180)`.
    steel_blue, 0x4682B4;
    /// Creates a new `Colour`, setting its RGB value to `(204, 51, 204)`.
    steel_pink, 0xCC33CC;
    /// Creates a new `Colour`, setting its RGB value to `(95, 138, 139)`.
    steel_teal, 0x5F8A8B;
    /// Creates a new `Colour`, setting its RGB value to `(250, 218, 94)`.
    stil_de_grain_yellow, 0xFADA5E;
    /// Creates a new `Colour`, setting its RGB value to `(153, 0, 0)`.
    stizza, 0x990000;
    /// Creates a new `Colour`, setting its RGB value to `(79, 102, 106)`.
    stormcloud, 0x4F666A;
    /// Creates a new `Colour`, setting its RGB value to `(228, 217, 111)`.
    straw, 0xE4D96F;
    /// Creates a new `Colour`, setting its RGB value to `(252, 90, 141)`.
    strawberry, 0xFC5A8D;
    /// Creates a new `Colour`, setting its RGB value to `(145, 78, 117)`.
    sugar_plum, 0x914E75;
    /// Creates a new `Colour`, setting its RGB value to `(255, 64, 76)`.
    sunburnt_cyclops, 0xFF404C;
    /// Creates a new `Colour`, setting its RGB value to `(255, 204, 51)`.
    sunglow, 0xFFCC33;
    /// Creates a new `Colour`, setting its RGB value to `(242, 242, 122)`.
    sunny, 0xF2F27A;
    /// Creates a new `Colour`, setting its RGB value to `(227, 171, 87)`.
    sunray, 0xE3AB57;
    /// Creates a new `Colour`, setting its RGB value to `(250, 214, 165)`.
    sunset, 0xFAD6A5;
    /// Creates a new `Colour`, setting its RGB value to `(253, 94, 83)`.
    sunset_orange, 0xFD5E53;
    /// Creates a new `Colour`, setting its RGB value to `(207, 107, 169)`.
    super_pink, 0xCF6BA9;
    /// Creates a new `Colour`, setting its RGB value to `(168, 55, 49)`.
    sweet_brown, 0xA83731;
    /// Creates a new `Colour`, setting its RGB value to `(210, 180, 140)`.
    tan, 0xD2B48C;
    /// Creates a new `Colour`, setting its RGB value to `(249, 77, 0)`.
    tangelo, 0xF94D00;
    /// Creates a new `Colour`, setting its RGB value to `(242, 133, 0)`.
    tangerine, 0xF28500;
    /// Creates a new `Colour`, setting its RGB value to `(255, 204, 0)`.
    tangerine_yellow, 0xFFCC00;
    /// Creates a new `Colour`, setting its RGB value to `(228, 113, 122)`.
    tango_pink, 0xE4717A;
    /// Creates a new `Colour`, setting its RGB value to `(251, 77, 70)`.
    tart_orange, 0xFB4D46;
    /// Creates a new `Colour`, setting its RGB value to `(72, 60, 50)`.
    taupe, 0x483C32;
    /// Creates a new `Colour`, setting its RGB value to `(139, 133, 137)`.
    taupe_gray, 0x8B8589;
    /// Creates a new `Colour`, setting its RGB value to `(208, 240, 192)`.
    tea_green, 0xD0F0C0;
    /// Creates a new `Colour`, setting its RGB value to `(244, 194, 194)`.
    tea_rose, 0xF4C2C2;
    /// Creates a new `Colour`, setting its RGB value to `(26, 188, 156)`.
    teal, 0x1ABC9C;
    /// Creates a new `Colour`, setting its RGB value to `(54, 117, 136)`.
    teal_blue, 0x367588;
    /// Creates a new `Colour`, setting its RGB value to `(153, 230, 179)`.
    teal_deer, 0x99E6B3;
    /// Creates a new `Colour`, setting its RGB value to `(0, 130, 127)`.
    teal_green, 0x00827F;
    /// Creates a new `Colour`, setting its RGB value to `(207, 52, 118)`.
    telemagenta, 0xCF3476;
    /// Creates a new `Colour`, setting its RGB value to `(205, 87, 0)`.
    tenn_tawny, 0xCD5700;
    /// Creates a new `Colour`, setting its RGB value to `(226, 114, 91)`.
    terra_cotta, 0xE2725B;
    /// Creates a new `Colour`, setting its RGB value to `(216, 191, 216)`.
    thistle, 0xD8BFD8;
    /// Creates a new `Colour`, setting its RGB value to `(222, 111, 161)`.
    thulian_pink, 0xDE6FA1;
    /// Creates a new `Colour`, setting its RGB value to `(252, 137, 172)`.
    tickle_me_pink, 0xFC89AC;
    /// Creates a new `Colour`, setting its RGB value to `(10, 186, 181)`.
    tiffany_blue, 0x0ABAB5;
    /// Creates a new `Colour`, setting its RGB value to `(224, 141, 60)`.
    tigers_eye, 0xE08D3C;
    /// Creates a new `Colour`, setting its RGB value to `(219, 215, 210)`.
    timberwolf, 0xDBD7D2;
    /// Creates a new `Colour`, setting its RGB value to `(238, 230, 0)`.
    titanium_yellow, 0xEEE600;
    /// Creates a new `Colour`, setting its RGB value to `(255, 99, 71)`.
    tomato, 0xFF6347;
    /// Creates a new `Colour`, setting its RGB value to `(116, 108, 192)`.
    toolbox, 0x746CC0;
    /// Creates a new `Colour`, setting its RGB value to `(255, 200, 124)`.
    topaz, 0xFFC87C;
    /// Creates a new `Colour`, setting its RGB value to `(253, 14, 53)`.
    tractor_red, 0xFD0E35;
    /// Creates a new `Colour`, setting its RGB value to `(128, 128, 128)`.
    trolley_grey, 0x808080;
    /// Creates a new `Colour`, setting its RGB value to `(0, 117, 94)`.
    tropical_rain_forest, 0x00755E;
    /// Creates a new `Colour`, setting its RGB value to `(205, 164, 222)`.
    tropical_violet, 0xCDA4DE;
    /// Creates a new `Colour`, setting its RGB value to `(0, 115, 207)`.
    true_blue, 0x0073CF;
    /// Creates a new `Colour`, setting its RGB value to `(62, 142, 222)`.
    tufts_blue, 0x3E8EDE;
    /// Creates a new `Colour`, setting its RGB value to `(255, 135, 141)`.
    tulip, 0xFF878D;
    /// Creates a new `Colour`, setting its RGB value to `(222, 170, 136)`.
    tumbleweed, 0xDEAA88;
    /// Creates a new `Colour`, setting its RGB value to `(181, 114, 129)`.
    turkish_rose, 0xB57281;
    /// Creates a new `Colour`, setting its RGB value to `(64, 224, 208)`.
    turquoise, 0x40E0D0;
    /// Creates a new `Colour`, setting its RGB value to `(0, 255, 239)`.
    turquoise_blue, 0x00FFEF;
    /// Creates a new `Colour`, setting its RGB value to `(160, 214, 180)`.
    turquoise_green, 0xA0D6B4;
    /// Creates a new `Colour`, setting its RGB value to `(0, 197, 205)`.
    turquoise_surf, 0x00C5CD;
    /// Creates a new `Colour`, setting its RGB value to `(138, 154, 91)`.
    turtle_green, 0x8A9A5B;
    /// Creates a new `Colour`, setting its RGB value to `(250, 214, 165)`.
    tuscan, 0xFAD6A5;
    /// Creates a new `Colour`, setting its RGB value to `(111, 78, 55)`.
    tuscan_brown, 0x6F4E37;
    /// Creates a new `Colour`, setting its RGB value to `(124, 72, 72)`.
    tuscan_red, 0x7C4848;
    /// Creates a new `Colour`, setting its RGB value to `(166, 123, 91)`.
    tuscan_tan, 0xA67B5B;
    /// Creates a new `Colour`, setting its RGB value to `(192, 153, 153)`.
    tuscany, 0xC09999;
    /// Creates a new `Colour`, setting its RGB value to `(138, 73, 107)`.
    twilight_lavender, 0x8A496B;
    /// Creates a new `Colour`, setting its RGB value to `(102, 2, 60)`.
    tyrian_purple, 0x66023C;
    /// Creates a new `Colour`, setting its RGB value to `(0, 51, 170)`.
    ua_blue, 0x0033AA;
    /// Creates a new `Colour`, setting its RGB value to `(217, 0, 76)`.
    ua_red, 0xD9004C;
    /// Creates a new `Colour`, setting its RGB value to `(136, 120, 195)`.
    ube, 0x8878C3;
    /// Creates a new `Colour`, setting its RGB value to `(83, 104, 149)`.
    ucla_blue, 0x536895;
    /// Creates a new `Colour`, setting its RGB value to `(255, 179, 0)`.
    ucla_gold, 0xFFB300;
    /// Creates a new `Colour`, setting its RGB value to `(60, 208, 112)`.
    ufo_green, 0x3CD070;
    /// Creates a new `Colour`, setting its RGB value to `(255, 111, 255)`.
    ultra_pink, 0xFF6FFF;
    /// Creates a new `Colour`, setting its RGB value to `(252, 108, 133)`.
    ultra_red, 0xFC6C85;
    /// Creates a new `Colour`, setting its RGB value to `(63, 0, 255)`.
    ultramarine, 0x3F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(65, 102, 245)`.
    ultramarine_blue, 0x4166F5;
    /// Creates a new `Colour`, setting its RGB value to `(99, 81, 71)`.
    umber, 0x635147;
    /// Creates a new `Colour`, setting its RGB value to `(255, 221, 202)`.
    unbleached_silk, 0xFFDDCA;
    /// Creates a new `Colour`, setting its RGB value to `(91, 146, 229)`.
    united_nations_blue, 0x5B92E5;
    /// Creates a new `Colour`, setting its RGB value to `(183, 135, 39)`.
    university_of_california_gold, 0xB78727;
    /// Creates a new `Colour`, setting its RGB value to `(247, 127, 0)`.
    university_of_tennessee_orange, 0xF77F00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 102)`.
    unmellow_yellow, 0xFFFF66;
    /// Creates a new `Colour`, setting its RGB value to `(1, 68, 33)`.
    up_forest_green, 0x014421;
    /// Creates a new `Colour`, setting its RGB value to `(123, 17, 19)`.
    up_maroon, 0x7B1113;
    /// Creates a new `Colour`, setting its RGB value to `(174, 32, 41)`.
    upsdell_red, 0xAE2029;
    /// Creates a new `Colour`, setting its RGB value to `(225, 173, 33)`.
    urobilin, 0xE1AD21;
    /// Creates a new `Colour`, setting its RGB value to `(0, 79, 152)`.
    usafa_blue, 0x004F98;
    /// Creates a new `Colour`, setting its RGB value to `(153, 0, 0)`.
    usc_cardinal, 0x990000;
    /// Creates a new `Colour`, setting its RGB value to `(255, 204, 0)`.
    usc_gold, 0xFFCC00;
    /// Creates a new `Colour`, setting its RGB value to `(211, 0, 63)`.
    utah_crimson, 0xD3003F;
    /// Creates a new `Colour`, setting its RGB value to `(102, 66, 40)`.
    van_dyke_brown, 0x664228;
    /// Creates a new `Colour`, setting its RGB value to `(243, 229, 171)`.
    vanilla, 0xF3E5AB;
    /// Creates a new `Colour`, setting its RGB value to `(243, 143, 169)`.
    vanilla_ice, 0xF38FA9;
    /// Creates a new `Colour`, setting its RGB value to `(197, 179, 88)`.
    vegas_gold, 0xC5B358;
    /// Creates a new `Colour`, setting its RGB value to `(200, 8, 21)`.
    venetian_red, 0xC80815;
    /// Creates a new `Colour`, setting its RGB value to `(67, 179, 174)`.
    verdigris, 0x43B3AE;
    /// Creates a new `Colour`, setting its RGB value to `(217, 56, 30)`.
    vermilion, 0xD9381E;
    /// Creates a new `Colour`, setting its RGB value to `(160, 32, 240)`.
    veronica, 0xA020F0;
    /// Creates a new `Colour`, setting its RGB value to `(116, 187, 251)`.
    very_light_azure, 0x74BBFB;
    /// Creates a new `Colour`, setting its RGB value to `(102, 102, 255)`.
    very_light_blue, 0x6666FF;
    /// Creates a new `Colour`, setting its RGB value to `(100, 233, 134)`.
    very_light_malachite_green, 0x64E986;
    /// Creates a new `Colour`, setting its RGB value to `(255, 176, 119)`.
    very_light_tangelo, 0xFFB077;
    /// Creates a new `Colour`, setting its RGB value to `(255, 223, 191)`.
    very_pale_orange, 0xFFDFBF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 191)`.
    very_pale_yellow, 0xFFFFBF;
    /// Creates a new `Colour`, setting its RGB value to `(143, 0, 255)`.
    violet, 0x8F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(50, 74, 178)`.
    violet_blue, 0x324AB2;
    /// Creates a new `Colour`, setting its RGB value to `(127, 0, 255)`.
    violet_color_wheel, 0x7F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(247, 83, 148)`.
    violet_red, 0xF75394;
    /// Creates a new `Colour`, setting its RGB value to `(134, 1, 175)`.
    violet_ryb, 0x8601AF;
    /// Creates a new `Colour`, setting its RGB value to `(238, 130, 238)`.
    violet_web, 0xEE82EE;
    /// Creates a new `Colour`, setting its RGB value to `(64, 130, 109)`.
    viridian, 0x40826D;
    /// Creates a new `Colour`, setting its RGB value to `(0, 150, 152)`.
    viridian_green, 0x009698;
    /// Creates a new `Colour`, setting its RGB value to `(124, 158, 217)`.
    vista_blue, 0x7C9ED9;
    /// Creates a new `Colour`, setting its RGB value to `(204, 153, 0)`.
    vivid_amber, 0xCC9900;
    /// Creates a new `Colour`, setting its RGB value to `(146, 39, 36)`.
    vivid_auburn, 0x922724;
    /// Creates a new `Colour`, setting its RGB value to `(159, 29, 53)`.
    vivid_burgundy, 0x9F1D35;
    /// Creates a new `Colour`, setting its RGB value to `(218, 29, 129)`.
    vivid_cerise, 0xDA1D81;
    /// Creates a new `Colour`, setting its RGB value to `(0, 170, 238)`.
    vivid_cerulean, 0x00AAEE;
    /// Creates a new `Colour`, setting its RGB value to `(204, 0, 51)`.
    vivid_crimson, 0xCC0033;
    /// Creates a new `Colour`, setting its RGB value to `(255, 153, 0)`.
    vivid_gamboge, 0xFF9900;
    /// Creates a new `Colour`, setting its RGB value to `(166, 214, 8)`.
    vivid_lime_green, 0xA6D608;
    /// Creates a new `Colour`, setting its RGB value to `(0, 204, 51)`.
    vivid_malachite, 0x00CC33;
    /// Creates a new `Colour`, setting its RGB value to `(184, 12, 227)`.
    vivid_mulberry, 0xB80CE3;
    /// Creates a new `Colour`, setting its RGB value to `(255, 95, 0)`.
    vivid_orange, 0xFF5F00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 160, 0)`.
    vivid_orange_peel, 0xFFA000;
    /// Creates a new `Colour`, setting its RGB value to `(204, 0, 255)`.
    vivid_orchid, 0xCC00FF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 108)`.
    vivid_raspberry, 0xFF006C;
    /// Creates a new `Colour`, setting its RGB value to `(247, 13, 26)`.
    vivid_red, 0xF70D1A;
    /// Creates a new `Colour`, setting its RGB value to `(223, 97, 36)`.
    vivid_red_tangelo, 0xDF6124;
    /// Creates a new `Colour`, setting its RGB value to `(0, 204, 255)`.
    vivid_sky_blue, 0x00CCFF;
    /// Creates a new `Colour`, setting its RGB value to `(240, 116, 39)`.
    vivid_tangelo, 0xF07427;
    /// Creates a new `Colour`, setting its RGB value to `(255, 160, 137)`.
    vivid_tangerine, 0xFFA089;
    /// Creates a new `Colour`, setting its RGB value to `(229, 96, 36)`.
    vivid_vermilion, 0xE56024;
    /// Creates a new `Colour`, setting its RGB value to `(159, 0, 255)`.
    vivid_violet, 0x9F00FF;
    /// Creates a new `Colour`, setting its RGB value to `(255, 227, 2)`.
    vivid_yellow, 0xFFE302;
    /// Creates a new `Colour`, setting its RGB value to `(206, 255, 0)`.
    volt, 0xCEFF00;
    /// Creates a new `Colour`, setting its RGB value to `(52, 178, 51)`.
    wageningen_green, 0x34B233;
    /// Creates a new `Colour`, setting its RGB value to `(0, 66, 66)`.
    warm_black, 0x004242;
    /// Creates a new `Colour`, setting its RGB value to `(164, 244, 249)`.
    waterspout, 0xA4F4F9;
    /// Creates a new `Colour`, setting its RGB value to `(124, 152, 171)`.
    weldon_blue, 0x7C98AB;
    /// Creates a new `Colour`, setting its RGB value to `(100, 84, 82)`.
    wenge, 0x645452;
    /// Creates a new `Colour`, setting its RGB value to `(245, 222, 179)`.
    wheat, 0xF5DEB3;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 255)`.
    white, 0xFFFFFF;
    /// Creates a new `Colour`, setting its RGB value to `(245, 245, 245)`.
    white_smoke, 0xF5F5F5;
    /// Creates a new `Colour`, setting its RGB value to `(162, 173, 208)`.
    wild_blue_yonder, 0xA2ADD0;
    /// Creates a new `Colour`, setting its RGB value to `(212, 112, 162)`.
    wild_orchid, 0xD470A2;
    /// Creates a new `Colour`, setting its RGB value to `(255, 67, 164)`.
    wild_strawberry, 0xFF43A4;
    /// Creates a new `Colour`, setting its RGB value to `(252, 108, 133)`.
    wild_watermelon, 0xFC6C85;
    /// Creates a new `Colour`, setting its RGB value to `(253, 88, 0)`.
    willpower_orange, 0xFD5800;
    /// Creates a new `Colour`, setting its RGB value to `(167, 85, 2)`.
    windsor_tan, 0xA75502;
    /// Creates a new `Colour`, setting its RGB value to `(114, 47, 55)`.
    wine, 0x722F37;
    /// Creates a new `Colour`, setting its RGB value to `(103, 49, 71)`.
    wine_dregs, 0x673147;
    /// Creates a new `Colour`, setting its RGB value to `(255, 0, 124)`.
    winter_sky, 0xFF007C;
    /// Creates a new `Colour`, setting its RGB value to `(160, 230, 255)`.
    winter_wizard, 0xA0E6FF;
    /// Creates a new `Colour`, setting its RGB value to `(86, 136, 125)`.
    wintergreen_dream, 0x56887D;
    /// Creates a new `Colour`, setting its RGB value to `(201, 160, 220)`.
    wisteria, 0xC9A0DC;
    /// Creates a new `Colour`, setting its RGB value to `(193, 154, 107)`.
    wood_brown, 0xC19A6B;
    /// Creates a new `Colour`, setting its RGB value to `(115, 134, 120)`.
    xanadu, 0x738678;
    /// Creates a new `Colour`, setting its RGB value to `(15, 77, 146)`.
    yale_blue, 0x0F4D92;
    /// Creates a new `Colour`, setting its RGB value to `(28, 40, 65)`.
    yankees_blue, 0x1C2841;
    /// Creates a new `Colour`, setting its RGB value to `(255, 255, 0)`.
    yellow, 0xFFFF00;
    /// Creates a new `Colour`, setting its RGB value to `(252, 232, 131)`.
    yellow_crayola, 0xFCE883;
    /// Creates a new `Colour`, setting its RGB value to `(154, 205, 50)`.
    yellow_green, 0x9ACD32;
    /// Creates a new `Colour`, setting its RGB value to `(239, 204, 0)`.
    yellow_munsell, 0xEFCC00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 211, 0)`.
    yellow_ncs, 0xFFD300;
    /// Creates a new `Colour`, setting its RGB value to `(255, 174, 66)`.
    yellow_orange, 0xFFAE42;
    /// Creates a new `Colour`, setting its RGB value to `(254, 223, 0)`.
    yellow_pantone, 0xFEDF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 239, 0)`.
    yellow_process, 0xFFEF00;
    /// Creates a new `Colour`, setting its RGB value to `(255, 240, 0)`.
    yellow_rose, 0xFFF000;
    /// Creates a new `Colour`, setting its RGB value to `(254, 254, 51)`.
    yellow_ryb, 0xFEFE33;
    /// Creates a new `Colour`, setting its RGB value to `(255, 247, 0)`.
    yellow_sunshine, 0xFFF700;
    /// Creates a new `Colour`, setting its RGB value to `(0, 20, 168)`.
    zaffre, 0x0014A8;
    /// Creates a new `Colour`, setting its RGB value to `(44, 22, 8)`.
    zinnwaldite_brown, 0x2C1608;
    /// Creates a new `Colour`, setting its RGB value to `(57, 167, 142)`.
    zomp, 0x39A78E;
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
