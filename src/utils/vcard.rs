const MIN_PHONE_DIGITS: usize = 8;
const MAX_PHONE_DIGITS: usize = 15;

pub fn phone_numbers(vcard: &str) -> Vec<String> {
    let mut numbers: Vec<String> = Vec::new();

    for line in vcard.lines() {
        let Some(number) = phone_number_of_line(line) else {
            continue;
        };

        if !numbers.contains(&number) {
            numbers.push(number);
        }
    }

    numbers
}

fn phone_number_of_line(line: &str) -> Option<String> {
    let property = strip_item_prefix(line.trim());

    if !property.to_ascii_lowercase().starts_with("tel") {
        return None;
    }

    let digits = whatsapp_id(property).unwrap_or_else(|| {
        property
            .rsplit(':')
            .next()
            .unwrap_or_default()
            .chars()
            .filter(char::is_ascii_digit)
            .collect()
    });

    (MIN_PHONE_DIGITS..=MAX_PHONE_DIGITS)
        .contains(&digits.len())
        .then_some(digits)
}

fn strip_item_prefix(line: &str) -> &str {
    match line.split_once('.') {
        Some((prefix, rest))
            if prefix.to_ascii_lowercase().starts_with("item")
                && prefix[4..]
                    .chars()
                    .all(|character| character.is_ascii_digit()) =>
        {
            rest
        }
        _ => line,
    }
}

fn whatsapp_id(property: &str) -> Option<String> {
    let start = property.to_ascii_lowercase().find("waid=")? + "waid=".len();

    let digits = property[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    (!digits.is_empty()).then_some(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_the_whatsapp_id() {
        let vcard =
            "BEGIN:VCARD\nFN:Ana\nTEL;type=CELL;waid=5511999990001:+55 11 99999-0001\nEND:VCARD";

        assert_eq!(phone_numbers(vcard), vec!["5511999990001"]);
    }

    #[test]
    fn falls_back_to_the_digits_of_the_number() {
        let vcard = "BEGIN:VCARD\r\nTEL;TYPE=CELL:+55 (21) 98888-0002\r\nEND:VCARD";

        assert_eq!(phone_numbers(vcard), vec!["5521988880002"]);
    }

    #[test]
    fn understands_item_prefixed_properties() {
        let vcard = "item1.TEL;waid=351912345678:+351 912 345 678\nitem1.X-ABLabel:Mobile";

        assert_eq!(phone_numbers(vcard), vec!["351912345678"]);
    }

    #[test]
    fn keeps_every_valid_number_once() {
        let vcard =
            "TEL;waid=5511999990001:+55 11 99999-0001\nTEL:+55 11 3333-4444\nTEL:+55 11 99999-0001";

        assert_eq!(phone_numbers(vcard), vec!["5511999990001", "551133334444"]);
    }

    #[test]
    fn ignores_cards_without_a_valid_phone() {
        assert!(phone_numbers("BEGIN:VCARD\nFN:Sem telefone\nTEL:123\nEND:VCARD").is_empty());
    }
}
