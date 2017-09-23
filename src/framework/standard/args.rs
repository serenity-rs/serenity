use vec_shift::Shift;
use std::str::FromStr;
use std::error::Error as StdError;
use std::fmt;
use utils::parse_quotes;

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

#[derive(Clone, Debug)]
pub struct Args {
    delimiter: String,
    delimiter_split: Vec<String>,
}

impl Args {
    pub fn new(message: &str, delimiter: &str) -> Self {
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

    /// Like [`single`], but doesn't remove the element.
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
    pub fn skip(&mut self) -> Option<String> { self.delimiter_split.shift() }

    /// Like [`skip`], but allows for multiple at once.
    ///
    /// [`skip`]: #method.skip
    pub fn skip_for(&mut self, i: u32) -> Option<Vec<String>> {
        let mut vec = Vec::with_capacity(i as usize);

        for _ in 0..i {
            vec.push(match self.delimiter_split.shift() {
                Some(x) => x,
                None => return None,
            });
        }

        Some(vec)
    }

    /// Like [`single`], but takes quotes into account.
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
        Ok(parse_quotes(&self.delimiter_split.get(0).ok_or(Error::Eos)?)
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
            .map(|s| s.parse::<T>().map_err(|e| Error::Parse(e)))
            .collect()
    }

    /// Empty outs the internal vector while parsing (if necessary) and returning them
    pub fn list<T: FromStr>(self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        self.delimiter_split
            .into_iter()
            .map(|s| s.parse::<T>().map_err(|e| Error::Parse(e)))
            .collect()
    }

    /// This method is just `internal_vector.join(delimiter)`
    pub fn full(&self) -> String { self.delimiter_split.join(&self.delimiter) }

    /// Returns the first argument that can be converted and removes it from the list.
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

impl ::std::ops::Deref for Args {
    type Target = [String];

    fn deref(&self) -> &Self::Target { &self.delimiter_split }
}
