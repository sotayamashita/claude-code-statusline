use super::Module;

pub struct ClaudeModelModule {
    model_name: String,
}

impl ClaudeModelModule {
    pub fn new(display_name: &str) -> Self {
        Self {
            model_name: display_name.to_string(),
        }
    }
}

impl Module for ClaudeModelModule {
    fn name(&self) -> &str {
        "claude_model"
    }
    
    fn should_display(&self) -> bool {
        !self.model_name.is_empty()
    }
    
    fn render(&self) -> String {
        format!("<{}>", self.model_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_claude_model_module() {
        let module = ClaudeModelModule::new("Opus");
        assert_eq!(module.name(), "claude_model");
        assert!(module.should_display());
        assert_eq!(module.render(), "<Opus>");
    }
    
    #[test]
    fn test_empty_model_name() {
        let module = ClaudeModelModule::new("");
        assert!(!module.should_display());
    }
}