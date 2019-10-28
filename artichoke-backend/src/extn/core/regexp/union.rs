//! [`Regexp::union`](https://ruby-doc.org/core-2.6.3/Regexp.html#method-c-union)

use std::str;

use crate::convert::{Convert, RustBackedValue};
use crate::extn::core::array::Array;
use crate::extn::core::exception::{RubyException, RuntimeError, TypeError};
use crate::extn::core::regexp::{initialize, syntax, Regexp};
use crate::value::{Value, ValueLike};
use crate::Artichoke;

pub fn method(interp: &Artichoke, args: &[Value]) -> Result<Value, Box<dyn RubyException>> {
    let mut iter = args.iter().peekable();
    let pattern = if let Some(first) = iter.next() {
        if iter.peek().is_none() {
            if let Ok(ary) = unsafe { Array::try_from_ruby(interp, &first) } {
                let borrow = ary.borrow();
                let mut patterns = vec![];
                for pattern in borrow.as_vec(interp) {
                    if let Ok(regexp) = unsafe { Regexp::try_from_ruby(&interp, &pattern) } {
                        patterns.push(regexp.borrow().pattern.clone());
                    } else if let Ok(pattern) = pattern.funcall::<&str>("to_str", &[], None) {
                        patterns.push(syntax::escape(pattern));
                    } else {
                        return Err(Box::new(TypeError::new(
                            interp,
                            "No implicit conversion into String",
                        )));
                    }
                }
                patterns.join("|")
            } else {
                let pattern = first;
                if let Ok(regexp) = unsafe { Regexp::try_from_ruby(&interp, &pattern) } {
                    regexp.borrow().pattern.clone()
                } else if let Ok(pattern) = pattern.funcall::<&str>("to_str", &[], None) {
                    syntax::escape(pattern)
                } else {
                    return Err(Box::new(TypeError::new(
                        interp,
                        "No implicit conversion into String",
                    )));
                }
            }
        } else {
            let mut patterns = vec![];
            if let Ok(regexp) = unsafe { Regexp::try_from_ruby(&interp, &first) } {
                patterns.push(regexp.borrow().pattern.clone());
            } else if let Ok(bytes) = first.clone().try_into::<&[u8]>() {
                let pattern = str::from_utf8(bytes)
                    .map_err(|_| RuntimeError::new(interp, "Pattern is invalid UTF-8"))?;
                patterns.push(syntax::escape(pattern));
            } else if let Ok(bytes) = first.funcall::<&[u8]>("to_str", &[], None) {
                let pattern = str::from_utf8(bytes)
                    .map_err(|_| RuntimeError::new(interp, "Pattern is invalid UTF-8"))?;
                patterns.push(syntax::escape(pattern));
            } else {
                return Err(Box::new(TypeError::new(
                    interp,
                    "no implicit conversion into String",
                )));
            }
            for pattern in iter {
                if let Ok(regexp) = unsafe { Regexp::try_from_ruby(&interp, &pattern) } {
                    patterns.push(regexp.borrow().pattern.clone());
                } else if let Ok(bytes) = pattern.clone().try_into::<&[u8]>() {
                    let pattern = str::from_utf8(bytes)
                        .map_err(|_| RuntimeError::new(interp, "Pattern is invalid UTF-8"))?;
                    patterns.push(syntax::escape(pattern));
                } else if let Ok(bytes) = pattern.funcall::<&[u8]>("to_str", &[], None) {
                    let pattern = str::from_utf8(bytes)
                        .map_err(|_| RuntimeError::new(interp, "Pattern is invalid UTF-8"))?;
                    patterns.push(syntax::escape(pattern));
                } else {
                    return Err(Box::new(TypeError::new(
                        interp,
                        "no implicit conversion into String",
                    )));
                }
            }
            patterns.join("|")
        }
    } else {
        "(?!)".to_owned()
    };

    initialize::method(
        interp,
        initialize::Args {
            pattern: interp.convert(pattern),
            options: None,
            encoding: None,
        },
        None,
    )
}
