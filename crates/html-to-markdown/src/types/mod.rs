//! Core types for structured HTML extraction results.
//!
//! These types are aligned with kreuzberg's `DocumentStructure` model for seamless integration.

mod document;
mod result;
pub mod structure_builder;
pub mod structure_collector;
mod tables;
mod warnings;

pub use document::{AnnotationKind, DocumentNode, DocumentStructure, NodeContent, TextAnnotation};
pub use result::ConversionResult;
pub use structure_builder::build_document_structure;
pub use structure_collector::{StructureCollector, StructureCollectorHandle};
pub use tables::{GridCell, TableData, TableGrid};
pub use warnings::{ProcessingWarning, WarningKind};
