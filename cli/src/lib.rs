//! Continuum CLI Library
//!
//! 暴露内部模块供集成测试使用。

// 允许未使用的代码（测试和未来功能）
#![allow(dead_code)]

pub mod agent;
pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod integration;
pub mod output;
pub mod tui;
