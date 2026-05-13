//! # SuperHarness Layer 3: Capabilities
//!
//! 特定领域的能力扩展。

pub mod types;
pub mod tool_executor;
pub mod builtin_tools;
pub mod skills;
pub mod memory_system;
pub mod retriever_engine;
pub mod query_engine;
pub mod output_parsers;
pub mod guard_rails;
pub mod example_selectors;
pub mod process_manager;
pub mod sandbox_runtime;
pub mod lsp_client;
pub mod document_loaders;
pub mod text_splitters;
pub mod vector_store;

// Re-export core types
pub use types::{
    ToolId, ToolRequest, ToolResponse, ToolMeta, ToolCategory,
    MemoryTier, MemoryEntry, MemoryQuery,
    QueryType, CodeLocation, CodeRange, QueryResult,
    ProcessState, ProcessInfo, Layer3Error, Layer3Result,
};

// Re-export trait interfaces (for dyn usage)
pub use tool_executor::{ToolExecutor, ToolValidator, ContextualExecutor, ExecutionContext, DefaultToolExecutor};
pub use memory_system::{MemoryStore, MemorySystem as MemorySystemTrait, ImportanceScorer, DecayPolicy, UnifiedMemorySystem, WorkingMemory, SessionMemory};
pub use query_engine::{QueryEngine, CodeAnalyzer, SymbolInfo, SymbolKind};
pub use process_manager::{ProcessManager as ProcessManagerTrait, ProcessSignal, ProcessLimits};
pub use sandbox_runtime::{SandboxRuntime as SandboxRuntimeTrait, SandboxId, SandboxConfig, ExecutionResult};
pub use lsp_client::{LspClient as LspClientTrait, LspCapabilities};
pub use retriever_engine::{RetrieverEngine, EmbeddingModel, ChunkingStrategy, FixedSizeChunker};
pub use vector_store::{VectorStore as VectorStoreTrait, VectorItem, VectorStoreConfig};

// Re-export builtin tools for Layer 2 integration
pub use builtin_tools::{BuiltinTool, BuiltinToolRegistry, ToolAdapter, register_builtin_tools};
pub use builtin_tools::file_ops::{ReadFileTool, WriteFileTool, EditFileTool, ListDirectoryTool};
pub use builtin_tools::search::{GrepTool, GlobTool};
pub use builtin_tools::shell::BashTool;
