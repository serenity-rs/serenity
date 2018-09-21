use std::{
    str::FromStr,
    error::Error as StdError,
    fmt
};

/// Defines how an operation on an `Args` method failed.
#[derive(Debug)]
pub enum Error<E: StdError> {
    /// "END-OF-STRING". There's nothing to parse anymore.
    Eos,
    /// The parsing operation failed; the error can be anything returned from the `FromStr` trait.
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

fn find_end(s: &str, i: usize) -> Option<usize> {
    if i > s.len() {
        return None;
    }

    let mut end = i + 1;

    while !s.is_char_boundary(end) {
        end += 1;
    }

    Some(end)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenKind {
    Delimiter,
    Argument,
    QuotedArgument,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    lit: String,
    // start position
    pos: usize,
}

impl Token {
    fn new(kind: TokenKind, lit: &str, pos: usize) -> Self {
        Token { kind, lit: lit.to_string(), pos }
    }
}

#[derive(Debug)]
struct Lexer<'a> {
    msg: &'a str,
    delims: &'a [char],
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(msg: &'a str, delims: &'a [char]) -> Self {
        Lexer {
            msg,
            delims,
            offset: 0,
        }
    }

    #[inline]
    fn at_end(&self) -> bool {
        self.offset >= self.msg.len()
    }

    fn current(&self) -> Option<&str> {
        if self.at_end() {
            return None;
        }

        let start = self.offset;

        let end = find_end(&self.msg, self.offset)?;

        Some(&self.msg[start..end])
    }

    fn next(&mut self) -> Option<()> {
        self.offset += self.current()?.len();

        Some(())
    }

    fn commit(&mut self) -> Option<Token> {
        if self.at_end() {
            return None;
        }

        if self.current()?.contains(self.delims) {
            let start = self.offset;
            self.next();
            return Some(Token::new(TokenKind::Delimiter, &self.msg[start..self.offset], start));
        }

        if self.current()? == "\"" {
            let start = self.offset;
            self.next();

            while !self.at_end() && self.current()? != "\"" {
                self.next();
            }

            let is_quote = self.current().map_or(false, |s| s == "\"");
            self.next();

            let end = self.offset;

            return Some(if is_quote {
                Token::new(TokenKind::QuotedArgument, &self.msg[start..end], start)
            } else {
                // We're missing an end quote. View this as a normal argument.
                Token::new(TokenKind::Argument, &self.msg[start..], start)
            });
        }

        let start = self.offset;

        while !self.at_end() {
            if self.current()?.contains(self.delims) {
                break;
            }

            self.next();
        }

        Some(Token::new(TokenKind::Argument, &self.msg[start..self.offset], start))
    }
}

/// A utility struct for handling "arguments" of a command.
///
/// An "argument" is a part of the message up that ends at one of the specified delimiters, or the end of the message.
///
/// # Example
///
/// ```rust
/// use serenity::framework::standard::Args;
///
/// let mut args = Args::new("hello world!", &[" ".to_string()]); // A space is our delimiter.
///
/// // Parse our argument as a `String` and assert that it's the "hello" part of the message.
/// assert_eq!(args.single::<String>().unwrap(), "hello");
/// // Same here.
/// assert_eq!(args.single::<String>().unwrap(), "world!");
///
/// ```
///
/// We can also parse "quoted arguments" (no pun intended):
///
/// ```rust
/// use serenity::framework::standard::Args;
///
/// // Let us imagine this scenario:
/// // You have a `photo` command that grabs the avatar url of a user. This command accepts names only.
/// // Now, one of your users wants the avatar of a user named Princess Zelda.
/// // Problem is, her name contains a space; our delimiter. This would result in two arguments, "Princess" and "Zelda".
/// // So how shall we get around this? Through quotes! By surrounding her name in them we can perceive it as one single argument.
/// let mut args = Args::new(r#""Princess Zelda""#, &[" ".to_string()]);
///
/// // Hooray!
/// assert_eq!(args.single_quoted::<String>().unwrap(), "Princess Zelda");
/// ```
///
/// In case of a mistake, we can go back in time... er I mean, one step (or entirely):
///
/// ```rust
/// use serenity::framework::standard::Args;
///
/// let mut args = Args::new("4 2", &[" ".to_string()]);
///
/// assert_eq!(args.single::<u32>().unwrap(), 4);
///
/// // Oh wait, oops, meant to double the 4.
/// // But I won't able to access it now...
/// // oh wait, I can `rewind`.
/// args.rewind();
///
/// assert_eq!(args.single::<u32>().unwrap() * 2, 8);
///
/// // And the same for the 2
/// assert_eq!(args.single::<u32>().unwrap() * 2, 4);
///
/// // WAIT, NO. I wanted to concatenate them into a "42" string...
/// // Argh, what should I do now????
/// // ....
/// // oh, `restore`
/// args.restore();
///
/// let res = format!("{}{}", args.single::<String>().unwrap(), args.single::<String>().unwrap());
///
/// // Yay.
/// assert_eq!(res, "42");
/// ```
///
/// Hmm, taking a glance at the prior example, it seems we have an issue with reading the same argument over and over.
/// Is there a more sensible solution than rewinding...? Actually, there is! The `*_n` methods:
///
/// ```rust
/// use serenity::framework::standard::Args;
///
/// let mut args = Args::new("trois cinq quatre six", &[" ".to_string()]);
///
/// assert_eq!(args.single_n::<String>().unwrap(), "trois");
///
/// // It might suggest we've lost the `trois`. But in fact, we didn't! And not only that, we can do it an infinite amount of times!
/// assert_eq!(args.single_n::<String>().unwrap(), "trois");
/// assert_eq!(args.single_n::<String>().unwrap(), "trois");
/// assert_eq!(args.single_n::<String>().unwrap(), "trois");
/// assert_eq!(args.single_n::<String>().unwrap(), "trois");
///
/// // Only if we use its brother method we'll then lose it.
/// assert_eq!(args.single::<String>().unwrap(), "trois");
/// assert_eq!(args.single::<String>().unwrap(), "cinq");
/// assert_eq!(args.single::<String>().unwrap(), "quatre");
/// assert_eq!(args.single::<String>().unwrap(), "six");
/// ```
#[derive(Clone, Debug)]
pub struct Args {
    message: String,
    args: Vec<Token>,
    offset: usize,
}

impl Args {
    /// Create a new instance of `Args` for parsing arguments.
    ///
    /// For more reference, look at [`Args`]'s struct documentation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(
    /// // Our message from which we'll parse over.
    /// "the quick brown fox jumps over the lazy",
    ///
    /// // The "delimiters", or aka the separators. They denote how we distinguish arguments as their own.
    /// // For this example, we'll use one delimiter, the space (`0x20`), which will separate the message.
    /// &[" ".to_string()],
    /// );
    ///
    /// assert_eq!(args.single::<String>().unwrap(), "the");
    /// assert_eq!(args.single::<String>().unwrap(), "quick");
    /// assert_eq!(args.single::<String>().unwrap(), "brown");
    ///
    /// // We shall not see `the quick brown` again.
    /// assert_eq!(args.rest(), "fox jumps over the lazy");
    /// ```
    ///
    /// [`Args`]: #struct.Args.html
    pub fn new(message: &str, possible_delimiters: &[String]) -> Self {
        let delims = possible_delimiters
            .iter()
            .filter(|d| message.contains(d.as_str()))
            .flat_map(|s| s.chars())
            .collect::<Vec<_>>();

        let mut args = Vec::new();

        // If there are no delimiters, then the only possible argument is the whole message.
        if delims.is_empty() && !message.is_empty() {
            args.push(Token::new(TokenKind::Argument, &message[..], 0));
        } else {
            let mut lex = Lexer::new(message, &delims);

            while let Some(token) = lex.commit() {
                if token.kind == TokenKind::Delimiter {
                    continue;
                }

                args.push(token);
            }
        }

        Args {
            args,
            message: message.to_string(),
            offset: 0,
        }
    }

