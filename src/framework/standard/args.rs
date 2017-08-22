use vec_shift::Shift;
use std::str::FromStr;
use std::error::Error as StdError;
use utils::parse_quotes;

/// Defines how an operation on an `Args` method failed.
#[derive(Debug)]
pub enum Error {
    /// "END-OF-STRING", more precisely, there isn't anything to parse anymore.
    Eos,
    /// A parsing operation failed; the error in it can be of any returned from the `FromStr`
    /// trait.
    Parse(Box<StdError>),
}

type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Clone)]
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
    pub fn single<T: FromStr>(&mut self) -> Result<T>
        where T::Err: StdError + 'static {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        self.delimiter_split
            .shift()
            .ok_or(Error::Eos)?
            .parse::<T>()
            .map_err(|e| Error::Parse(Box::new(e)))
    }

    /// Like [`single`], but doesn't remove the element.
    ///
    /// [`single`]: #method.single
    pub fn single_n<T: FromStr>(&mut self) -> Result<T>
        where T::Err: StdError + 'static {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        self.delimiter_split
            .get(0)
            .ok_or(Error::Eos)?
            .parse::<T>()
            .map_err(|e| Error::Parse(Box::new(e)))
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
    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T>
        where T::Err: StdError + 'static {
        parse_quotes(&self.delimiter_split.shift().ok_or(Error::Eos)?)
            .remove(0)
            .parse::<T>()
            .map_err(|e| Error::Parse(Box::new(e)))
    }

    /// Like [`single_quoted`], but doesn't remove the element.
    ///
    /// [`single_quoted`]: #method.single_quoted
    pub fn single_quoted_n<T: FromStr>(&mut self) -> Result<T>
        where T::Err: StdError + 'static {
        parse_quotes(&self.delimiter_split.get(0).ok_or(Error::Eos)?)
            .remove(0)
            .parse::<T>()
            .map_err(|e| Error::Parse(Box::new(e)))
    }

    /// Like [`list`], but takes quotes into account.
    ///
    /// [`list`]: #method.list
    pub fn multiple_quoted<T: FromStr>(self) -> Result<Vec<T>>
        where T::Err: StdError + 'static {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        parse_quotes(&self.delimiter_split.join(&self.delimiter))
            .into_iter()
            .map(|s| s.parse::<T>().map_err(|e| Error::Parse(Box::new(e))))
            .collect()
    }

    /// Empty outs the internal vector while parsing (if necessary) and returning them
    pub fn list<T: FromStr>(self) -> Result<Vec<T>>
        where T::Err: StdError + 'static {
        if self.delimiter_split.is_empty() {
            return Err(Error::Eos);
        }

        self.delimiter_split
            .into_iter()
            .map(|s| s.parse::<T>().map_err(|e| Error::Parse(Box::new(e))))
            .collect()
    }

    /// This method is just `internal_vector.join(delimiter)`
    pub fn full(&self) -> String { self.delimiter_split.join(&self.delimiter) }
}

impl ::std::ops::Deref for Args {
    type Target = [String];

    fn deref(&self) -> &Self::Target { &self.delimiter_split }
}
