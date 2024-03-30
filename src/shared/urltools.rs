use url::Url;

pub trait WithVar {
    fn with_var<T: AsRef<str>>(self, var: &str, val: T) -> Url;
    fn with_var_opt<T: AsRef<str>>(self, var: &str, val: Option<T>) -> Url;
    fn with_var_key(self, var: &str) -> Url;
}

impl WithVar for Url {
    fn with_var_opt<T: AsRef<str>>(mut self, var: &str, val: Option<T>) -> Url {
        if let Some(val) = val {
            self.query_pairs_mut().append_pair(var, val.as_ref());
        }
        self
    }

    fn with_var<T: AsRef<str>>(mut self, var: &str, val: T) -> Url {
        self.query_pairs_mut().append_pair(var, val.as_ref());
        self
    }

    fn with_var_key(mut self, var: &str) -> Url {
        self.query_pairs_mut().append_key_only(var);
        self
    }

}
