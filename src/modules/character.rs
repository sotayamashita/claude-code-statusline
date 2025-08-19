use super::Module;

pub struct CharacterModule {
    success_symbol: String,
    error_symbol: String,
    is_error: bool,
}

impl CharacterModule {
    pub fn new() -> Self {
        Self {
            success_symbol: "❯".to_string(),
            error_symbol: "❯".to_string(), // Can be different in the future
            is_error: false,
        }
    }
}

impl Default for CharacterModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for CharacterModule {
    fn name(&self) -> &str {
        "character"
    }
    
    fn should_display(&self) -> bool {
        true // Always display prompt character
    }
    
    fn render(&self) -> String {
        if self.is_error {
            self.error_symbol.clone()
        } else {
            self.success_symbol.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_character_module() {
        let module = CharacterModule::new();
        assert_eq!(module.name(), "character");
        assert!(module.should_display());
        assert_eq!(module.render(), "❯");
    }
}