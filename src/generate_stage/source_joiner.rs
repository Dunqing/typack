//! Assembles per-module code fragments into the final output with source map
//! offset tracking.
//!
//! `SourceJoiner` collects unmapped (raw) and mapped code fragments, then joins
//! them into a single output string with a composed source map.

use oxc_sourcemap::{ConcatSourceMapBuilder, SourceMap};

pub(super) enum SourceContent {
    /// Unmapped text (directives, imports, markers, exports)
    Raw(String),
    /// Mapped code with optional sourcemap
    Mapped { code: String, map: Option<SourceMap> },
}

#[derive(Default)]
pub(super) struct SourceJoiner {
    sources: Vec<SourceContent>,
}

impl SourceJoiner {
    pub fn append_raw(&mut self, text: impl Into<String>) {
        let text = text.into();
        if !text.is_empty() {
            self.sources.push(SourceContent::Raw(text));
        }
    }

    pub fn append_mapped(&mut self, code: String, map: Option<SourceMap>) {
        if !code.is_empty() {
            self.sources.push(SourceContent::Mapped { code, map });
        }
    }

    pub fn join(self) -> (String, Option<SourceMap>) {
        let mut output = String::new();
        let mut line_offset: u32 = 0;
        let mut sourcemaps: Vec<(SourceMap, u32)> = Vec::new();

        for source in self.sources {
            match source {
                SourceContent::Raw(text) => {
                    line_offset += count_newlines(&text);
                    output.push_str(&text);
                }
                SourceContent::Mapped { code, map } => {
                    if let Some(map) = map {
                        sourcemaps.push((map, line_offset));
                    }
                    line_offset += count_newlines(&code);
                    output.push_str(&code);
                }
            }
        }

        let map = if sourcemaps.is_empty() {
            None
        } else {
            let mut builder = ConcatSourceMapBuilder::default();
            for (sourcemap, offset) in sourcemaps {
                builder.add_sourcemap(&sourcemap, offset);
            }
            Some(builder.into_sourcemap())
        };

        (output, map)
    }
}

fn count_newlines(text: &str) -> u32 {
    text.match_indices('\n').fold(0u32, |count, _| count.saturating_add(1))
}
