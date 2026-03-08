//! Assembles per-module code fragments into the final output with source map
//! offset tracking.

use oxc_sourcemap::{ConcatSourceMapBuilder, SourceMap};

use super::GenerateOutput;

#[derive(Default)]
pub struct OutputAssembler {
    code: String,
    line_offset: u32,
    sourcemaps: Vec<(SourceMap, u32)>,
}

impl OutputAssembler {
    pub fn push_unmapped(&mut self, text: impl AsRef<str>) {
        let text = text.as_ref();
        if text.is_empty() {
            return;
        }
        self.line_offset += count_newlines(text);
        self.code.push_str(text);
    }

    pub fn push_mapped(&mut self, code: &str, map: Option<SourceMap>) {
        if code.is_empty() {
            return;
        }
        if let Some(map) = map {
            self.sourcemaps.push((map, self.line_offset));
        }
        self.line_offset += count_newlines(code);
        self.code.push_str(code);
    }

    pub fn finish(self) -> GenerateOutput {
        let map = if self.sourcemaps.is_empty() {
            None
        } else {
            let mut builder = ConcatSourceMapBuilder::default();
            for (sourcemap, offset) in self.sourcemaps {
                builder.add_sourcemap(&sourcemap, offset);
            }
            Some(builder.into_sourcemap())
        };

        GenerateOutput { code: self.code, map, warnings: Vec::new() }
    }
}

fn count_newlines(text: &str) -> u32 {
    u32::try_from(text.matches('\n').count()).unwrap_or(u32::MAX)
}
