use super::Module;

/// Character モジュール（現在未使用）
/// 
/// Claude Code のステータスラインはユーザー入力を受け付けないため、
/// Starship のようなプロンプト記号は不要。
/// 将来的に別の用途で使用する可能性を考慮して保持
pub struct CharacterModule {
    success_symbol: String,
    error_symbol: String,
    is_error: bool,
}

impl CharacterModule {
    pub fn new() -> Self {
        Self {
            success_symbol: "❯".to_string(),
            error_symbol: "❯".to_string(),
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
        false  // Claude Code では使用しない
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
        assert!(!module.should_display());
        assert_eq!(module.render(), "❯");
    }
}