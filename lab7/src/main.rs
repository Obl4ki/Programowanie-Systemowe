use crate::gen_hash::EtcShadowEntry;

mod gen_hash;
mod break_password;

fn main() {
    let password = "dees";
    let salt = "5MfvmFOaDU";
    let entry: EtcShadowEntry = gen_hash::get_entry(password, salt);
    
    let expected_value = "CVt7jU9wJRYz3K98EklAJqp8RMG5NvReUSVK7ctVvc2VOnYVrvyTfXaIgHn2xQS78foEJZBq2oCIqwfdNp.2V1";
    assert_eq!(entry.hash, expected_value);
    
    let expected_repr = "$6$5MfvmFOaDU$CVt7jU9wJRYz3K98EklAJqp8RMG5NvReUSVK7ctVvc2VOnYVrvyTfXaIgHn2xQS78foEJZBq2oCIqwfdNp.2V1";
    assert_eq!(entry.repr(), expected_repr.to_string());

    let entry: EtcShadowEntry = gen_hash::get_entry("Achacjusz8", salt); // pozycja nr 40000
    let found_password = break_password::break_password(entry, "./lab007.medium.txt");
    println!("{found_password:?}");
}
