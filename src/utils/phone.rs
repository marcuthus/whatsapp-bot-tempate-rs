const BRAZIL_PREFIX: &str = "55";
const BRAZIL_MOBILE_LENGTH_WITH_NINTH_DIGIT: usize = 13;
const NINTH_DIGIT_INDEX: usize = 4;

pub fn digits(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

pub fn canonical(number: &str) -> String {
    let mut digits = digits(number);

    let has_ninth_digit = digits.starts_with(BRAZIL_PREFIX)
        && digits.len() == BRAZIL_MOBILE_LENGTH_WITH_NINTH_DIGIT
        && digits.as_bytes()[NINTH_DIGIT_INDEX] == b'9';

    if has_ninth_digit {
        digits.remove(NINTH_DIGIT_INDEX);
    }

    digits
}

pub fn same_number(first: &str, second: &str) -> bool {
    let first = canonical(first);

    !first.is_empty() && first == canonical(second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_digits() {
        assert_eq!(digits("+55 (11) 99999-9999"), "5511999999999");
    }

    #[test]
    fn drops_the_brazilian_ninth_digit() {
        assert_eq!(canonical("5511999999999"), "551199999999");
        assert_eq!(canonical("551199999999"), "551199999999");
        assert_eq!(canonical("+55 11 9 9999-9999"), "551199999999");
    }

    #[test]
    fn leaves_other_numbers_untouched() {
        assert_eq!(canonical("14155550123"), "14155550123");
        assert_eq!(canonical("5511399999999"), "5511399999999");
        assert_eq!(canonical("205691033800830"), "205691033800830");
    }

    #[test]
    fn matches_a_number_with_and_without_the_ninth_digit() {
        assert!(same_number("5511999999999", "551199999999"));
        assert!(same_number("551199999999", "+55 11 99999-9999"));
        assert!(same_number("5511999999999", "5511999999999"));
    }

    #[test]
    fn does_not_match_different_numbers_or_empty_values() {
        assert!(!same_number("5511999999999", "5511888888888"));
        assert!(!same_number("", ""));
        assert!(!same_number("abc", ""));
    }
}
