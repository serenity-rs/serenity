use std::{
    str::FromStr,
    error::Error as StdError,
    fmt
};

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
    s.match_indices('"').nth(1).map(|(pos, _)| pos)
}

fn parse_quotes<T: FromStr>(s: &mut String, delimiters: &[String]) -> Result<T, T::Err>
    where T::Err: StdError {
    
    if s.is_empty() {
        return Err(Error::Eos);
    }

    // Fall back to `parse` if there're no quotes at the start
    // or if there is no closing one as well.
    if let Some(mut pos) = second_quote_occurence(s) {
        if s.starts_with('"') {
            let res = (&s[1..pos]).parse::<T>().map_err(Error::Parse)?;
            pos += '"'.len_utf8();

            for delimiter in delimiters {
                if s[pos..].starts_with(delimiter) {
                    pos += delimiter.len();
                    break;
                }
            }

            s.drain(..pos);

            return Ok(res);
        }
    }

    parse::<T>(s, delimiters)
}


fn parse<T: FromStr>(s: &mut String, delimiters: &[String]) -> Result<T, T::Err>
    where T::Err: StdError {
    if s.is_empty() {
        return Err(Error::Eos);
    }

    let (mut smallest_pos, delimiter_len) = delimiters.iter().fold((s.len(), 0usize), |mut acc, delim| {
        let other_pos = s.find(delim).unwrap_or_else(|| s.len());

        if acc.0 > other_pos {
            acc.0 = other_pos;
            acc.1 = delim.len();
        }

        acc
    });

    let res = (&s[..smallest_pos]).parse::<T>().map_err(Error::Parse)?;

    if smallest_pos < s.len() {
        smallest_pos += delimiter_len;
    }

    s.drain(..smallest_pos);

    Ok(res)
}

/// A utility struct for handling arguments of a command.
///
/// An "argument" is a part of the message up until the end of the message or at one of the specified delimiters.
/// For instance, with a space delimiter (" ") in a message like "ab cd", we would get the argument "ab", and then "cd".
/// 
/// For the most part, almost all methods provided by this struct not only make arguments convenient to handle,
/// they'll also parse your argument to a specific type if you need to work with the type itself and not some shady string.
///
/// And for another part, in case you need multiple things, whether delimited or not, gobled in one argument,
/// you can utilize the `*_quoted` methods that will extract anything inside quotes for you.
/// Though they'll fall back to the original behaviour of, for example, `single`, 
/// on the occasion that the quotes are malformed (missing a starting or ending quote).  
///  
/// # Catch regarding how `Args` functions
/// 
/// Majority of the methods here internally chop of the argument (i.e you won't encounter it anymore), to advance to further arguments.
/// If you do not desire for this behaviour, consider using the suffixed `*_n` methods instead.
#[derive(Clone, Debug)]
pub struct Args {
    delimiters: Vec<String>,
    message: String,
    len: Option<usize>,
    len_quoted: Option<usize>,
}

impl Args {
    pub fn new(message: &str, possible_delimiters: &[String]) -> Self {
        Args {
            delimiters: possible_delimiters
                .iter()
                .filter(|&d| message.contains(d)).cloned().collect(),
            message: message.to_string(),
            len: None,
            len_quoted: None,
        }
    }

    /// Parses the current argument and advances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.single::<i32>().unwrap(), 42);
    /// assert_eq!(args.full(), "69");
    /// ```
    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        if let Some(ref mut val) = self.len {
            *val -= 1
        }

