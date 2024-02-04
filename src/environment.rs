use crate::error::{Error, LoxResult};
use crate::token::{Literal, Token};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new_global() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new(enclosing: Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    // The RunTime error needs to be changed to hold token
    // in order to be able to tell exactly where in code the
    // error occurred
    pub fn get<'a>(&self, name: &Token) -> LoxResult<'a, Literal> {
        match self.values.get(name.lexeme) {
            Some(v) => Ok(v.clone()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.borrow().get(name);
                }
                Err(Error::RunTime(format!(
                    "Undefined variable: {}.",
                    name.lexeme,
                )))
            }
        }
    }

    pub fn assign<'a>(&mut self, name: String, value: Literal) -> LoxResult<'a, ()> {
        match self.values.contains_key(&name) {
            true => self.define(name, value),
            false => {
                if let Some(enclosing) = &mut self.enclosing {
                    return enclosing.borrow_mut().assign(name, value);
                }
                return Err(Error::RunTime(format!("Undefined variable: {}.", name)));
            }
        }
        Ok(())
    }
}
