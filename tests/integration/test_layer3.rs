//! # Layer 3 Integration Tests
//!
//! 测试 Layer 3 能力扩展模块的集成。

#[cfg(test)]
mod tests {
    // Note: These tests require the full crate to compile.
    // They will be enabled once all dependencies are resolved.

    /*
    use sh_layer3::{
        types::*,
        builtin_tools::*,
        memory_system::*,
        tool_executor::*,
    };

    #[tokio::test]
    async fn test_builtin_tools_registration() {
        use sh_layer2::ToolRegistry;

        let registry = ToolRegistry::new();
        register_builtin_tools(&registry).unwrap();

        // Verify tools are registered
        assert!(registry.exists("read_file"));
        assert!(registry.exists("write_file"));
        assert!(registry.exists("grep"));
        assert!(registry.exists("bash"));
    }

    #[tokio::test]
    async fn test_memory_system() {
        let system = UnifiedMemorySystem::new("test-session");

        // Test working memory
        let id = system.store_at(MemoryTier::Working, "test content").await.unwrap();
        assert!(!id.is_empty());

        // Test stats
        let stats = system.stats().await.unwrap();
        assert!(stats.contains_key(&MemoryTier::Working));
    }

    #[test]
    fn test_tool_adapter() {
        let tool = ToolAdapter::new(Box::new(ReadFileTool));
        assert_eq!(tool.name(), "read_file");
    }
    */

    #[test]
    fn test_layer3_types() {
        // Basic type tests
        let response = sh_layer3::ToolResponse::success("call_1", "test", "result");
        assert!(!response.is_error);

        let err_response = sh_layer3::ToolResponse::error("call_2", "test", "error");
        assert!(err_response.is_error);
    }

    #[test]
    fn test_memory_tier() {
        let tier = sh_layer3::MemoryTier::default();
        assert_eq!(tier, sh_layer3::MemoryTier::Working);
    }
}