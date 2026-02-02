//! The `pw_gen` module handles the password generator 
// use crate::app::PixelVaultApp;
use rand::Rng;
extern crate zxcvbn;
use zxcvbn::zxcvbn;

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
  
  /// Log 10 return guesses it might take to crack this password
  pub fn calc_strength(password: &str) -> String {
    let estimate = zxcvbn(password, &[]);
    estimate.crack_times().offline_slow_hashing_1e4_per_second().to_string()
  }
  pub fn get_password_score(password: &str) -> u8{
    let estimate = zxcvbn(password, &[]);
    estimate.score() as u8
  }
}
