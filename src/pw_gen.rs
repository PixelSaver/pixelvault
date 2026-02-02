//! The `pw_gen` module handles the password generator 
// use crate::app::PixelVaultApp;
use rand::Rng;

/// Password generator configuration values
#[derive(Clone)]
pub struct PasswordGeneratorConfig {
  /// Intended length of password generator
  pub length: usize,
  pub include_uppercase: bool,
  pub include_lowercase: bool,
  pub include_numbers: bool,
  pub include_symbols: bool,
  /// Whether or not to use words instead of perfectly random ones
  pub use_words: bool,
}

impl Default for PasswordGeneratorConfig {
  fn default() -> Self {
    Self {
      length: 16,
      include_uppercase: true,
      include_lowercase: true,
      include_numbers: true,
      include_symbols: true,
      use_words: false,
    }
  }
}

pub struct PasswordGenerator {
  pub(super) generated_password: String,
  pub(super) config: PasswordGeneratorConfig,
}
impl Default for PasswordGenerator {
  fn default() -> Self {
    Self {
      generated_password: String::new(),
      config: PasswordGeneratorConfig::default(),
    }
  }
}

impl PasswordGenerator {
  pub fn generate(&mut self) -> Option<String> {
    let config = &self.config;
    let mut char_set = String::new();
    
    if config.include_uppercase {
      char_set.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if config.include_lowercase {
      char_set.push_str("abcdefghijklmnopqrstuvwxyz");
    }
    if config.include_numbers {
      char_set.push_str("0123456789");
    }
    if config.include_symbols {
      char_set.push_str("~`!@#$%^&*()-_=+[]{}|;:,.<>?");
    }
    if char_set.is_empty() {
      return None;
    }
    
    let chars: Vec<char> = char_set.chars().collect();
    let mut rng = rand::rng();
    
    let password: String = (0..config.length)
      .map(|_| chars[rng.random_range(0..chars.len())])
      .collect();
    
    Some(password)
  }
  
  /// From 0 to 100 score of how strong a password is
  pub fn calc_strength(password: &str) -> u8 {
    if password.is_empty() {
      return 0;
    }
    let mut score: u8 = 0;
    
    let length = password.len();
    score += match length {
      0..=7 => 0,
      8..=11 => 20,
      12..=15 => 30,
      16..=19 => 40,
      _ => 50,
    };
    
    
    // Character variety
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_numbers = password.chars().any(|c| c.is_ascii_digit());
    let has_symbols = password.chars().any(|c| !c.is_alphanumeric());
    
    if has_lowercase { score += 10; }
    if has_uppercase { score += 10; }
    if has_numbers { score += 15; }
    if has_symbols { score += 15; }
    
    // Decrement for having repeating or sequential
    if Self::has_sequential_chars(password) {
      score = score.saturating_sub(20);
    }
    if Self::has_repeating_chars(password) {
      score = score.saturating_sub(10);
    }
    score.min(100)
  }
  
  /// Returns true if the password contains sequential integers.
  fn has_sequential_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
      if let [a, b, c] = window {
        let a_val = *a as i32;
        let b_val = *b as i32;
        let c_val = *c as i32;
        
        if b_val == a_val + 1 && c_val == b_val + 1 {
          return true;
        }
      }
    }
    false
  }
  
  /// Returns true if there is a repetition of 3 characters in a row
  fn has_repeating_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
      if window[0] == window[1] || window[1] == window[2] || window[0] == window[2] {
        return true;
      }
    }
    false
  }
}
