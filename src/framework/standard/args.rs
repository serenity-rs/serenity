use std::str::FromStr;
use std::error::Error as StdError;
use std::fmt;
use utils::parse_quotes;
use vec_shift::Shift;

/// Defines how an operation on an `Args` method failed.
#[derive(Debug)]
pub enum Error<E: StdError> {
    /// "END-OF-STRING", more precisely, there isn't anything to parse anymore.
    Eos,
    /// A parsing operation failed; the error in it can be of any returned from the `FromStr`
    /// trait.
    Parse(E),
}

impl<E: StdError> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Parse(e)
    }
}

impl<E: StdError> StdError for Error<E> {
    fn description(&self) -> &str {
        use self::Error::*;

        match *self {
            Eos => "end-of-string",
            Parse(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        use self::Error::*;

        match *self {
            Parse(ref e) => Some(e),
            _ => None,
        }
    }
}

impl<E: StdError> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match *self {
            Eos => write!(f, "end of string"),
            Parse(ref e) => fmt::Display::fmt(&e, f),
        }
    }
}

type Result<T, E> = ::std::result::Result<T, Error<E>>;

/// A utility struct for handling arguments of a command.
///
/// General functionality is done via removing an item, parsing it, then returning it; this however
/// can be mitigated with the `*_n` methods, which just parse and return.
#[derive(Clone, Debug)]
pub struct Args {
    delimiter: String,
    delimiter_split: Vec<String>,
}

impl Args {
    pub fn new(message: &str, possible_delimiters: Vec<String>) -> Self {
        let delimiter = possible_delimiters
            .iter()
            .find(|&d| message.contains(d))
            .map_or(possible_delimiters[0].as_str(), |s| s.as_str());

        let split = if message.trim().is_empty() {
            Vec::new()
        } else {
            message.split(delimiter).map(|s| s.to_string()).collect()
        };

        Args {
            delimiter: delimiter.to_string(),
            delimiter_split: split,
        }
    }

    /// Removes the first element, parses it to a specific type if necessary, returns.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.single::<i32>().unwrap(), 42);
    /// assert_eq!(args, ["69"]);
    /// ```
    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        Ok(self.delimiter_split
            .shift()
            .ok_or(Error::Eos)?
            .parse::<T>()?)
    }

    /// Like [`single`], but does "zero-copy" parsing.
    ///
    /// Refer to [`FromStrZc`]'s example on how to use this method.
    ///
    /// [`single`]: #method.single
    /// [`FromStrZc`]: trait.FromStrZc.html
    pub fn single_zc<'a, T: FromStrZc<'a> + 'a>(&'a mut self) -> Result<T, T::Err>
        where T::Err: StdError {

        // This is a hack as to mitigate some nasty lifetime errors.
        //
        // (Culprit `Vec::remove`s return type)
        fn get_and_remove(b: &mut Vec<String>) -> Option<&str> {
            struct GetThenRemove<'a>(&'a mut Vec<String>);

            impl<'a> Drop for GetThenRemove<'a> {
                fn drop(&mut self) {
                    if !self.0.is_empty() {
                        self.0.remove(0);
                    }
                }
            }

            GetThenRemove(b).0.get(0).map(|s| s.as_str())
        }

        let a = get_and_remove(&mut self.delimiter_split).ok_or(Error::Eos)?;
        Ok(FromStrZc::from_str(a)?)
    }

    /// Like [`single`], but doesn't remove the element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("42 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.single_n::<i32>().unwrap(), 42);
    /// assert_eq!(args, ["42", "69"]);
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        Ok(self.delimiter_split
            .get(0)
            .ok_or(Error::Eos)?
            .parse::<T>()?)
    }

    /// Skips if there's a first element, but also returns it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.skip().unwrap(), "42");
    /// assert_eq!(args, ["69"]);
    /// ```
    pub fn skip(&mut self) -> Option<String> { self.delimiter_split.shift() }

    /// Like [`skip`], but allows for multiple at once.
    ///
    /// # Examples
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69 88 99", vec![" ".to_owned()]);
    ///
    /// assert_eq!(*args.skip_for(3).unwrap(), ["42".to_string(), "69".to_string(), "88".to_string()]);
    /// assert_eq!(args, ["99"]);
    /// ```
    ///
    /// [`skip`]: #method.skip
    pub fn skip_for(&mut self, i: u32) -> Option<Vec<String>> {
        let mut vec = Vec::with_capacity(i as usize);

        for _ in 0..i {
            vec.push(try_opt!(self.delimiter_split.shift()));
        }

        Some(vec)
    }

    /// Like [`single`], but takes quotes into account.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42"#, vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.single_quoted::<i32>().unwrap(), 42);
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        Ok(parse_quotes(&self.delimiter_split.shift().ok_or(Error::Eos)?)
            .remove(0)
            .parse::<T>()?)
    }

    /// Like [`single_quoted`], but doesn't remove the element.
    ///
    /// [`single_quoted`]: #method.single_quoted
    pub fn single_quoted_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        Ok(parse_quotes(self.delimiter_split.get(0).ok_or(Error::Eos)?)
            .remove(0)
            .parse::<T>()?)
    }

    /// Like [`list`], but takes quotes into account.
    ///
    /// [`list`]: #method.list
    pub fn multiple_quoted<T: FromStr>(self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        parse_quotes(&self.delimiter_split.join(&self.delimiter))
            .into_iter()
            .map(|s| s.parse::<T>().map_err(Error::Parse))
            .collect()
    }

    /// Empty outs the internal vector while parsing (if necessary) and returning them
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(*args.multiple::<i32>().unwrap(), [42, 69]);
    /// ```
    pub fn multiple<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        Iter::<T>::new(&mut self).collect()
    }

    /// Provides an iterator of items: (`T: FromStr`) `Result<T, T::Err>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("3 4", vec![" ".to_owned()]);
    ///
    /// assert_eq!(*args.iter::<i32>().map(|num| num.unwrap().pow(2)).collect::<Vec<_>>(), [9, 16]);
    /// assert!(args.is_empty());
    /// ```
    pub fn iter<T: FromStr>(&mut self) -> Iter<T> where T::Err: StdError  {
        Iter::new(self)
    }

    /// This method is just `internal_vector.join(delimiter)`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.full(), "42 69");
    /// ```
    pub fn full(&self) -> String { self.delimiter_split.join(&self.delimiter) }

    /// Returns the first argument that can be converted and removes it from the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("c47 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.find::<i32>().unwrap(), 69);
    /// assert_eq!(args, ["c47"]);
    /// ```
    pub fn find<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        match self.delimiter_split
                  .iter()
                  .position(|e| e.parse::<T>().is_ok())
        {
            Some(index) => {
                let value = self.delimiter_split
                    .get(index)
                    .ok_or(Error::Eos)?
                    .parse::<T>()?;

                self.delimiter_split.remove(index);

                Ok(value)
            },
            _ => Err(Error::Eos),
        }
    }

    /// Returns the first argument that can be converted and does not remove it from the list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("c47 69", vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.find_n::<i32>().unwrap(), 69);
    /// assert_eq!(args, ["c47", "69"]);
    /// ```
    pub fn find_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        Ok(self.delimiter_split
            .iter()
            .find(|e| e.parse::<T>().is_ok())
            .ok_or(Error::Eos)?
            .parse::<T>()?)
    }
}

