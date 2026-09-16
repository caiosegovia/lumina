#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedRange {
    pub start: u64,
    pub end: u64,
}

impl BoundedRange {
    pub fn len(self) -> u64 {
        self.end - self.start + 1
    }
}

pub fn parse_bounded_range(
    header: Option<&str>,
    total: u64,
    maximum: u64,
) -> Result<Option<BoundedRange>, String> {
    if maximum == 0 {
        return Err("RANGE_INVALID: limite de resposta zerado".into());
    }
    if total == 0 {
        return Ok(None);
    }
    let Some(value) = header else {
        return Ok(Some(BoundedRange {
            start: 0,
            end: total.min(maximum) - 1,
        }));
    };
    let value = value
        .strip_prefix("bytes=")
        .ok_or_else(|| "RANGE_INVALID: unidade não suportada".to_string())?;
    if value.contains(',') {
        return Err("RANGE_INVALID: múltiplos intervalos não são suportados".into());
    }
    let (start_text, end_text) = value
        .split_once('-')
        .ok_or_else(|| "RANGE_INVALID: formato inválido".to_string())?;

    if start_text.is_empty() {
        let requested = end_text
            .parse::<u64>()
            .map_err(|_| "RANGE_INVALID: sufixo inválido".to_string())?;
        if requested == 0 {
            return Err("RANGE_INVALID: sufixo zerado".into());
        }
        let length = requested.min(maximum).min(total);
        return Ok(Some(BoundedRange {
            start: total - length,
            end: total - 1,
        }));
    }

    let start = start_text
        .parse::<u64>()
        .map_err(|_| "RANGE_INVALID: início inválido".to_string())?;
    if start >= total {
        return Err("RANGE_UNSATISFIABLE: início fora do arquivo".into());
    }
    let requested_end = if end_text.is_empty() {
        total - 1
    } else {
        end_text
            .parse::<u64>()
            .map_err(|_| "RANGE_INVALID: fim inválido".to_string())?
    };
    if requested_end < start {
        return Err("RANGE_UNSATISFIABLE: intervalo invertido".into());
    }
    let bounded_end = start
        .saturating_add(maximum - 1)
        .min(requested_end)
        .min(total - 1);
    Ok(Some(BoundedRange {
        start,
        end: bounded_end,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHUNK: u64 = 4 * 1024 * 1024;

    #[test]
    fn missing_range_never_returns_the_whole_large_file() {
        let range = parse_bounded_range(None, 64 * 1024 * 1024 * 1024, CHUNK)
            .unwrap()
            .unwrap();
        assert_eq!(
            range,
            BoundedRange {
                start: 0,
                end: CHUNK - 1
            }
        );
    }

    #[test]
    fn explicit_range_is_capped_and_overflow_safe() {
        let range = parse_bounded_range(Some("bytes=1-18446744073709551615"), u64::MAX, CHUNK)
            .unwrap()
            .unwrap();
        assert_eq!(range.start, 1);
        assert_eq!(range.len(), CHUNK);
    }

    #[test]
    fn suffix_range_returns_the_bounded_tail() {
        assert_eq!(
            parse_bounded_range(Some("bytes=-500"), 1_000, CHUNK).unwrap(),
            Some(BoundedRange {
                start: 500,
                end: 999
            })
        );
    }

    #[test]
    fn rejects_multiple_inverted_and_out_of_bounds_ranges() {
        assert!(parse_bounded_range(Some("bytes=0-1,3-4"), 10, CHUNK).is_err());
        assert!(parse_bounded_range(Some("bytes=8-2"), 10, CHUNK).is_err());
        assert!(parse_bounded_range(Some("bytes=10-"), 10, CHUNK).is_err());
    }

    #[test]
    fn empty_file_has_no_allocation() {
        assert_eq!(parse_bounded_range(None, 0, CHUNK).unwrap(), None);
    }
}