    /// Retrieves the current argument. Does not parse.
    ///
    /// # Note
    ///
    /// This borrows `Args` for the entire lifetime of the returned argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.current(), Some("4"));
    /// args.next();
    /// assert_eq!(args.current(), Some("2"));
    /// args.next();
    /// assert_eq!(args.current(), None);
    /// ```
    pub fn current(&self) -> Option<&str> {
        self.args.get(self.offset).map(|t| t.lit.as_str())
    }

    /// Trims the current argument off leading and trailing whitespace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("     42     ", &[]);
    ///
    /// assert_eq!(args.trim().current(), Some("42"));
    /// ```
    pub fn trim(&mut self) -> &mut Self {
        if self.is_empty() {
            return self;
        }

        self.args[self.offset].lit = self.args[self.offset].lit.trim().to_string();

        self
    }

    /// Trims all of the arguments after the offset off leading and trailing whitespace.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("     42     , 84 ,\t168\t", &[",".to_string()]);
    ///
    /// args.trim_all();
    /// assert_eq!(args.single::<String>().unwrap(), "42");
    /// assert_eq!(args.single::<String>().unwrap(), "84");
    /// assert_eq!(args.single::<String>().unwrap(), "168");
    /// ```
    pub fn trim_all(&mut self) {
        if self.is_empty() {
            return;
        }

        for token in &mut self.args[self.offset..] {
            token.lit = token.lit.trim().to_string();
        }
    }