/// A version of `FromStr` that allows for "zero-copy" parsing.
///
/// # Examples
///
/// ```rust,ignore
/// use serenity::framework::standard::{Args, FromStrZc};
/// use std::fmt;
///
/// struct NameDiscrim<'a>(&'a str, Option<&'a str>);
///
/// #[derive(Debug)]
/// struct Error(&'static str);
///
/// impl std::error::Error for Error {
///     fn description(&self) -> &str { self.0 }
/// }
///
/// impl fmt::Display for Error {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
/// }
///
/// impl<'a> FromStrZc<'a> for NameDiscrim<'a> {
///     type Err = Error;
///
///     fn from_str(s: &'a str) -> Result<NameDiscrim<'a>, Error> {
///         let mut it = s.split("#");
///         let name = it.next().ok_or(Error("name must be specified"))?;
///         let discrim = it.next();
///         Ok(NameDiscrim(name, discrim))
///     }
/// }
///
/// let mut args = Args::new("abc#1234", vec![" ".to_owned()]);
/// let NameDiscrim(name, discrim) = args.single_zc::<NameDiscrim>().unwrap();
///
/// assert_eq!(name, "abc");
/// assert_eq!(discrim, Some("1234"));
/// ```
pub trait FromStrZc<'a>: Sized {
    type Err;

    fn from_str(s: &'a str) -> ::std::result::Result<Self, Self::Err>;
}

impl<'a, T: FromStr> FromStrZc<'a> for T {
    type Err = T::Err;

    fn from_str(s: &'a str) -> ::std::result::Result<Self, Self::Err> {
        <T as FromStr>::from_str(s)
    }
}

impl ::std::ops::Deref for Args {
    type Target = [String];

    fn deref(&self) -> &Self::Target { &self.delimiter_split }
}

impl<'a> PartialEq<[&'a str]> for Args {
    fn eq(&self, other: &[&str]) -> bool {
        let mut b = true;

        for (s, o) in self.delimiter_split.iter().zip(other.iter()) {
            if s != o {
                b = false;
                break;
            }
        }

        b
    }
}

macro_rules! impl_slices {
    ($($num:expr),*) => {
        impl<'a> PartialEq<[&'a str; 0]> for Args {
            fn eq(&self, _: &[&str; 0]) -> bool {
                self.delimiter_split.is_empty()
            }
        }

        $(
            impl<'a> PartialEq<[&'a str; $num]> for Args {
                fn eq(&self, other: &[&str; $num]) -> bool {
                    <Args as PartialEq<[&str]>>::eq(self, &other[..])
                }
            }
        )*
    }
}

impl_slices! {
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31, 32
}

impl PartialEq for Args {
    fn eq(&self, other: &Self) -> bool {
        self.delimiter_split == other.delimiter_split
    }
}

impl Eq for Args {}

use std::marker::PhantomData;

/// Provides `list`'s functionality, but as an iterator.
pub struct Iter<'a, T: FromStr> where T::Err: StdError {
    args: &'a mut Args,
    _marker: PhantomData<T>,
}

impl<'a, T: FromStr> Iter<'a, T> where T::Err: StdError {
    fn new(args: &'a mut Args) -> Self {
        Iter { args, _marker: PhantomData }
    }
}

impl<'a, T: FromStr> Iterator for Iter<'a, T> where T::Err: StdError  {
    type Item = Result<T, T::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.single::<T>())
        }
    }
}
