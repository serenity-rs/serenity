use std::str::FromStr;
use std::error::Error as StdError;
use std::fmt;

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

fn second_quote_occurence(s: &str) -> Option<usize> {
    s.chars().enumerate().filter(|&(_, c)| c == '"').nth(1).map(|(pos, _)| pos)
}

fn parse_quotes<T: FromStr>(s: &mut String, delimiter: &str) -> Result<T, T::Err> 
    where T::Err: StdError {

    // Fall back to `parse` if there're no quotes at the start.
    if s.chars().next().unwrap() != '"' {
        return parse::<T>(s, delimiter);
    }

    let mut pos = second_quote_occurence(&s).unwrap_or(s.len());
    let res = (&s[1..pos]).parse::<T>().map_err(Error::Parse);
    // +1 is for the quote
    if pos < s.len() {
        pos += 1;
    }
    
    s.drain(..pos);

    res
}

fn parse<T: FromStr>(s: &mut String, delimiter: &str) -> Result<T, T::Err> 
    where T::Err: StdError {
    let mut pos = s.find(delimiter).unwrap_or(s.len());

    let res = (&s[..pos]).parse::<T>().map_err(Error::Parse);
    // +1 is for the delimiter
    if pos < s.len() {
        pos += 1;
    }
    
    s.drain(..pos);
    res
}

/// A utility struct for handling arguments of a command.
///
/// General functionality is done via removing an item, parsing it, then returning it; this however
/// can be mitigated with the `*_n` methods, which just parse and return.
#[derive(Clone, Debug)]
pub struct Args {
    delimiter: String,
    message: String,
}

impl Args {
    pub fn new(message: &str, possible_delimiters: Vec<String>) -> Self {
        let delimiter = possible_delimiters
            .iter()
            .find(|&d| message.contains(d))
            .map_or(possible_delimiters[0].as_str(), |s| s.as_str());

        Args {
            delimiter: delimiter.to_string(),
            message: message.to_string(),
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
    /// assert_eq!(args, "69");
    /// ```
    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.message.is_empty() {
            return Err(Error::Eos);
        }

        parse::<T>(&mut self.message, &self.delimiter)
    }

    /// Like [`single`], but does "zero-copy" parsing.
    ///
    /// Refer to [`FromStrZc`]'s example on how to use this method.
    ///
    /// [`single`]: #method.single
    /// [`FromStrZc`]: trait.FromStrZc.html
    pub fn single_zc<'a, T: FromStrZc<'a> + 'a>(&'a mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.message.is_empty() {
            return Err(Error::Eos);
        }

        let pos = self.message.find(&self.delimiter).unwrap_or(self.message.len());

        fn parse_then_remove(msg: &mut String, pos: usize) -> &str {
            struct ParseThenRemove<'a>(&'a mut String, usize);

            impl<'a> Drop for ParseThenRemove<'a> {
                fn drop(&mut self) {
                    if !self.0.is_empty() {
                        self.0.drain(..self.1 + 1);
                    }
                }
            }

            (ParseThenRemove(msg, pos).0).as_str()
        }

        let string = parse_then_remove(&mut self.message, pos);
        FromStrZc::from_str(&string[..pos]).map_err(Error::Parse)
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
    /// assert_eq!(args, "42 69");
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.message.is_empty() {
            return Err(Error::Eos);
        }

        parse::<T>(&mut self.message.clone(), &self.delimiter)
    }

    /// Accesses the current state of the internal string.
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
    pub fn full(&self) -> &str { &self.message }

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
    /// assert_eq!(args, "69");
    /// ```
    pub fn skip(&mut self) -> Option<String> { parse::<String>(&mut self.message, &self.delimiter).ok() }

    /// Like [`skip`], but allows for multiple at once.
    ///
    /// # Examples
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69 88 99", vec![" ".to_owned()]);
    ///
    /// assert_eq!(*args.skip_for(3).unwrap(), ["42".to_string(), "69".to_string(), "88".to_string()]);
    /// assert_eq!(args, "99");
    /// ```
    ///
    /// [`skip`]: #method.skip
    pub fn skip_for(&mut self, i: u32) -> Option<Vec<String>> {
        let mut vec = Vec::with_capacity(i as usize);

        for _ in 0..i {
            vec.push(self.skip()?);
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
    /// let mut args = Args::new(r#""42 69"#, vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.single_quoted::<String>().unwrap(), "42 69");
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        parse_quotes::<T>(&mut self.message, &self.delimiter)
    }

    /// Like [`single_quoted`], but doesn't remove the element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42 69"#, vec![" ".to_owned()]);
    ///
    /// assert_eq!(args.single_quoted_n::<String>().unwrap(), "42 69");
    /// assert_eq!(args, r#""42 69"#);
    /// ```
    ///
    /// [`single_quoted`]: #method.single_quoted
    pub fn single_quoted_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        parse_quotes::<T>(&mut self.message.clone(), &self.delimiter)
    }

    // Fix this.

    /// Like [`multiple`], but takes quotes into account.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42" "69""#, vec![" ".to_owned()]);
    ///
    /// assert_eq!(*args.multiple_quoted::<i32>().unwrap(), [42, 69]);
    /// ```
    ///
    /// [`multiple`]: #method.multiple
    pub fn multiple_quoted<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        let mut res = Vec::new();

        let count = self.message.chars().filter(|&c| c == '"').count() / 2;

        for _ in 0..count {
            res.push(parse_quotes::<T>(&mut self.message, &self.delimiter)?);
        }

        Ok(res)
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
    /// assert_eq!(args, "c47");
    /// ```
    pub fn find<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.message.is_empty() {
            return Err(Error::Eos);
        }

        // TODO: Make this efficient
        
        let pos = self.message
            .split(&self.delimiter)
            .position(|e| e.parse::<T>().is_ok());

        match pos {
            Some(index) => {
                let mut vec = self.message.split(&self.delimiter).map(|s| s.to_string()).collect::<Vec<_>>();
                let mut ss = vec.remove(index);
                let res = parse::<T>(&mut ss, &self.delimiter);
                self.message = vec.join(&self.delimiter);
                res
            },
            None => Err(Error::Eos),
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
    /// assert_eq!(args, "c47 69");
    /// ```
    pub fn find_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.message.is_empty() {
            return Err(Error::Eos);
        }

        // Same here.
        let pos = self.message
            .split(&self.delimiter)
            .position(|e| e.parse::<T>().is_ok());

        match pos {
            Some(index) => {
                let mut vec = self.message.split(&self.delimiter).map(|s| s.to_string()).collect::<Vec<_>>();
                let mut ss = vec.remove(index);
                parse::<T>(&mut ss, &self.delimiter)
            },
            None => Err(Error::Eos),
        }
    }
}

// Fix this.

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
    type Target = str;

    fn deref(&self) -> &Self::Target { self.full() }
}

impl PartialEq<str> for Args {
    fn eq(&self, other: &str) -> bool {
        self.message == other
    }
}

impl<'a> PartialEq<&'a str> for Args {
    fn eq(&self, other: &&'a str) -> bool {
        self.message == *other
    }
}


impl PartialEq for Args {
    fn eq(&self, other: &Self) -> bool {
        self.message == *other.message
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