    /// Parses the current argument and advances.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.single::<u32>().unwrap(), 4);
    ///
    /// // `4` is now out of the way. Next we have `2`
    /// assert_eq!(args.single::<u32>().unwrap(), 2);
    /// ```
    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let cur = &self.args[self.offset];

        let parsed = T::from_str(&cur.lit)?;
        self.offset += 1;
        Ok(parsed)
    }

    /// Like [`single`], but doesn't advance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_n::<u32>().unwrap(), 4);
    /// assert_eq!(args.rest(), "4 2");
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let cur = &self.args[self.offset];

        Ok(T::from_str(&cur.lit)?)
    }

    /// "Skip" the argument. Equivalent to `args.single::<String>().ok()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// args.skip();
    /// assert_eq!(args.single::<u32>().unwrap(), 2);
    /// ```
    pub fn skip(&mut self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        self.single::<String>().ok()
    }

    /// Like [`skip`], but do it multiple times.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("man of culture topknot", &[" ".to_string()]);
    ///
    /// args.skip_for(3);
    /// assert_eq!(args.remaining(), 1);
    /// assert_eq!(args.single::<String>().unwrap(), "topknot");
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

        Some(vec)
    }

    /// Iterate until end of message.
    ///
    /// # Examples
    ///
    /// Assert that all of the numbers in the message are even.
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// for arg in args.iter::<u32>() {
    ///     // Default to zero in case some linguist turns our numbers into words and can't parse those.
    ///     let arg = arg.unwrap_or(0);
    ///     assert!(arg % 2 == 0);
    /// }
    ///
    /// assert!(args.is_empty());
    /// ```
    pub fn iter<T: FromStr>(&mut self) -> Iter<T>
        where T::Err: StdError {
        Iter::new(self)
    }

    /// Parses all of the remaining arguments and returns them in a `Vec`.
    /// Equivalent to `args.iter().collect::<Vec<_>>()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(*args.multiple::<u32>().unwrap(), [4, 2]);
    /// ```
    pub fn multiple<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        self.iter::<T>().collect()
    }

    /// Retrieves the current argument and also removes quotes around it if they're present.
    /// Does not parse.
    ///
    /// # Note
    ///
    /// This borrows `Args` for the entire lifetime of the returned argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 \"2\"", &[" ".to_string()]);
    ///
    /// assert_eq!(args.current_quoted(), Some("4"));
    /// args.next();
    /// assert_eq!(args.current_quoted(), Some("2"));
    /// args.next();
    /// assert_eq!(args.current_quoted(), None);
    /// ```
    pub fn current_quoted(&self) -> Option<&str> {
        self.args.get(self.offset).map(|t| quotes_extract(t))
    }

    /// Like [`single`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""4 2""#, &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_quoted::<String>().unwrap(), "4 2");
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`single`]: #method.single
    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let current = &self.args[self.offset];

        // Discard quotations if present
        let lit = quotes_extract(current);

        let parsed = T::from_str(&lit)?;
        self.offset += 1;
        Ok(parsed)
    }

    /// Like [`single_quoted`], but doesn't advance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""4 2""#, &[" ".to_string()]);
    ///
    /// assert_eq!(args.single_quoted_n::<String>().unwrap(), "4 2");
    /// assert_eq!(args.rest(), r#""4 2""#);
    /// ```
    ///
    /// [`single_quoted`]: #method.single_quoted
    pub fn single_quoted_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let current = &self.args[self.offset];

        let lit = quotes_extract(current);

        Ok(T::from_str(&lit)?)
    }

    /// Like [`iter`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// Assert that all of the numbers in quotations in the message are odd.
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""5" "3""#, &[" ".to_string()]);
    ///
    /// for arg in args.iter_quoted::<u32>() {
    ///     // Default to zero in case some linguist turns our numbers into words and can't parse those.
    ///     let arg = arg.unwrap_or(0);
    ///     assert!(arg % 2 != 0);
    /// }
    ///
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`iter`]: #method.iter
    pub fn iter_quoted<T: FromStr>(&mut self) -> IterQuoted<T>
        where T::Err: StdError {
        IterQuoted::new(self)
    }

    /// Like [`multiple`], but accounts quotes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new(r#""4" "2""#, &[" ".to_string()]);
    ///
    /// assert_eq!(*args.multiple_quoted::<u32>().unwrap(), [4, 2]);
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

    /// Returns the first argument that can be parsed and removes it from the message. The suitable argument
    /// can be in an arbitrary position in the message. Likewise, takes quotes into account.
    ///
    /// # Note
    ///
    /// Unlike the rest, this function permantently removes the argument if it was **found** and was **succesfully** parsed.
    /// Hence, use with caution.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("c4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.find::<u32>().unwrap(), 2);
    /// assert_eq!(args.single::<String>().unwrap(), "c4");
    /// assert!(args.is_empty());
    /// ```
    pub fn find<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let pos = match self.args.iter().map(|t| quotes_extract(t)).position(|s| s.parse::<T>().is_ok()) {
            Some(p) => p,
            None => return Err(Error::Eos),
        };

        let parsed = T::from_str(quotes_extract(&self.args[pos]))?;
        self.args.remove(pos);
        self.rewind();

        Ok(parsed)
    }

    /// Like [`find`], but does not remove it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("c4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.find_n::<u32>().unwrap(), 2);
    ///
    /// // The `69` is still here, so let's parse it again.
    /// assert_eq!(args.single::<String>().unwrap(), "c4");
    /// assert_eq!(args.single::<u32>().unwrap(), 2);
    /// assert!(args.is_empty());
    /// ```
    ///
    /// [`find`]: #method.find
    pub fn find_n<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let pos = match self.args.iter().map(|t| quotes_extract(t)).position(|s| s.parse::<T>().is_ok()) {
            Some(p) => p,
            None => return Err(Error::Eos),
        };

        Ok(T::from_str(quotes_extract(&self.args[pos]))?)
    }

    /// Gets original message passed to the command.
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
    pub fn full(&self) -> &str {
        &self.message
    }

    /// Gets the original message passed to the command,
    /// but without quotes (if both starting and ending quotes are present, otherwise returns as is).
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

        let end = s.rfind('"').unwrap();

        // If it got the quote at the start, then there's no closing quote.
        if end == 0 {
            return s;
        }

        &s[1..end]
    }

    /// Returns the message starting from the token in the current argument offset; the "rest" of the message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("to tre fire", &[" ".to_string()]);
    ///
    /// assert_eq!(args.rest(), "to tre fire");
    ///
    /// args.skip();
    ///
    /// assert_eq!(args.rest(), "tre fire");
    ///
    /// args.skip();
    ///
    /// assert_eq!(args.rest(), "fire");
    ///
    /// args.skip();
    ///
    /// assert_eq!(args.rest(), "");
    /// ```
    pub fn rest(&self) -> &str {
        if self.is_empty() {
            return "";
        }

        let args = &self.args[self.offset..];

        if let Some(token) = args.get(0) {
            &self.message[token.pos..]
        } else {
            &self.message[..]
        }
    }

    /// The full amount of recognised arguments.
    ///
    /// # Note
    ///
    /// This never changes. Except for [`find`], which upon success, subtracts the length by 1. (e.g len of `3` becomes `2`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.len(), 2); // `2` because `["4", "2"]`
    /// ```
    ///
    /// [`find`]: #method.find
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Returns true if there are no arguments left.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("", &[" ".to_string()]);
    ///
    /// // will be `true` because passed message is empty thus no arguments.
    /// assert!(args.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.offset >= self.args.len()
    }

    /// Amount of arguments still available.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("2 4", &[" ".to_string()]);
    ///
    /// assert_eq!(args.remaining(), 2);
    ///
    /// args.skip();
    ///
    /// assert_eq!(args.remaining(), 1);
    /// ```
    pub fn remaining(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        self.len() - self.offset
    }

    /// Move to the next argument.
    /// This increments the offset pointer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// args.next();
    ///
    /// assert_eq!(args.single::<u32>().unwrap(), 2);
    /// assert!(args.is_empty());
    /// ```
    #[inline]
    pub fn next(&mut self) {
        self.offset += 1;
    }

    /// Go one step behind.
    /// This decrements the offset pointer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("4 2", &[" ".to_string()]);
    ///
    /// assert_eq!(args.single::<u32>().unwrap(), 4);
    ///
    /// // By this point, we can only parse 2 now.
    /// // However, with the help of `rewind`, we can mess with 4 again.
    /// args.rewind();
    ///
    /// assert_eq!(args.single::<u32>().unwrap() * 2, 8);
    /// ```
    #[inline]
    pub fn rewind(&mut self) {
        if self.offset == 0 {
            return;
        }

        self.offset -= 1;
    }

    /// Go back to the starting point.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serenity::framework::standard::Args;
    ///
    /// let mut args = Args::new("42 420 69", &[" ".to_string()]);
    ///
    /// // Let's parse 'em numbers!
    /// assert_eq!(args.single::<u32>().unwrap(), 42);
    /// assert_eq!(args.single::<u32>().unwrap(), 420);
    /// assert_eq!(args.single::<u32>().unwrap(), 69);
    ///
    /// // Oh, no! I actually wanted to multiply all of them by 2!
    /// // I don't want to call `rewind` 3 times manually....
    /// // Wait, I could just go entirely back!
    /// args.restore();
    ///
    /// assert_eq!(args.single::<u32>().unwrap() * 2, 84);
    /// assert_eq!(args.single::<u32>().unwrap() * 2, 840);
    /// assert_eq!(args.single::<u32>().unwrap() * 2, 138);
    /// ```
    ///
    #[inline]
    pub fn restore(&mut self) {
        self.offset = 0;
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
    ///
    /// [`len`]: #method.len
    #[deprecated(since = "0.5.3", note = "Its task was merged with `len`, please use it instead.")]
    pub fn len_quoted(&mut self) -> usize {
        self.len()
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

/// Parse each argument individually, as an iterator.
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

fn quotes_extract(token: &Token) -> &str {
    if token.kind == TokenKind::QuotedArgument {
        &token.lit[1..token.lit.len() - 1]
    } else {
        &token.lit
    }
}

#[cfg(test)]
mod test {
    use super::{Args, Error as ArgError};

    #[test]
    fn single_with_empty_message() {
        let mut args = Args::new("", &["".to_string()]);
        assert_matches!(args.single::<String>().unwrap_err(), ArgError::Eos);

        let mut args = Args::new("", &[",".to_string()]);
        assert_matches!(args.single::<String>().unwrap_err(), ArgError::Eos);
    }

    #[test]
    fn single_n_with_empty_message() {
        let args = Args::new("", &["".to_string()]);
        assert_matches!(args.single_n::<String>().unwrap_err(), ArgError::Eos);

        let args = Args::new("", &[",".to_string()]);
        assert_matches!(args.single_n::<String>().unwrap_err(), ArgError::Eos);
    }

    #[test]
    fn single_quoted_with_empty_message() {
        let mut args = Args::new("", &["".to_string()]);
        assert_matches!(args.single_quoted::<String>().unwrap_err(), ArgError::Eos);

        let mut args = Args::new("", &[",".to_string()]);
        assert_matches!(args.single_quoted::<String>().unwrap_err(), ArgError::Eos);
    }

    #[test]
    fn multiple_with_empty_message() {
        let args = Args::new("", &["".to_string()]);
        assert_matches!(args.multiple::<String>().unwrap_err(), ArgError::Eos);

        let args = Args::new("", &[",".to_string()]);
        assert_matches!(args.multiple::<String>().unwrap_err(), ArgError::Eos);
    }

    #[test]
    fn multiple_quoted_with_empty_message() {
        let args = Args::new("", &["".to_string()]);
        assert_matches!(args.multiple_quoted::<String>().unwrap_err(), ArgError::Eos);

        let args = Args::new("", &[",".to_string()]);
        assert_matches!(args.multiple_quoted::<String>().unwrap_err(), ArgError::Eos);
    }

    #[test]
    fn skip_with_empty_message() {
        let mut args = Args::new("", &["".to_string()]);
        assert_matches!(args.skip(), None);

        let mut args = Args::new("", &[",".to_string()]);
        assert_matches!(args.skip(), None);
    }

    #[test]
    fn skip_for_with_empty_message() {
        let mut args = Args::new("", &["".to_string()]);
        assert_matches!(args.skip_for(0), None);

        let mut args = Args::new("", &["".to_string()]);
        assert_matches!(args.skip_for(5), None);

        let mut args = Args::new("", &[",".to_string()]);
        assert_matches!(args.skip_for(0), None);

        let mut args = Args::new("", &[",".to_string()]);
        assert_matches!(args.skip_for(5), None);
    }

    #[test]
    fn single_i32_with_2_bytes_long_delimiter() {
        let mut args = Args::new("1, 2", &[", ".to_string()]);

        assert_eq!(args.single::<i32>().unwrap(), 1);
        assert_eq!(args.single::<i32>().unwrap(), 2);
    }

    #[test]
    fn single_i32_with_1_byte_long_delimiter_i32() {
        let mut args = Args::new("1,2", &[",".to_string()]);

        assert_eq!(args.single::<i32>().unwrap(), 1);
        assert_eq!(args.single::<i32>().unwrap(), 2);
    }

    #[test]
    fn single_i32_with_wrong_char_after_first_arg() {
        let mut args = Args::new("1, 2", &[",".to_string()]);

        assert_eq!(args.single::<i32>().unwrap(), 1);
        assert!(args.single::<i32>().is_err());
    }

    #[test]
    fn single_i32_with_one_character_being_3_bytes_long() {
        let mut args = Args::new("1★2", &["★".to_string()]);

        assert_eq!(args.single::<i32>().unwrap(), 1);
        assert_eq!(args.single::<i32>().unwrap(), 2);
    }

    #[test]
    fn single_i32_with_untrimmed_whitespaces() {
        let mut args = Args::new(" 1, 2 ", &[",".to_string()]);

        assert!(args.single::<i32>().is_err());
    }

    #[test]
    fn single_i32_n() {
        let args = Args::new("1,2", &[",".to_string()]);

        assert_eq!(args.single_n::<i32>().unwrap(), 1);
        assert_eq!(args.single_n::<i32>().unwrap(), 1);
    }

    #[test]
    fn single_quoted_chaining() {
        let mut args = Args::new(r#""1, 2" "2" """#, &[" ".to_string()]);

        assert_eq!(args.single_quoted::<String>().unwrap(), "1, 2");
        assert_eq!(args.single_quoted::<String>().unwrap(), "2");
        assert_eq!(args.single_quoted::<String>().unwrap(), "");
    }

    #[test]
    fn single_quoted_and_single_chaining() {
        let mut args = Args::new(r#""1, 2" "2" "3" 4"#, &[" ".to_string()]);

        assert_eq!(args.single_quoted::<String>().unwrap(), "1, 2");
        assert!(args.single_n::<i32>().is_err());
        assert_eq!(args.single::<String>().unwrap(), "\"2\"");
        assert_eq!(args.single_quoted::<i32>().unwrap(), 3);
        assert_eq!(args.single::<i32>().unwrap(), 4);
    }

    #[test]
    fn full_on_args() {
        let test_text = "Some text to ensure `full()` works.";
        let args = Args::new(test_text, &[" ".to_string()]);

        assert_eq!(args.full(), test_text);
    }

    #[test]
    fn multiple_quoted_strings_one_delimiter() {
        let args = Args::new(r#""1, 2" "a" "3" 4 "5"#, &[" ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", "4", "\"5"]);
    }

    #[test]
    fn multiple_quoted_strings_with_multiple_delimiter() {
        let args = Args::new(r#""1, 2" "a","3"4 "5"#, &[" ".to_string(), ",".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", "4", "\"5"]);
    }

    #[test]
    fn multiple_quoted_strings_with_multiple_delimiters() {
        let args = Args::new(r#""1, 2" "a","3" """#, &[" ".to_string(), ",".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["1, 2", "a", "3", ""]);
    }

    #[test]
    fn multiple_quoted_i32() {
        let args = Args::new(r#""1" "2" 3"#, &[" ".to_string()]);

        assert_eq!(args.multiple_quoted::<i32>().unwrap(), [1, 2, 3]);
    }

    #[test]
    fn multiple_quoted_quote_appears_without_delimiter_in_front() {
        let args = Args::new(r#"hello, my name is cake" 2"#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "my", "name", "is", "cake\"", "2"]);
    }

    #[test]
    fn multiple_quoted_single_quote() {
        let args = Args::new(r#"hello "2 b"#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "\"2 b"]);
    }

    #[test]
    fn multiple_quoted_one_quote_pair() {
        let args = Args::new(r#"hello "2 b""#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello", "2 b"]);
    }


    #[test]
    fn delimiter_before_multiple_quoted() {
        let args = Args::new(r#","hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
    }

    #[test]
    fn no_quote() {
        let args = Args::new("hello, my name is cake", &[",".to_string(), " ".to_string()]);

        assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello");
    }

    #[test]
    fn single_quoted_n() {
        let args = Args::new(r#""hello, my name is cake","test"#, &[",".to_string()]);

        assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello, my name is cake");
        assert_eq!(args.single_quoted_n::<String>().unwrap(), "hello, my name is cake");
    }

    #[test]
    fn multiple_quoted_starting_with_wrong_delimiter_in_first_quote() {
        let args = Args::new(r#""hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
    }

    #[test]
    fn multiple_quoted_with_one_correct_and_one_invalid_quote() {
        let args = Args::new(r#""hello, my name is cake" "2""#, &[",".to_string(), " ".to_string()]);

        assert_eq!(args.multiple_quoted::<String>().unwrap(), ["hello, my name is cake", "2"]);
    }

    #[test]
    fn find_i32_one_one_byte_delimiter() {
        let mut args = Args::new("hello,my name is cake 2", &[" ".to_string()]);

        assert_eq!(args.find::<i32>().unwrap(), 2);
    }

    #[test]
    fn find_i32_one_three_byte_delimiter() {
        let mut args = Args::new("hello,my name is cakeé2", &["é".to_string()]);

        assert_eq!(args.find::<i32>().unwrap(), 2);
    }

    #[test]
    fn find_i32_multiple_delimiter_but_i32_not_last() {
        let mut args = Args::new("hello,my name is 2 cake", &[" ".to_string(), ",".to_string()]);

        assert_eq!(args.find::<i32>().unwrap(), 2);
    }

    #[test]
    fn find_i32_multiple_delimiter() {
        let mut args = Args::new("hello,my name is cake 2", &[" ".to_string(), ",".to_string()]);

        assert_eq!(args.find::<i32>().unwrap(), 2);
    }

    #[test]
    fn find_n_i32() {
        let mut args = Args::new("a 2", &[" ".to_string()]);

        assert_eq!(args.find_n::<i32>().unwrap(), 2);
        assert_eq!(args.find_n::<i32>().unwrap(), 2);
    }

    #[test]
    fn skip() {
        let mut args = Args::new("1 2", &[" ".to_string()]);

        assert_eq!(args.skip().unwrap(), "1");
        assert_eq!(args.remaining(), 1);
        assert_eq!(args.single::<String>().unwrap(), "2");
    }

    #[test]
    fn skip_for() {
        let mut args = Args::new("1 2 neko 100", &[" ".to_string()]);

        assert_eq!(args.skip_for(2).unwrap(), ["1", "2"]);
        assert_eq!(args.remaining(), 2);
        assert_eq!(args.single::<String>().unwrap(), "neko");
        assert_eq!(args.single::<String>().unwrap(), "100");
    }

    #[test]
    fn len_with_one_delimiter() {
        let args = Args::new("1 2 neko 100", &[" ".to_string()]);

        assert_eq!(args.len(), 4);
        assert_eq!(args.remaining(), 4);
    }

    #[test]
    fn len_multiple_quoted() {
        let args = Args::new(r#""hello, my name is cake" "2""#, &[" ".to_string()]);

        assert_eq!(args.len(), 2);
    }

    #[test]
    fn remaining_len_before_and_after_single() {
        let mut args = Args::new("1 2", &[" ".to_string()]);

        assert_eq!(args.remaining(), 2);
        assert_eq!(args.single::<i32>().unwrap(), 1);
        assert_eq!(args.remaining(), 1);
        assert_eq!(args.single::<i32>().unwrap(), 2);
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_single_quoted() {
        let mut args = Args::new(r#""1" "2" "3""#, &[" ".to_string()]);

        assert_eq!(args.remaining(), 3);
        assert_eq!(args.single_quoted::<i32>().unwrap(), 1);
        assert_eq!(args.remaining(), 2);
        assert_eq!(args.single_quoted::<i32>().unwrap(), 2);
        assert_eq!(args.remaining(), 1);
        assert_eq!(args.single_quoted::<i32>().unwrap(), 3);
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_skip() {
        let mut args = Args::new("1 2", &[" ".to_string()]);

        assert_eq!(args.remaining(), 2);
        assert_eq!(args.skip().unwrap(), "1");
        assert_eq!(args.remaining(), 1);
        assert_eq!(args.skip().unwrap(), "2");
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_skip_empty_string() {
        let mut args = Args::new("", &[" ".to_string()]);

        assert_eq!(args.remaining(), 0);
        assert_eq!(args.skip(), None);
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_skip_for() {
        let mut args = Args::new("1 2", &[" ".to_string()]);

        assert_eq!(args.remaining(), 2);
        assert_eq!(args.skip_for(2), Some(vec!["1".to_string(), "2".to_string()]));
        assert_eq!(args.skip_for(2), None);
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_find() {
        let mut args = Args::new("a 2 6", &[" ".to_string()]);

        assert_eq!(args.remaining(), 3);
        assert_eq!(args.find::<i32>().unwrap(), 2);
        assert_eq!(args.remaining(), 2);
        assert_eq!(args.find::<i32>().unwrap(), 6);
        assert_eq!(args.remaining(), 1);
        assert_eq!(args.find::<String>().unwrap(), "a");
        assert_eq!(args.remaining(), 0);
        assert_matches!(args.find::<String>().unwrap_err(), ArgError::Eos);
        assert_eq!(args.remaining(), 0);
    }

    #[test]
    fn remaining_len_before_and_after_find_n() {
        let mut args = Args::new("a 2 6", &[" ".to_string()]);

        assert_eq!(args.remaining(), 3);
        assert_eq!(args.find_n::<i32>().unwrap(), 2);
        assert_eq!(args.remaining(), 3);
    }


    #[test]
    fn multiple_strings_with_one_delimiter() {
        let args = Args::new("hello, my name is cake 2", &[" ".to_string()]);

        assert_eq!(args.multiple::<String>().unwrap(), ["hello,", "my", "name", "is", "cake", "2"]);
    }

    #[test]
    fn multiple_i32_with_one_delimiter() {
        let args = Args::new("1 2 3", &[" ".to_string()]);

        assert_eq!(args.multiple::<i32>().unwrap(), [1, 2, 3]);
    }

    #[test]
    fn multiple_i32_with_one_delimiter_and_parse_error() {
        let args = Args::new("1 2 3 abc", &[" ".to_string()]);

        assert_matches!(args.multiple::<i32>().unwrap_err(), ArgError::Parse(_));
    }

    #[test]
    fn multiple_i32_with_three_delimiters() {
        let args = Args::new("1 2 3", &[" ".to_string(), ",".to_string()]);

        assert_eq!(args.multiple::<i32>().unwrap(), [1, 2, 3]);
    }

    #[test]
    fn single_after_failed_single() {
        let mut args = Args::new("b 2", &[" ".to_string()]);

        assert_matches!(args.single::<i32>().unwrap_err(), ArgError::Parse(_));
        // Test that `single` short-circuts on an error and leaves the source as is.
        assert_eq!(args.remaining(), 2);
        assert_eq!(args.single::<String>().unwrap(), "b");
        assert_eq!(args.single::<String>().unwrap(), "2");
    }

    #[test]
    fn remaining_len_after_failed_single_quoted() {
        let mut args = Args::new("b a", &[" ".to_string()]);

        assert_eq!(args.remaining(), 2);
        // Same goes for `single_quoted` and the alike.
        assert_matches!(args.single_quoted::<i32>().unwrap_err(), ArgError::Parse(_));
        assert_eq!(args.remaining(), 2);
    }

    #[test]
    fn no_delims_entire_message() {
        let mut args = Args::new("abc", &[]);

        assert_eq!(args.remaining(), 1);
        assert_eq!(args.single::<String>().unwrap(), "abc");
        assert_eq!(args.remaining(), 0);
    }
}
