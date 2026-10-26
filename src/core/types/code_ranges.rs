/// One-based Unicode character position, with columns between characters.
pub type CodePosition = [usize; 2];

/// Half-open code interval from a start position to an exclusive end position.
pub type CodeRange = [CodePosition; 2];

/// Weighted range sets used to crossfade selections without losing overlapping ranges.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeRanges(pub Vec<(Vec<CodeRange>, f32)>);

impl Default for CodeRanges {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl CodeRanges {
    pub fn new(ranges: Vec<CodeRange>) -> Self {
        Self(vec![(ranges, 1.0)])
    }

    pub fn coverage(&self, position: CodePosition, empty: bool) -> f32 {
        self.0
            .iter()
            .map(|(ranges, weight)| {
                let covered = if ranges.is_empty() {
                    empty
                } else {
                    ranges
                        .iter()
                        .any(|[start, end]| *start <= position && position < *end)
                };
                if covered { *weight } else { 0.0 }
            })
            .sum::<f32>()
            .clamp(0.0, 1.0)
    }

    pub fn lerp(&self, to: &Self, progress: f32) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        let mut layers: Vec<(Vec<CodeRange>, f32)> = Vec::new();
        for (source, factor) in [(self, 1.0 - progress), (to, progress)] {
            for (ranges, weight) in &source.0 {
                let weight = weight * factor;
                if weight <= 0.0 {
                    continue;
                }
                if let Some((_, existing)) = layers.iter_mut().find(|(r, _)| r == ranges) {
                    *existing += weight;
                } else {
                    layers.push((ranges.clone(), weight));
                }
            }
        }
        Self(layers)
    }
}

/// Converts a Unicode position to a byte offset, validating both coordinates.
pub fn code_offset(code: &str, [line, column]: CodePosition) -> Result<usize, String> {
    if line == 0 || column == 0 {
        return Err("Code positions start at line 1, column 1.".into());
    }
    let mut offset = 0;
    for (index, text) in code.split('\n').enumerate() {
        if index + 1 == line {
            return text
                .char_indices()
                .map(|(byte, _)| byte)
                .chain([text.len()])
                .nth(column - 1)
                .map(|byte| offset + byte)
                .ok_or_else(|| "Code column is out of bounds.".into());
        }
        offset += text.len() + 1;
    }
    Err("Code line is out of bounds.".into())
}

pub fn code_interval(
    code: &str,
    [start, end]: CodeRange,
) -> Result<std::ops::Range<usize>, String> {
    let start = code_offset(code, start)?;
    let end = code_offset(code, end)?;
    if start > end {
        return Err("Code range end precedes its start.".into());
    }
    Ok(start..end)
}

pub fn code_position(code: &str, offset: usize) -> CodePosition {
    let prefix = &code[..offset];
    [
        prefix.bytes().filter(|b| *b == b'\n').count() + 1,
        prefix.rsplit('\n').next().unwrap().chars().count() + 1,
    ]
}

impl std::fmt::Display for CodeRanges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, (ranges, weight)) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("; ")?;
            }
            write!(f, "{ranges:?} ({weight:.2})")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_positions_and_overlapping_selections_crossfade() {
        assert_eq!(code_offset("é🦀\r\nx", [1, 3]), Ok(6));
        assert_eq!(code_offset("é🦀\r\nx", [2, 1]), Ok(8));
        assert!(code_offset("x", [0, 1]).is_err());
        assert!(code_offset("x", [1, 3]).is_err());
        assert!(code_interval("abc", [[1, 3], [1, 2]]).is_err());
        let ranges = CodeRanges::new(vec![[[1, 1], [1, 3]], [[1, 2], [1, 4]]]);
        let half = ranges.lerp(&CodeRanges::default(), 0.5);
        assert_eq!(half.coverage([1, 2], false), 0.5);
        assert_eq!(half.coverage([2, 1], true), 0.5);
        assert_eq!(half.coverage([1, 2], true), 1.0);
    }
}
