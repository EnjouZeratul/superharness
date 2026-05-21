//! # Continuum Layer 3: Capabilities
//!
//! 特定领域的能力扩展。

pub mod builtin_tools;
pub mod document_loaders;
pub mod example_selectors;
pub mod guard_rails;
pub mod lsp_client;
pub mod memory_system;
pub mod output_parsers;
pub mod process_manager;
pub mod query_engine;
pub mod retriever_engine;
pub mod sandbox_runtime;
pub mod skills;
pub mod text_splitters;
pub mod tool_executor;
pub mod types;
pub mod vector_store;

// Re-export core types
pub use types::{
    CodeLocation, CodeRange, Layer3Error, Layer3Result, MemoryEntry, MemoryQuery, MemoryTier,
    ProcessInfo, ProcessState, QueryResult, QueryType, ToolCategory, ToolId, ToolMeta, ToolRequest,
    ToolResponse,
};

// Re-export trait interfaces (for dyn usage)
pub use lsp_client::{LspCapabilities, LspClient as LspClientTrait};
pub use memory_system::{
    DecayPolicy, ImportanceScorer, MemoryStore, MemorySystem as MemorySystemTrait, SessionMemory,
    UnifiedMemorySystem, WorkingMemory,
};
pub use process_manager::{ProcessLimits, ProcessManager as ProcessManagerTrait, ProcessSignal};
pub use query_engine::{CodeAnalyzer, QueryEngine, SymbolInfo, SymbolKind};
pub use retriever_engine::{ChunkingStrategy, EmbeddingModel, FixedSizeChunker, RetrieverEngine};
pub use sandbox_runtime::{
    ExecutionResult, SandboxConfig, SandboxId, SandboxRuntime as SandboxRuntimeTrait,
};
pub use tool_executor::{
    ContextualExecutor, DefaultToolExecutor, ExecutionContext, ToolExecutor, ToolValidator,
};
pub use vector_store::{VectorItem, VectorStore as VectorStoreTrait, VectorStoreConfig};

// Re-export builtin tools for Layer 2 integration
pub use builtin_tools::file_ops::{EditFileTool, ListDirectoryTool, ReadFileTool, WriteFileTool};
pub use builtin_tools::search::{GlobTool, GrepTool};
pub use builtin_tools::shell::BashTool;
pub use builtin_tools::{register_builtin_tools, BuiltinTool, BuiltinToolRegistry, ToolAdapter};
