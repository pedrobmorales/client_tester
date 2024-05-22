use crate::{models::status::StatusUserInfo, users::UserInfo};

#[derive(Default)]
pub struct Errors {
    pub strings: Vec<String>,
}

impl Errors {
    pub fn new() -> Errors {
        Errors {
            strings: Vec::new(),
        }
    }
    pub fn add_error(&mut self, err: String) {
        self.strings.push(err);
    }

    pub fn string_eq_assert(&mut self, str1: String, str2: String) {
        if str1 != str2 {
            let res = format!("{}!={}", str1, str2);
            self.strings.push(res);
        }
    }
    pub fn string_slice_eq_assert(&mut self, str1: &str, str2: &str) {
        if str1 != str2 {
            let res = format!("{}!={}", str1, str2);
            self.strings.push(res);
        }
    }
    pub fn bool_assert(&mut self, check: bool, reason: String) {
        if !check {
            self.strings.push(reason);
        }
    }
    pub fn assert_pop(&mut self) {
        if !self.strings.is_empty() {
            dbg!(&self.strings);
            panic!();
        }
    }
    pub fn num_eq_assert<T: PartialEq>(&mut self, num1: T, num2: T, comment: &str) {
        if num1 != num2 {
            self.add_error(comment.to_string());
        }
    }
    pub fn expect_none<T>(&mut self, opt: &Option<T>, field_name: &str) {
        if opt.is_some() {
            self.add_error(format!("{} expected to be None, was Some", field_name))
        }
    }
    pub fn expect_some<T>(&mut self, opt: &Option<T>, field_name: &str) {
        if opt.is_none() {
            self.add_error(format!("{} expected to be Some, was None", field_name))
        }
    }
    pub fn verify_user(&mut self, user_object: &StatusUserInfo, user_info: &UserInfo) {
        self.string_slice_eq_assert(&user_info.first_name, &user_object.first_name);
        self.string_slice_eq_assert(&user_info.last_name, &user_object.last_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let err = Errors::new();
        assert!(err.strings.is_empty());
    }

    #[test]
    fn test_empty_and_pop() {
        let mut err = Errors::new();
        assert!(err.strings.is_empty());
        err.assert_pop();
    }

    #[test]
    fn test_add_equal_strings_empty() {
        let mut err = Errors::new();
        err.string_eq_assert("String".to_string(), "String".to_string());
        err.assert_pop();
    }

    #[test]
    #[should_panic]
    fn test_add_unequal_strings_panic() {
        let mut err = Errors::new();
        err.string_eq_assert("String2".to_string(), "String1".to_string());
        err.assert_pop();
    }
}
