//! Internal prelude for conversion modules.
//!
//! This module collects the crate-local types that are threaded through most
//! conversion helpers. Import it with `use crate::prelude::*;` in internal
//! modules that need the standard converter surface.

#![allow(unused_imports)]

pub(crate) use std::borrow::Cow;
pub(crate) use std::cell::RefCell;
pub(crate) use std::collections::{BTreeMap, HashMap, HashSet};
pub(crate) use std::rc::Rc;

pub(crate) use crate::converter::context::{Context, InlineCollectorHandle};
pub(crate) use crate::converter::dom_context::DomContext;
pub(crate) use crate::error::{ConversionError, Result};
pub(crate) use crate::options::{
    CodeBlockStyle, ConversionOptions, HeadingStyle, HighlightStyle, LinkStyle, ListIndentType,
    NewlineStyle, OutputFormat, TierStrategy, UrlEscapeStyle, WhitespaceMode,
};
pub(crate) use crate::tl_types::{Dom, Parser};
pub(crate) use crate::types::{
    ConversionResult, DocumentNode, DocumentStructure, GridCell, ProcessingWarning,
    StructureCollector, StructureCollectorHandle, TableData, TableGrid, WarningKind,
};

#[cfg(feature = "metadata")]
pub(crate) use crate::metadata::{MetadataCollectorHandle, MetadataConfig};

#[cfg(feature = "visitor")]
pub(crate) use crate::visitor::{HtmlVisitor, NodeContext, NodeType, VisitResult, VisitorHandle};
