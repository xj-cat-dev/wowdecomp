use crate::fii::CooldownResult;

#[cxx::bridge(namespace = "wow::gameplay")]
mod fii {
    struct CooldownResult {
        active: bool,
        start: u32,
        duration: u32,
    }

    extern "Rust" {
        fn ends_at_or_after(candidate: u32, latest: u32) -> bool;
        fn query_cooldown(now: u32, start: u32, duration: u32) -> CooldownResult;
    }
}

pub fn query_cooldown(now: u32, start: u32, duration: u32) -> fii::CooldownResult {
    let end = start.wrapping_add(duration);

    if duration == 0 || !ends_at_or_after(end, now) {
        return CooldownResult {
            active: false,
            start: 0,
            duration: 0,
        };
    }
    CooldownResult {
        active: end != now,
        start,
        duration,
    }
}

pub fn ends_at_or_after(candidate: u32, latest: u32) -> bool {
    (candidate.wrapping_sub(latest) as i32) >= 0
}

#[cfg(test)]
mod tests {
    use crate::ends_at_or_after;

    #[test]
    fn ends_at_or_after_test() {
        assert!(ends_at_or_after(1200, 1000));
        assert!(!ends_at_or_after(800, 1000));
        assert!(ends_at_or_after(1000, 1000));
        assert!(ends_at_or_after(4, u32::MAX - 5));
    }

    #[test]
    fn query_active_cooldown() {
        let result = super::query_cooldown(3000, 1000, 5000);

        assert!(result.active);
        assert_eq!(result.start, 1000);
        assert_eq!(result.duration, 5000);
    }

    #[test]
    fn query_at_exact_expiry() {
        let result = super::query_cooldown(6000, 1000, 5000);

        assert!(!result.active);
        assert_eq!(result.start, 1000);
        assert_eq!(result.duration, 5000);
    }
}
