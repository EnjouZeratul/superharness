//! # Text Splitters
//!
//! 文本分割器：将长文本分割为小块。

use crate::retriever_engine::{Chunk, ChunkPosition, ChunkingStrategy, Document};

/// 递归字符文本分割器
///
/// 按分隔符层级分割文本。
pub struct RecursiveCharacterTextSplitter {
    /// 分块大小
    chunk_size: usize,
    /// 重叠大小
    chunk_overlap: usize,
    /// 分隔符优先级（按顺序尝试）
    separators: Vec<String>,
    /// 是否保持分隔符
    keep_separator: bool,
}

impl RecursiveCharacterTextSplitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            separators: vec![
                "\n\n".to_string(), // 段落
                "\n".to_string(),   // 行
                " ".to_string(),    // 词
                "".to_string(),     // 字符
            ],
            keep_separator: true,
        }
    }

    pub fn with_separators(mut self, separators: Vec<String>) -> Self {
        self.separators = separators;
        self
    }
}

impl Default for RecursiveCharacterTextSplitter {
    fn default() -> Self {
        Self::new(1000, 200)
    }
}

impl ChunkingStrategy for RecursiveCharacterTextSplitter {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        let content = &document.content;
        self.split_text(content, document)
    }
}

impl RecursiveCharacterTextSplitter {
    fn split_text(&self, text: &str, document: &Document) -> Vec<Chunk> {
        if text.len() <= self.chunk_size {
            return vec![self.create_chunk(text, 0, 1, document)];
        }

        // 尝试按分隔符分割
        for separator in &self.separators {
            if separator.is_empty() {
                // 按字符分割
                return self.split_by_characters(text, document);
            }

            if text.contains(separator) {
                return self.split_by_separator(text, separator, document);
            }
        }

        self.split_by_characters(text, document)
    }

    fn split_by_separator(&self, text: &str, separator: &str, document: &Document) -> Vec<Chunk> {
        let parts: Vec<&str> = text.split(separator).collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut start = 0;
        let mut index = 0;

        for part in parts {
            let part_len = part.len();
            let sep_len = if self.keep_separator {
                separator.len()
            } else {
                0
            };

            if current_chunk.len() + part_len + sep_len > self.chunk_size
                && !current_chunk.is_empty()
            {
                chunks.push(self.create_chunk(&current_chunk, start, index, document));
                start += current_chunk.len().saturating_sub(self.chunk_overlap);
                current_chunk = String::new();
                index += 1;
            }

            current_chunk.push_str(part);
            if self.keep_separator && !current_chunk.is_empty() {
                current_chunk.push_str(separator);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(self.create_chunk(&current_chunk, start, index, document));
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.position.total = total;
        }

        chunks
    }

    fn split_by_characters(&self, text: &str, document: &Document) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < text.len() {
            let end = (start + self.chunk_size).min(text.len());
            chunks.push(self.create_chunk(&text[start..end], start, index, document));
            start = end.saturating_sub(self.chunk_overlap);
            index += 1;
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.position.total = total;
        }

        chunks
    }

    fn create_chunk(
        &self,
        content: &str,
        start: usize,
        index: usize,
        document: &Document,
    ) -> Chunk {
        Chunk {
            id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
            doc_id: document.id.clone().unwrap_or_default(),
            content: content.to_string(),
            position: ChunkPosition {
                start,
                end: start + content.len(),
                index,
                total: 0, // 将在最后更新
            },
            metadata: document.metadata.clone(),
        }
    }
}

/// Markdown 文本分割器
#[allow(dead_code)]
pub struct MarkdownTextSplitter {
    chunk_size: usize,
    #[allow(dead_code)]
    chunk_overlap: usize,
}

impl MarkdownTextSplitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl Default for MarkdownTextSplitter {
    fn default() -> Self {
        Self::new(1000, 200)
    }
}

impl ChunkingStrategy for MarkdownTextSplitter {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        // 按 Markdown 标题分割
        let content = &document.content;
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        // 按 ## 标题分割
        let lines: Vec<&str> = content.lines().collect();
        let mut current_chunk = String::new();
        let mut chunk_start = 0;