        parse::<T>(&mut self.message, &self.delimiters)
    }

    /// Like [`single`], but doesn't advance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_n::<i32>().unwrap(), 42);
    /// assert_eq!(args, "42 69");
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        parse::<T>(&mut self.message.clone(), &self.delimiters)
    }

    /// Accesses the current state of the internally-stored message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.full(), "42 69");
    /// ```
    pub fn full(&self) -> &str { &self.message }

    /// Accesses the current state of the internally-stored message, 
    /// removing quotes if it contains the opening and closing ones,
    /// but otherwise returns the string as is.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("\"42 69\"", &[" ".to_string()]);
    ///
    /// assert_eq!(args.full_quoted(), "42 69");
    /// ```
    /// 
    /// ```rust
    /// use serenity::framework::standard::Args;
    /// 
    /// let args = Args::new("\"42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.full_quoted(), "\"42 69");
    /// ```
    /// 
    /// ```rust
    /// use serenity::framework::standard::Args;
    /// 
    /// let args = Args::new("42 69\"", &[" ".to_string()]);
    ///
    /// assert_eq!(args.full_quoted(), "42 69\"");
    /// ```
    pub fn full_quoted(&self) -> &str {
        let s = &self.message;

        if !s.starts_with('"') {
            return s;
        }
    
        let end = s.rfind('"');
        if end.is_none() {
            return s;
        }

        let end = end.unwrap();
    
        // If it got the quote at the start, then there's no closing quote.
        if end == 0 {
            return s;
        }
    
        &s[1..end]
    }

    /// The amount of args.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.len(), 2); // `2` because `["42", "69"]`
    /// ```
    pub fn len(&mut self) -> usize {
        if let Some(len) = self.len {
            len
        } else if self.is_empty() {
            0
        } else {
            let mut words: Box<Iterator<Item = &str>> = Box::new(Some(&self.message[..]).into_iter());

            for delimiter in &self.delimiters {
                words = Box::new(words.flat_map(move |x| x.split(delimiter)));
            }

            let len = words.count();
            self.len = Some(len);
            len
        }
    }

    /// Returns true if the string is empty or else false.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("", &[" ".to_string()]);
    ///
    /// assert!(args.is_empty()); // `true` because passed message is empty.
    /// ```
    pub fn is_empty(&self) -> bool {
        self.message.is_empty()
    }

    /// Like [`len`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42" "69""#, &[" ".to_string()]);
    ///
    /// assert_eq!(args.len_quoted(), 2); // `2` because `["42", "69"]`
    /// ```
    pub fn len_quoted(&mut self) -> usize {
        if self.is_empty() {
            0
        } else if let Some(len_quoted) = self.len_quoted {
            len_quoted
        } else {
            let countable_self = self.clone();

            if let Ok(ref vec) = countable_self.multiple_quoted::<String>() {
                vec.iter().count()
            } else {
                0
            }
        }
    }

    /// Returns the argument as a string (thus sort-of skipping it).
    /// 
    /// *This is sugar for `args.single::<String>().ok()`*
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.skip().unwrap(), "42");
    /// assert_eq!(args.full(), "69");
    /// ```
    pub fn skip(&mut self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        if let Some(ref mut val) = self.len {
            if 1 <= *val {
                *val -= 1
            }
        };

        parse::<String>(&mut self.message, &self.delimiters).ok()
    }

    /// Like [`skip`], but allows for multiple at once.
    ///
    /// # Examples
    /// 
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 69 88 99", &[" ".to_string()]);
    ///
    /// assert_eq!(*args.skip_for(3).unwrap(), ["42".to_string(), "69".to_string(), "88".to_string()]);
    /// assert_eq!(args, "99");
    /// ```
    ///
    /// [`skip`]: #method.skip
    pub fn skip_for(&mut self, i: u32) -> Option<Vec<String>> {
        if self.is_empty() {
            return None;
        }

        let mut vec = Vec::with_capacity(i as usize);

        for _ in 0..i {
            vec.push(self.skip()?);
        }

        if let Some(ref mut val) = self.len {

            if i as usize <= *val {
                *val -= i as usize
            } else {
                *val = 0
            }
        }

        Some(vec)
    }

    /// Like [`single`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42 69""#, &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_quoted::<String>().unwrap(), "42 69");
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        if let Some(ref mut val) = self.len_quoted {
            *val -= 1
        }

        parse_quotes::<T>(&mut self.message, &self.delimiters)
    }

    /// Like [`single_quoted`], but doesn't advance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42 69""#, &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_quoted_n::<String>().unwrap(), "42 69");
    /// assert_eq!(args.full(), r#""42 69""#);
    /// ```
    ///
    /// [`single_quoted`]: #method.single_quoted
    pub fn single_quoted_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        parse_quotes::<T>(&mut self.message.clone(), &self.delimiters)
    }

    /// Like [`multiple`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""42" "69""#, &[" ".to_string()]);
    ///
    /// assert_eq!(*args.multiple_quoted::<i32>().unwrap(), [42, 69]);
    /// ```
    ///
    /// [`multiple`]: #method.multiple
    pub fn multiple_quoted<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }
        
        self.iter_quoted::<T>().collect()
    }

    /// Like [`iter`], but accounts quotes.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""2" "5""#, &[" ".to_string()]);
    /// 
    /// assert_eq!(*args.iter_quoted::<i32>().map(|n| n.unwrap().pow(2)).collect::<Vec<_>>(), [4, 25]);
    /// assert!(args.is_empty());
    /// ```
    /// 
    /// [`iter`]: #method.iter
    pub fn iter_quoted<T: FromStr>(&mut self) -> IterQuoted<T>
        where T::Err: StdError {
        IterQuoted::new(self)
    }

    /// This is a convenience function for parsing until the end of the message and returning the parsed results in a `Vec`.
    /// 
    /// *This is sugar for `args.iter().collect::<Vec<_>>()`*
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("42 69", &[" ".to_string()]);
    ///
    /// assert_eq!(*args.multiple::<i32>().unwrap(), [42, 69]);
    /// ```
    pub fn multiple<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        self.iter::<T>().collect()
    }

    /// Provides an arguments iterator up until the end of the message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("3 4", &[" ".to_string()]);
    ///
    /// assert_eq!(*args.iter::<i32>().map(|num| num.unwrap().pow(2)).collect::<Vec<_>>(), [9, 16]);
    /// assert!(args.is_empty());
    /// ```
    pub fn iter<T: FromStr>(&mut self) -> Iter<T> 
        where T::Err: StdError {
        Iter::new(self)
    }

    /// Returns the first argument that can be parsed and removes it from the message. The suitable argument 
    /// can be in an arbitrary position in the message.
    ///
    /// **Note**: This replaces all delimiters within the message
    /// by the first set in your framework-config to win performance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("c47 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.find::<i32>().unwrap(), 69);
    /// assert_eq!(args.full(), "c47");
    /// ```
    pub fn find<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        // TODO: Make this efficient

        if self.delimiters.len() == 1 as usize {
            match self.message.split(&self.delimiters[0]).position(|e| e.parse::<T>().is_ok()) {
                Some(index) => {
                    let mut vec = self.message.split(self.delimiters[0].as_str()).map(|s| s.to_string()).collect::<Vec<_>>();
                    let mut ss = vec.remove(index);
                    let res = parse::<T>(&mut ss, &self.delimiters);
                    self.message = vec.join(&self.delimiters[0]);
                    if let Some(ref mut val) = self.len { if 1 <= *val { *val -= 1 } };
                    res
                },
                None => Err(Error::Eos),
            }
        } else {
            let msg = self.message.clone();
            let mut words: Box<Iterator<Item = &str>> = Box::new(Some(&msg[..]).into_iter());

            for delimiter in &self.delimiters {
                words = Box::new(words.flat_map(move |x| x.split(delimiter)));
            }

            let mut words: Vec<&str> = words.collect();
            let pos = words.iter().position(|e| e.parse::<T>().is_ok());
            if let Some(ref mut val) = self.len { if 1 <= *val { *val -= 1 } };

            match pos {
                Some(index) => {
                    let ss = words.remove(index);

                    let res = parse::<T>(&mut ss.to_string(), &self.delimiters);
                    self.len = Some(words.len());
                    self.message = words.join(&self.delimiters[0]);
                    res
                },
                None => Err(Error::Eos),
            }
        }
    }

    /// Like [`find`], but does not remove it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("c47 69", &[" ".to_string()]);
    ///
    /// assert_eq!(args.find_n::<i32>().unwrap(), 69);
    /// assert_eq!(args.full(), "c47 69");
    /// ```
    /// 
    /// [`find`]: #method.find
    pub fn find_n<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        // Same here.
        if self.delimiters.len() == 1 {
            let pos = self.message
                .split(&self.delimiters[0])
                .position(|e| e.parse::<T>().is_ok());

            match pos {
                Some(index) => {
                    let mut vec = self.message.split(&self.delimiters[0]).map(|s| s.to_string()).collect::<Vec<_>>();
                    let mut ss = vec.remove(index);
                    parse::<T>(&mut ss, &self.delimiters)
                },
                None => Err(Error::Eos),
            }
        } else {
            let mut words: Box<Iterator<Item = &str>> = Box::new(Some(&self.message[..]).into_iter());
            for delimiter in &self.delimiters {
                words = Box::new(words.flat_map(move |x| x.split(delimiter)));
            }

            let pos = words.position(|e| e.parse::<T>().is_ok());
            let mut words: Vec<&str> = words.collect();

            match pos {
                Some(index) => {
                    let ss = words.remove(index);
                    self.len = Some(words.len());
                    parse::<T>(&mut ss.to_string(), &self.delimiters)
                },
                None => Err(Error::Eos),
            }
        }
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

/// Same as [`Iter`], but considers quotes.
/// 
/// [`Iter`]: #struct.Iter.html
pub struct IterQuoted<'a, T: FromStr> where T::Err: StdError {
    args: &'a mut Args,
    _marker: PhantomData<T>,
}

impl<'a, T: FromStr> IterQuoted<'a, T> where T::Err: StdError {
    fn new(args: &'a mut Args) -> Self {
        IterQuoted { args, _marker: PhantomData }
    }
}

impl<'a, T: FromStr> Iterator for IterQuoted<'a, T> where T::Err: StdError  {
    type Item = Result<T, T::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.single_quoted::<T>())
        }
    }
}
