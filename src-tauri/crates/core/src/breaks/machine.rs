//! Mola zamanlayıcısının saf durum makinesi: saat veya pencere bilmez, her saniye `tick` çağrılır.

/// Bu kadar saniye girdi olmazsa kullanıcı bilgisayar başında sayılmaz.
pub const IDLE_LIMIT: u32 = 60;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Working,
    Break { remaining: u32 },
}

#[derive(Debug, PartialEq)]
pub enum Event {
    BreakStarted,
    BreakEnded { completed: bool },
}

pub struct Machine {
    pub enabled: bool,
    pub interval_secs: u32,
    pub break_secs: u32,
    pub work_secs: u32,
    pub phase: Phase,
}

impl Machine {
    pub fn new(enabled: bool, interval_secs: u32, break_secs: u32) -> Self {
        Self { enabled, interval_secs, break_secs, work_secs: 0, phase: Phase::Working }
    }

    /// `idle_secs`: son klavye/fare hareketinden bu yana geçen süre.
    pub fn tick(&mut self, idle_secs: u32) -> Option<Event> {
        match self.phase {
            Phase::Working => {
                if !self.enabled {
                    self.work_secs = 0;
                    return None;
                }
                // Kullanıcı zaten yeterince uzun süre ekrandan uzaksa bu kendiliğinden bir moladır.
                if idle_secs >= self.break_secs.max(IDLE_LIMIT) {
                    self.work_secs = 0;
                    return None;
                }
                // Kısa süreli boşta kalma: sayaç durur ama sıfırlanmaz.
                if idle_secs >= IDLE_LIMIT {
                    return None;
                }
                self.work_secs += 1;
                if self.work_secs >= self.interval_secs {
                    self.phase = Phase::Break { remaining: self.break_secs };
                    return Some(Event::BreakStarted);
                }
                None
            }
            Phase::Break { remaining } => {
                if remaining <= 1 {
                    self.phase = Phase::Working;
                    self.work_secs = 0;
                    Some(Event::BreakEnded { completed: true })
                } else {
                    self.phase = Phase::Break { remaining: remaining - 1 };
                    None
                }
            }
        }
    }

    pub fn start_now(&mut self) -> Option<Event> {
        if matches!(self.phase, Phase::Break { .. }) {
            return None;
        }
        self.phase = Phase::Break { remaining: self.break_secs };
        Some(Event::BreakStarted)
    }

    /// Molayı atlar; sayaç baştan başlar.
    pub fn skip(&mut self) -> Option<Event> {
        self.end_break(0, false)
    }

    /// Molayı erteler; `minutes` dakika sonra yeniden hatırlatır.
    pub fn snooze(&mut self, minutes: u32) -> Option<Event> {
        let work = self.interval_secs.saturating_sub(minutes * 60);
        self.end_break(work, false)
    }

    fn end_break(&mut self, work_secs: u32, completed: bool) -> Option<Event> {
        if !matches!(self.phase, Phase::Break { .. }) {
            return None;
        }
        self.phase = Phase::Working;
        self.work_secs = work_secs;
        Some(Event::BreakEnded { completed })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(m: &mut Machine, secs: u32, idle: u32) -> Vec<Event> {
        (0..secs).filter_map(|_| m.tick(idle)).collect()
    }

    #[test]
    fn break_starts_after_interval_and_completes() {
        let mut m = Machine::new(true, 10, 3);
        assert!(run(&mut m, 9, 0).is_empty());
        assert_eq!(m.tick(0), Some(Event::BreakStarted));
        assert_eq!(m.phase, Phase::Break { remaining: 3 });
        assert_eq!(m.tick(0), None);
        assert_eq!(m.tick(0), None);
        assert_eq!(m.tick(0), Some(Event::BreakEnded { completed: true }));
        assert_eq!((m.phase, m.work_secs), (Phase::Working, 0));
    }

    #[test]
    fn idle_longer_than_a_break_counts_as_a_break() {
        let mut m = Machine::new(true, 100, 20);
        run(&mut m, 30, 0);
        run(&mut m, 1, IDLE_LIMIT - 1); // hâlâ başında sayılır
        assert_eq!(m.work_secs, 31);
        run(&mut m, 1, IDLE_LIMIT); // 60 sn girdi yok, mola 20 sn → doğal mola
        assert_eq!(m.work_secs, 0);
    }

    #[test]
    fn long_break_setting_needs_matching_idle() {
        let mut m = Machine::new(true, 100, 300);
        run(&mut m, 10, 0);
        run(&mut m, 1, 120); // 2 dk boşta, mola 5 dk: yeterli değil
        assert_eq!(m.work_secs, 10);
        run(&mut m, 1, 300);
        assert_eq!(m.work_secs, 0);
    }

    #[test]
    fn disabled_never_breaks() {
        let mut m = Machine::new(false, 2, 2);
        assert!(run(&mut m, 10, 0).is_empty());
    }

    #[test]
    fn skip_and_snooze() {
        let mut m = Machine::new(true, 600, 20);
        assert_eq!(m.skip(), None, "mola yokken atlanamaz");
        m.start_now();
        assert_eq!(m.skip(), Some(Event::BreakEnded { completed: false }));
        assert_eq!(m.work_secs, 0);

        m.start_now();
        m.snooze(5);
        assert_eq!(m.work_secs, 300, "5 dk sonra yeniden başlamalı");
        let evs = run(&mut m, 300, 0);
        assert_eq!(evs, vec![Event::BreakStarted]);
    }
}