        for line in lines {
            if line.starts_with("#") && current_chunk.len() > self.chunk_size / 2 {
                // 新标题，保存当前块
                if !current_chunk.trim().is_empty() {
                    chunks.push(Chunk {
                        id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
                        doc_id: document.id.clone().unwrap_or_default(),
                        content: current_chunk.trim().to_string(),
                        position: ChunkPosition {
                            start: chunk_start,
                            end: chunk_start + current_chunk.len(),
                            index,
                            total: 0,
                        },
                        metadata: document.metadata.clone(),
                    });
                    index += 1;
                }
                current_chunk = String::new();
                chunk_start = start;
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
            start += line.len() + 1;
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
                doc_id: document.id.clone().unwrap_or_default(),
                content: current_chunk.trim().to_string(),
                position: ChunkPosition {
                    start: chunk_start,
                    end: start,
                    index,
                    total: 0,
                },
                metadata: document.metadata.clone(),
            });
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.position.total = total;
        }

        chunks
    }
}

/// 代码文本分割器
#[allow(dead_code)]
pub struct CodeTextSplitter {
    chunk_size: usize,
    #[allow(dead_code)]
    chunk_overlap: usize,
    #[allow(dead_code)]
    language: String,
}

impl CodeTextSplitter {
    pub fn new(chunk_size: usize, chunk_overlap: usize, language: impl Into<String>) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            language: language.into(),
        }
    }
}

impl ChunkingStrategy for CodeTextSplitter {
    fn chunk(&self, document: &Document) -> Vec<Chunk> {
        // 按 函数/类 分割
        let content = &document.content;
        let mut chunks = Vec::new();

        // 简化实现：按函数定义分割
        let lines: Vec<&str> = content.lines().collect();
        let mut current_chunk = String::new();
        let mut start = 0;
        let mut index = 0;
        let mut chunk_start = 0;

        for line in lines {
            // 检测函数/类定义
            let is_definition = line.trim().starts_with("fn ")
                || line.trim().starts_with("pub fn ")
                || line.trim().starts_with("async fn ")
                || line.trim().starts_with("class ")
                || line.trim().starts_with("def ")
                || line.trim().starts_with("public ")
                || line.trim().starts_with("function ");

            if is_definition
                && current_chunk.len() > self.chunk_size / 2
                && !current_chunk.trim().is_empty()
            {
                chunks.push(Chunk {
                    id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
                    doc_id: document.id.clone().unwrap_or_default(),
                    content: current_chunk.trim().to_string(),
                    position: ChunkPosition {
                        start: chunk_start,
                        end: chunk_start + current_chunk.len(),
                        index,
                        total: 0,
                    },
                    metadata: document.metadata.clone(),
                });
                index += 1;
                current_chunk = String::new();
                chunk_start = start;
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
            start += line.len() + 1;
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(Chunk {
                id: format!("{}-{}", document.id.as_deref().unwrap_or("doc"), index),
                doc_id: document.id.clone().unwrap_or_default(),
                content: current_chunk.trim().to_string(),
                position: ChunkPosition {
                    start: chunk_start,
                    end: start,
                    index,
                    total: 0,
                },
                metadata: document.metadata.clone(),
            });
        }

        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.position.total = total;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recursive_splitter_default() {
        let splitter = RecursiveCharacterTextSplitter::default();
        assert_eq!(splitter.chunk_size, 1000);
        assert_eq!(splitter.chunk_overlap, 200);
    }

    #[test]
    fn test_markdown_splitter() {
        let splitter = MarkdownTextSplitter::default();
        let doc = Document::new("# Title\n\nContent\n\n## Section\n\nMore content");
        let chunks = splitter.chunk(&doc);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_code_splitter() {
        let splitter = CodeTextSplitter::new(500, 100, "rust");
        let doc = Document::new("fn foo() {}\n\nfn bar() {}");
        let chunks = splitter.chunk(&doc);
        assert!(!chunks.is_empty());
    }
}
