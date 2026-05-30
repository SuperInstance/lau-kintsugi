//! `lau-kintsugi` — Error recovery, debugging trails, and "golden repairs".
//!
//! Inspired by the Japanese art of kintsugi: repairing broken pottery with gold,
//! making the break lines the most beautiful part. When something breaks in PLATO
//! (a build fails, an agent crashes, a room dissolves), the repair is visible and
//! MORE valuable than the original. The gold in the cracks is the learning.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// BreakId
// ---------------------------------------------------------------------------

/// A unique identifier for a recorded break.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BreakId(pub String);

impl std::fmt::Display for BreakId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// BreakType
// ---------------------------------------------------------------------------

/// The category of a break.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BreakType {
    /// A compilation or build error.
    CompilationError,
    /// An unexpected runtime crash.
    RuntimeCrash,
    /// A conservation (constraints/invariants) violation.
    ConservationViolation,
    /// An agent process failure.
    AgentFailure,
    /// A complete build collapse.
    BuildCollapse,
    /// A room dissolving / context loss.
    RoomDissolution,
    /// A skill execution failure.
    SkillFailure,
    /// A network timeout or connectivity loss.
    NetworkTimeout,
}

impl std::fmt::Display for BreakType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::CompilationError => "CompilationError",
            Self::RuntimeCrash => "RuntimeCrash",
            Self::ConservationViolation => "ConservationViolation",
            Self::AgentFailure => "AgentFailure",
            Self::BuildCollapse => "BuildCollapse",
            Self::RoomDissolution => "RoomDissolution",
            Self::SkillFailure => "SkillFailure",
            Self::NetworkTimeout => "NetworkTimeout",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Break
// ---------------------------------------------------------------------------

/// A recorded break event — something went wrong.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Break {
    /// Unique identifier for this break.
    pub id: BreakId,
    /// The category of the break.
    pub break_type: BreakType,
    /// Human-readable description.
    pub description: String,
    /// Ticks (logical time) at which the break occurred.
    pub tick_occurred: u64,
    /// Arbitrary contextual key-value data attached to the break.
    pub context: HashMap<String, String>,
    /// Severity between 0.0 and 1.0.
    pub severity: f64,
}

impl Break {
    /// Returns `true` if this break is critical (severity > 0.8).
    pub fn is_critical(&self) -> bool {
        self.severity > 0.8
    }
}

// ---------------------------------------------------------------------------
// Repair
// ---------------------------------------------------------------------------

/// A repair applied to a previously recorded break.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repair {
    /// The `BreakId` of the break that was repaired.
    pub break_id: BreakId,
    /// The method used for repair (e.g. "auto-retry", "rollback", "hotpatch").
    pub method: String,
    /// How much insight this repair added, between 0.0 and 1.0.
    pub gold_content: f64,
    /// Tick at which the repair was applied.
    pub repaired_tick: u64,
    /// Name / identifier of the repairer.
    pub repairer: String,
    /// Insight or lesson learned from the repair.
    pub insight: String,
    /// Snapshot of state before the repair.
    pub before_state: String,
    /// Snapshot of state after the repair.
    pub after_state: String,
}

impl Repair {
    /// Returns `true` if this is a golden repair (gold_content > 0.7).
    pub fn is_golden(&self) -> bool {
        self.gold_content > 0.7
    }
}

// ---------------------------------------------------------------------------
// GoldenTrail
// ---------------------------------------------------------------------------

/// A trail of breaks and their repairs for a single logical unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldenTrail {
    /// Breaks indexed by their `BreakId`.
    pub breaks: HashMap<BreakId, Break>,
    /// Repairs indexed by the `BreakId` of the broken event.
    pub repairs: HashMap<BreakId, Repair>,
    /// Sum of all `gold_content` across repairs in this trail.
    pub total_gold: f64,
}

impl GoldenTrail {
    /// Create an empty trail.
    pub fn new() -> Self {
        Self {
            breaks: HashMap::new(),
            repairs: HashMap::new(),
            total_gold: 0.0,
        }
    }

    /// Record a break, returning its `BreakId`.
    pub fn record_break(&mut self, brk: Break) -> BreakId {
        let id = brk.id.clone();
        self.breaks.insert(id.clone(), brk);
        id
    }

    /// Repair a break recorded in this trail.
    ///
    /// Returns `None` if the break was not found or already repaired.
    #[allow(clippy::too_many_arguments)]
    pub fn repair(
        &mut self,
        break_id: &BreakId,
        method: &str,
        gold: f64,
        repairer: &str,
        insight: &str,
        before: &str,
        after: &str,
    ) -> Option<&Repair> {
        if !self.breaks.contains_key(break_id) {
            return None;
        }
        if self.repairs.contains_key(break_id) {
            return None;
        }
        let repair = Repair {
            break_id: break_id.clone(),
            method: method.to_string(),
            gold_content: gold.clamp(0.0, 1.0),
            repaired_tick: 0, // caller should set via record_break's tick
            repairer: repairer.to_string(),
            insight: insight.to_string(),
            before_state: before.to_string(),
            after_state: after.to_string(),
        };
        self.total_gold += repair.gold_content;
        self.repairs.insert(break_id.clone(), repair);
        self.repairs.get(break_id)
    }

    /// Look up a break by id.
    pub fn get_break(&self, id: &BreakId) -> Option<&Break> {
        self.breaks.get(id)
    }

    /// Look up a repair by break id.
    pub fn get_repair(&self, id: &BreakId) -> Option<&Repair> {
        self.repairs.get(id)
    }

    /// All breaks that have not yet been repaired.
    pub fn unrepaired(&self) -> Vec<&Break> {
        self.breaks
            .iter()
            .filter(|(id, _)| !self.repairs.contains_key(id))
            .map(|(_, brk)| brk)
            .collect()
    }

    /// All repairs that are golden (gold_content > 0.7).
    pub fn golden_repairs(&self) -> Vec<&Repair> {
        self.repairs
            .values()
            .filter(|r| r.is_golden())
            .collect()
    }

    /// Total gold across all repairs.
    pub fn total_gold(&self) -> f64 {
        self.total_gold
    }

    /// Frequency of each `BreakType` across all recorded breaks.
    pub fn break_frequency(&self) -> HashMap<BreakType, usize> {
        let mut freq = HashMap::new();
        for brk in self.breaks.values() {
            *freq.entry(brk.break_type.clone()).or_insert(0) += 1;
        }
        freq
    }

    /// The repair with the highest `gold_content`, if any.
    pub fn most_insightful_repair(&self) -> Option<&Repair> {
        self.repairs
            .values()
            .max_by(|a, b| a.gold_content.partial_cmp(&b.gold_content).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// The percentage of breaks that have been repaired (0.0 – 1.0).
    ///
    /// Returns 1.0 if there are no breaks (nothing to repair).
    pub fn repair_rate(&self) -> f64 {
        let total = self.breaks.len();
        if total == 0 {
            return 1.0;
        }
        self.repairs.len() as f64 / total as f64
    }

    /// A human-readable summary of the trail.
    pub fn trail_summary(&self) -> String {
        let total_breaks = self.breaks.len();
        let total_repairs = self.repairs.len();
        let gold = self.total_gold;
        let rate = self.repair_rate();

        format!(
            "GoldenTrail: {} break(s), {} repair(s), repair rate {:.1}%, total gold {:.2}",
            total_breaks,
            total_repairs,
            rate * 100.0,
            gold,
        )
    }
}

impl Default for GoldenTrail {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ArtifactHistory
// ---------------------------------------------------------------------------

/// The full break/repair history of a single artifact across iterations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactHistory {
    /// Stable identifier for the artifact.
    pub artifact_id: String,
    /// Ordered list of trails, one per iteration.
    pub trails: Vec<GoldenTrail>,
    /// Current iteration number.
    pub iteration: u32,
}

impl ArtifactHistory {
    /// Create a new history for the given artifact.
    pub fn new(artifact_id: &str) -> Self {
        Self {
            artifact_id: artifact_id.to_string(),
            trails: Vec::new(),
            iteration: 0,
        }
    }

    /// Start a new iteration, adding a fresh `GoldenTrail`.
    pub fn new_iteration(&mut self) -> &mut GoldenTrail {
        self.iteration += 1;
        self.trails.push(GoldenTrail::new());
        self.trails.last_mut().expect("freshly pushed trail")
    }

    /// The most recent trail, if any.
    pub fn current_trail(&self) -> Option<&GoldenTrail> {
        self.trails.last()
    }

    /// The most recent trail, mutable.
    pub fn current_trail_mut(&mut self) -> Option<&mut GoldenTrail> {
        self.trails.last_mut()
    }

    /// Total number of breaks across all iterations.
    pub fn total_breaks(&self) -> usize {
        self.trails.iter().map(|t| t.breaks.len()).sum()
    }

    /// Total gold across all iterations.
    pub fn total_gold(&self) -> f64 {
        self.trails.iter().map(|t| t.total_gold).sum()
    }

    /// Whether this artifact has achieved true kintsugi:
    /// - at least one break
    /// - repair rate > 0.8 across all trails
    /// - total gold > 1.0
    pub fn is_kintsugi(&self) -> bool {
        let total_breaks = self.total_breaks();
        let total_repairs: usize = self.trails.iter().map(|t| t.repairs.len()).sum();
        if total_breaks == 0 {
            return false;
        }
        let rate = total_repairs as f64 / total_breaks as f64;
        rate > 0.8 && self.total_gold() > 1.0
    }

    /// A value multiplier derived from accumulated gold.
    ///
    /// In kintsugi, the broken-repaired object is MORE valuable.
    /// Base of 1.0, plus 0.5 × total gold.
    pub fn value_multiplier(&self) -> f64 {
        1.0 + self.total_gold() * 0.5
    }

    /// A human-readable summary of the artifact's history.
    pub fn history_summary(&self) -> String {
        let breaks = self.total_breaks();
        let gold = self.total_gold();
        let kintsugi = if self.is_kintsugi() { " ✨ KINTSUGI" } else { "" };
        format!(
            "Artifact[{}]: {} iteration(s), {} break(s), {:.2} total gold, {:.2}x value multiplier{}",
            self.artifact_id,
            self.iteration,
            breaks,
            gold,
            self.value_multiplier(),
            kintsugi,
        )
    }
}

// ---------------------------------------------------------------------------
// KintsugiRegistry
// ---------------------------------------------------------------------------

/// A global registry tracking break/repair histories for many artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KintsugiRegistry {
    /// Artifact histories indexed by artifact id.
    pub histories: HashMap<String, ArtifactHistory>,
}

impl KintsugiRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            histories: HashMap::new(),
        }
    }

    /// Register a new artifact. If already registered, this is a no-op.
    pub fn register(&mut self, artifact_id: &str) {
        self.histories
            .entry(artifact_id.to_string())
            .or_insert_with(|| ArtifactHistory::new(artifact_id));
    }

    /// Record a break for an artifact. The artifact must be registered.
    ///
    /// Returns `None` if the artifact is not registered.
    pub fn record_break(&mut self, artifact_id: &str, brk: Break) -> Option<BreakId> {
        let history = self.histories.get_mut(artifact_id)?;
        let trail = history.new_iteration();
        Some(trail.record_break(brk))
    }

    /// Repair a break on an artifact. The artifact must be registered.
    ///
    /// Applies repair on the current (latest) trail. Returns `false` if the
    /// artifact isn't registered, the break doesn't exist, or is already repaired.
    #[allow(clippy::too_many_arguments)]
    pub fn repair(
        &mut self,
        artifact_id: &str,
        break_id: &BreakId,
        method: &str,
        gold: f64,
        repairer: &str,
        insight: &str,
        before: &str,
        after: &str,
    ) -> bool {
        let history = match self.histories.get_mut(artifact_id) {
            Some(h) => h,
            None => return false,
        };
        let trail: &mut GoldenTrail = match history.current_trail_mut() {
            Some(t) => t,
            None => return false,
        };
        trail.repair(break_id, method, gold, repairer, insight, before, after).is_some()
    }

    /// Look up an artifact's history.
    pub fn get(&self, artifact_id: &str) -> Option<&ArtifactHistory> {
        self.histories.get(artifact_id)
    }

    /// All artifacts that have achieved kintsugi status.
    pub fn kintsugi_artifacts(&self) -> Vec<&ArtifactHistory> {
        self.histories
            .values()
            .filter(|h| h.is_kintsugi())
            .collect()
    }

    /// The artifact with the highest `value_multiplier`, if any.
    pub fn most_valuable(&self) -> Option<&ArtifactHistory> {
        self.histories
            .values()
            .max_by(|a, b| {
                a.value_multiplier()
                    .partial_cmp(&b.value_multiplier())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Aggregate statistics across the entire registry.
    pub fn registry_stats(&self) -> KintsugiStats {
        let total_artifacts = self.histories.len();
        let mut total_breaks = 0;
        let mut total_repairs = 0;
        let mut total_gold = 0.0;
        let mut golden_artifacts = 0;
        let mut value_sum = 0.0;

        for history in self.histories.values() {
            total_breaks += history.total_breaks();
            total_repairs += history.trails.iter().map(|t| t.repairs.len()).sum::<usize>();
            total_gold += history.total_gold();
            if history.is_kintsugi() {
                golden_artifacts += 1;
            }
            value_sum += history.value_multiplier();
        }

        let avg_value_multiplier = if total_artifacts == 0 {
            0.0
        } else {
            value_sum / total_artifacts as f64
        };

        KintsugiStats {
            total_artifacts,
            total_breaks,
            total_repairs,
            total_gold,
            golden_artifacts,
            avg_value_multiplier,
        }
    }
}

impl Default for KintsugiRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// KintsugiStats
// ---------------------------------------------------------------------------

/// Aggregate statistics for a `KintsugiRegistry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KintsugiStats {
    /// Total number of artifacts registered.
    pub total_artifacts: usize,
    /// Total number of breaks across all artifacts.
    pub total_breaks: usize,
    /// Total number of repairs across all artifacts.
    pub total_repairs: usize,
    /// Sum of all `gold_content` across all repairs.
    pub total_gold: f64,
    /// Number of artifacts that have achieved kintsugi status.
    pub golden_artifacts: usize,
    /// Average `value_multiplier` across all artifacts.
    pub avg_value_multiplier: f64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Helpers -----------------------------------------------------------

    fn make_break(
        id: &str,
        bt: BreakType,
        desc: &str,
        tick: u64,
        sev: f64,
    ) -> Break {
        Break {
            id: BreakId(id.to_string()),
            break_type: bt,
            description: desc.to_string(),
            tick_occurred: tick,
            context: HashMap::new(),
            severity: sev,
        }
    }

    fn make_break_with_ctx(
        id: &str,
        bt: BreakType,
        desc: &str,
        tick: u64,
        sev: f64,
        ctx: HashMap<String, String>,
    ) -> Break {
        Break {
            id: BreakId(id.to_string()),
            break_type: bt,
            description: desc.to_string(),
            tick_occurred: tick,
            context: ctx,
            severity: sev,
        }
    }

    // -- BreakId -----------------------------------------------------------

    #[test]
    fn break_id_newtype() {
        let id = BreakId("build-42".into());
        assert_eq!(id.0, "build-42");
        assert_eq!(format!("{id}"), "build-42");
    }

    #[test]
    fn break_id_clone_eq_hash() {
        let a = BreakId("x".into());
        let b = BreakId("x".into());
        let c = BreakId("y".into());
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.clone(), a);
    }

    // -- BreakType ---------------------------------------------------------

    #[test]
    fn break_type_display() {
        assert_eq!(format!("{}", BreakType::CompilationError), "CompilationError");
        assert_eq!(format!("{}", BreakType::RuntimeCrash), "RuntimeCrash");
        assert_eq!(format!("{}", BreakType::ConservationViolation), "ConservationViolation");
        assert_eq!(format!("{}", BreakType::AgentFailure), "AgentFailure");
        assert_eq!(format!("{}", BreakType::BuildCollapse), "BuildCollapse");
        assert_eq!(format!("{}", BreakType::RoomDissolution), "RoomDissolution");
        assert_eq!(format!("{}", BreakType::SkillFailure), "SkillFailure");
        assert_eq!(format!("{}", BreakType::NetworkTimeout), "NetworkTimeout");
    }

    // -- Break -------------------------------------------------------------

    #[test]
    fn break_is_critical() {
        let b = make_break("b1", BreakType::BuildCollapse, "everything failed", 1, 0.9);
        assert!(b.is_critical());
    }

    #[test]
    fn break_is_not_critical() {
        let b = make_break("b2", BreakType::NetworkTimeout, "minor timeout", 2, 0.3);
        assert!(!b.is_critical());
    }

    #[test]
    fn break_at_boundary() {
        let b = make_break("b3", BreakType::SkillFailure, "boundary", 3, 0.8);
        assert!(!b.is_critical()); // 0.8 is NOT > 0.8
    }

    #[test]
    fn break_with_context() {
        let mut ctx = HashMap::new();
        ctx.insert("file".into(), "src/main.rs".into());
        let b = make_break_with_ctx("b4", BreakType::CompilationError, "oops", 5, 0.5, ctx);
        assert_eq!(b.context.get("file").unwrap(), "src/main.rs");
    }

    // -- Repair ------------------------------------------------------------

    #[test]
    fn repair_is_golden() {
        let r = Repair {
            break_id: BreakId("b1".into()),
            method: "hotpatch".into(),
            gold_content: 0.85,
            repaired_tick: 10,
            repairer: "plato-agent".into(),
            insight: "learned to validate before build".into(),
            before_state: "broken".into(),
            after_state: "fixed".into(),
        };
        assert!(r.is_golden());
    }

    #[test]
    fn repair_not_golden() {
        let r = Repair {
            break_id: BreakId("b2".into()),
            method: "reboot".into(),
            gold_content: 0.3,
            repaired_tick: 10,
            repairer: "admin".into(),
            insight: "not much".into(),
            before_state: "hung".into(),
            after_state: "ok".into(),
        };
        assert!(!r.is_golden());
    }

    // -- GoldenTrail -------------------------------------------------------

    #[test]
    fn trail_record_break() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::RuntimeCrash, "panic", 1, 0.9));
        assert_eq!(id.0, "b1");
        assert_eq!(trail.breaks.len(), 1);
    }

    #[test]
    fn trail_repair() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::RuntimeCrash, "panic", 1, 0.9));
        let repair = trail.repair(&id, "rollback", 0.8, "repairer-x", "learned a lot", "broken", "fixed");
        assert!(repair.is_some());
        assert_eq!(repair.unwrap().method, "rollback");
        assert!(repair.unwrap().is_golden());
    }

    #[test]
    fn trail_repair_nonexistent() {
        let mut trail = GoldenTrail::new();
        let r = trail.repair(&BreakId("ghost".into()), "x", 0.5, "me", "none", "-", "-");
        assert!(r.is_none());
    }

    #[test]
    fn trail_repair_duplicate() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::RuntimeCrash, "x", 1, 0.5));
        let first = trail.repair(&id, "a", 0.3, "me", "x", "-", "-");
        assert!(first.is_some());
        let second = trail.repair(&id, "b", 0.4, "me", "y", "-", "-");
        assert!(second.is_none());
    }

    #[test]
    fn trail_get_break() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::AgentFailure, "crashed", 7, 0.6));
        let found = trail.get_break(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().tick_occurred, 7);
    }

    #[test]
    fn trail_get_break_missing() {
        let trail = GoldenTrail::new();
        assert!(trail.get_break(&BreakId("nope".into())).is_none());
    }

    #[test]
    fn trail_get_repair() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::BuildCollapse, "boom", 2, 0.9));
        trail.repair(&id, "revert", 0.6, "bot", "fixed", "broken", "ok");
        let r = trail.get_repair(&id);
        assert!(r.is_some());
    }

    #[test]
    fn trail_unrepaired() {
        let mut trail = GoldenTrail::new();
        trail.record_break(make_break("b1", BreakType::CompilationError, "err1", 1, 0.3));
        let id2 = trail.record_break(make_break("b2", BreakType::SkillFailure, "err2", 2, 0.5));
        trail.repair(&id2, "retry", 0.2, "me", "trivial", "-", "-");
        let ur = trail.unrepaired();
        assert_eq!(ur.len(), 1);
        assert_eq!(ur[0].description, "err1");
    }

    #[test]
    fn trail_golden_repairs() {
        let mut trail = GoldenTrail::new();
        let id1 = trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        let id2 = trail.record_break(make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.9));
        trail.repair(&id1, "patch", 0.8, "a", "good insight", "x", "y");
        trail.repair(&id2, "restart", 0.2, "b", "bad", "y", "z");
        let golden = trail.golden_repairs();
        assert_eq!(golden.len(), 1);
        assert_eq!(golden[0].method, "patch");
    }

    #[test]
    fn trail_total_gold() {
        let mut trail = GoldenTrail::new();
        let id1 = trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        let id2 = trail.record_break(make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.9));
        trail.repair(&id1, "a", 0.5, "x", "-", "-", "-");
        trail.repair(&id2, "b", 0.3, "y", "-", "-", "-");
        assert!((trail.total_gold() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn trail_break_frequency() {
        let mut trail = GoldenTrail::new();
        trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.1));
        trail.record_break(make_break("b2", BreakType::CompilationError, "y", 2, 0.2));
        trail.record_break(make_break("b3", BreakType::NetworkTimeout, "z", 3, 0.3));
        let freq = trail.break_frequency();
        assert_eq!(*freq.get(&BreakType::CompilationError).unwrap(), 2);
        assert_eq!(*freq.get(&BreakType::NetworkTimeout).unwrap(), 1);
    }

    #[test]
    fn trail_most_insightful_repair() {
        let mut trail = GoldenTrail::new();
        let id1 = trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        let id2 = trail.record_break(make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.9));
        trail.repair(&id1, "a", 0.2, "x", "-", "-", "-");
        trail.repair(&id2, "b", 0.9, "y", "-", "-", "-");
        let best = trail.most_insightful_repair();
        assert!(best.is_some());
        assert!((best.unwrap().gold_content - 0.9).abs() < 1e-10);
    }

    #[test]
    fn trail_repair_rate() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::BuildCollapse, "x", 1, 0.9));
        trail.repair(&id, "revert", 0.4, "me", "insight", "-", "-");
        assert!((trail.repair_rate() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trail_repair_rate_empty() {
        let trail = GoldenTrail::new();
        assert!((trail.repair_rate() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn trail_repair_rate_partial() {
        let mut trail = GoldenTrail::new();
        trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        let id2 = trail.record_break(make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.5));
        trail.repair(&id2, "retry", 0.1, "me", "-", "-", "-");
        assert!((trail.repair_rate() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn trail_summary() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        trail.repair(&id, "patch", 0.5, "me", "lesson", "-", "-");
        let s = trail.trail_summary();
        assert!(s.contains("1 break(s)"));
        assert!(s.contains("1 repair(s)"));
        assert!(s.contains("total gold 0.5"));
    }

    // -- ArtifactHistory ---------------------------------------------------

    #[test]
    fn artifact_new() {
        let h = ArtifactHistory::new("badge-service");
        assert_eq!(h.artifact_id, "badge-service");
        assert_eq!(h.iteration, 0);
    }

    #[test]
    fn artifact_new_iteration() {
        let mut h = ArtifactHistory::new("svc");
        h.new_iteration();
        assert_eq!(h.iteration, 1);
        assert_eq!(h.trails.len(), 1);
    }

    #[test]
    fn artifact_current_trail() {
        let mut h = ArtifactHistory::new("svc");
        assert!(h.current_trail().is_none());
        h.new_iteration();
        assert!(h.current_trail().is_some());
    }

    #[test]
    fn artifact_total_breaks() {
        let mut h = ArtifactHistory::new("svc");
        let t = h.new_iteration();
        t.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        t.record_break(make_break("b2", BreakType::NetworkTimeout, "y", 2, 0.5));
        assert_eq!(h.total_breaks(), 2);
    }

    #[test]
    fn artifact_total_gold() {
        let mut h = ArtifactHistory::new("svc");
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        t.repair(&id, "patch", 0.8, "me", "great learning", "-", "-");
        assert!((h.total_gold() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn artifact_is_kintsugi_true() {
        let mut h = ArtifactHistory::new("golden-potion");
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::BuildCollapse, "boom", 1, 0.9));
        t.repair(&id, "reconstruct", 0.9, "master-artisan", "deep refactor", "ruins", "beautiful");
        let id2 = t.record_break(make_break("b2", BreakType::RuntimeCrash, "segfault", 2, 0.7));
        t.repair(&id2, "memcheck", 0.8, "master-artisan", "found leak", "crash", "stable");
        assert!(h.is_kintsugi()); // 2 breaks, 2 repairs, gold > 1.0
    }

    #[test]
    fn artifact_is_kintsugi_no_breaks() {
        let h = ArtifactHistory::new("pristine");
        assert!(!h.is_kintsugi());
    }

    #[test]
    fn artifact_is_kintsugi_low_gold() {
        let mut h = ArtifactHistory::new("dull");
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::CompilationError, "typo", 1, 0.1));
        t.repair(&id, "fix", 0.3, "me", "simple fix", "err", "ok");
        assert!(!h.is_kintsugi()); // gold <= 1.0
    }

    #[test]
    fn artifact_value_multiplier() {
        let mut h = ArtifactHistory::new("vase");
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::BuildCollapse, "shattered", 1, 0.9));
        t.repair(&id, "kintsugi-repair", 0.9, "artist", "beautiful", "shards", "masterpiece");
        assert!((h.value_multiplier() - 1.45).abs() < 1e-10); // 1.0 + 0.9*0.5
    }

    #[test]
    fn artifact_history_summary() {
        let mut h = ArtifactHistory::new("bowl");
        h.new_iteration();
        let s = h.history_summary();
        assert!(s.contains("bowl"));
        assert!(s.contains("0 break(s)"));
    }

    #[test]
    fn artifact_multi_iteration() {
        let mut h = ArtifactHistory::new("multi-svc");
        let t1 = h.new_iteration();
        let id1 = t1.record_break(make_break("b1", BreakType::NetworkTimeout, "timeout", 1, 0.4));
        t1.repair(&id1, "retry", 0.2, "bot", "added retry", "down", "up");
        let t2 = h.new_iteration();
        let id2 = t2.record_break(make_break("b2", BreakType::RuntimeCrash, "panic", 2, 0.8));
        t2.repair(&id2, "hotfix", 0.7, "dev", "fast fix", "crashed", "running");
        assert_eq!(h.total_breaks(), 2);
        assert_eq!(h.iteration, 2);
    }

    // -- KintsugiRegistry ---------------------------------------------------

    #[test]
    fn registry_register() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc-a");
        assert!(reg.histories.contains_key("svc-a"));
    }

    #[test]
    fn registry_register_duplicate() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        reg.register("svc");
        assert_eq!(reg.histories.len(), 1);
    }

    #[test]
    fn registry_record_break() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        let id = reg.record_break("svc", make_break("b1", BreakType::CompilationError, "err", 1, 0.5));
        assert!(id.is_some());
        assert_eq!(id.unwrap().0, "b1");
    }

    #[test]
    fn registry_record_break_unregistered() {
        let mut reg = KintsugiRegistry::new();
        let id = reg.record_break("ghost", make_break("b1", BreakType::CompilationError, "x", 1, 0.3));
        assert!(id.is_none());
    }

    #[test]
    fn registry_repair() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        let id = reg.record_break("svc", make_break("b1", BreakType::BuildCollapse, "boom", 1, 0.9)).unwrap();
        let ok = reg.repair("svc", &id, "reconstruct", 0.85, "artisan", "deep lesson", "shards", "whole");
        assert!(ok);
    }

    #[test]
    fn registry_repair_unregistered() {
        let mut reg = KintsugiRegistry::new();
        let id = BreakId("b1".into());
        let ok = reg.repair("ghost", &id, "fix", 0.5, "me", "-", "-", "-");
        assert!(!ok);
    }

    #[test]
    fn registry_get() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        assert!(reg.get("svc").is_some());
        assert!(reg.get("nope").is_none());
    }

    #[test]
    fn registry_kintsugi_artifacts() {
        let mut reg = KintsugiRegistry::new();
        reg.register("normal");
        reg.register("golden");
        let id = reg.record_break("golden", make_break("b1", BreakType::BuildCollapse, "boom", 1, 0.9)).unwrap();
        reg.repair("golden", &id, "kintsugi", 0.9, "artist", "amazing", "x", "y");
        let id2 = reg.record_break("golden", make_break("b2", BreakType::RuntimeCrash, "crash", 2, 0.7)).unwrap();
        reg.repair("golden", &id2, "fix", 0.8, "artist", "insight", "a", "b");
        let k = reg.kintsugi_artifacts();
        assert_eq!(k.len(), 1);
        assert_eq!(k[0].artifact_id, "golden");
    }

    #[test]
    fn registry_most_valuable() {
        let mut reg = KintsugiRegistry::new();
        assert!(reg.most_valuable().is_none());
        reg.register("a");
        reg.register("b");
        let id = reg.record_break("b", make_break("b1", BreakType::BuildCollapse, "x", 1, 0.9)).unwrap();
        reg.repair("b", &id, "kintsugi", 2.0, "artist", "amazing", "x", "y");
        let id2 = reg.record_break("b", make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.7)).unwrap();
        reg.repair("b", &id2, "fix", 1.5, "artist", "nice", "a", "b");
        let mv = reg.most_valuable().unwrap();
        assert_eq!(mv.artifact_id, "b");
    }

    #[test]
    fn registry_stats() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        let id = reg.record_break("svc", make_break("b1", BreakType::CompilationError, "err", 1, 0.3)).unwrap();
        reg.repair("svc", &id, "fix", 0.5, "me", "-", "-", "-");
        let stats = reg.registry_stats();
        assert_eq!(stats.total_artifacts, 1);
        assert_eq!(stats.total_breaks, 1);
        assert_eq!(stats.total_repairs, 1);
    }

    #[test]
    fn registry_stats_empty() {
        let reg = KintsugiRegistry::new();
        let stats = reg.registry_stats();
        assert_eq!(stats.total_artifacts, 0);
        assert_eq!(stats.avg_value_multiplier, 0.0);
    }

    // -- Serde round-trip ---------------------------------------------------

    #[test]
    fn serde_break_roundtrip() {
        let b = make_break("b1", BreakType::BuildCollapse, "boom", 42, 0.95);
        let json = serde_json::to_string(&b).unwrap();
        let deserialized: Break = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id.0, "b1");
        assert_eq!(deserialized.tick_occurred, 42);
    }

    #[test]
    fn serde_repair_roundtrip() {
        let r = Repair {
            break_id: BreakId("b1".into()),
            method: "golden-fix".into(),
            gold_content: 0.85,
            repaired_tick: 100,
            repairer: "artisan".into(),
            insight: "profound".into(),
            before_state: "broken".into(),
            after_state: "beautiful".into(),
        };
        let json = serde_json::to_string(&r).unwrap();
        let deserialized: Repair = serde_json::from_str(&json).unwrap();
        assert!(deserialized.is_golden());
    }

    #[test]
    fn serde_trail_roundtrip() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::CompilationError, "err", 1, 0.3));
        trail.repair(&id, "patch", 0.5, "me", "ok", "-", "+");
        let json = serde_json::to_string(&trail).unwrap();
        let deserialized: GoldenTrail = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.breaks.len(), 1);
        assert_eq!(deserialized.repairs.len(), 1);
    }

    #[test]
    fn serde_registry_roundtrip() {
        let mut reg = KintsugiRegistry::new();
        reg.register("svc");
        let json = serde_json::to_string(&reg).unwrap();
        let deserialized: KintsugiRegistry = serde_json::from_str(&json).unwrap();
        assert!(deserialized.histories.contains_key("svc"));
    }

    #[test]
    fn serde_stats_roundtrip() {
        let stats = KintsugiStats {
            total_artifacts: 5,
            total_breaks: 20,
            total_repairs: 18,
            total_gold: 12.5,
            golden_artifacts: 3,
            avg_value_multiplier: 2.1,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: KintsugiStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_artifacts, 5);
    }

    // -- Kintsugi philosophy edge cases -------------------------------------

    #[test]
    fn value_always_increases_with_gold() {
        let mut h = ArtifactHistory::new("bowl");
        let base = h.value_multiplier();
        assert!((base - 1.0).abs() < 1e-10);
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::BuildCollapse, "x", 1, 0.9));
        t.repair(&id, "glue", 0.5, "me", "-", "-", "+");
        assert!(h.value_multiplier() > base);
    }

    #[test]
    fn golden_value_exceeds_original() {
        let mut h = ArtifactHistory::new("priceless-vase");
        let t = h.new_iteration();
        let id = t.record_break(make_break("b1", BreakType::BuildCollapse, "shattered", 1, 0.99));
        t.repair(&id, "masterful-kintsugi", 0.99, "master", "each crack is a story", "shards", "masterpiece");
        // value_multiplier = 1.0 + 0.99 * 0.5 = 1.495
        assert!(
            h.value_multiplier() > 1.0,
            "repaired artifact should be more valuable than the original"
        );
    }

    #[test]
    fn trail_with_no_breaks_repair_rate_is_one() {
        let trail = GoldenTrail::new();
        assert!((trail.repair_rate() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn in_multi_artifact_registry_only_one_golden() {
        let mut reg = KintsugiRegistry::new();
        // Two artifacts with some breaks but not enough gold
        reg.register("plain");
        reg.register("golden-one");
        let id = reg.record_break("golden-one", make_break("b1", BreakType::BuildCollapse, "x", 1, 0.9)).unwrap();
        reg.repair("golden-one", &id, "kintsugi", 0.9, "artisan", "great", "x", "y");
        let id2 = reg.record_break("golden-one", make_break("b2", BreakType::RuntimeCrash, "y", 2, 0.7)).unwrap();
        reg.repair("golden-one", &id2, "fix", 0.8, "artisan", "deep", "a", "b");
        assert_eq!(reg.kintsugi_artifacts().len(), 1);
    }

    #[test]
    fn golden_repair_in_trail_updates_total_gold() {
        let mut trail = GoldenTrail::new();
        let id = trail.record_break(make_break("b1", BreakType::CompilationError, "typo", 1, 0.2));
        trail.repair(&id, "fix", 0.75, "dev", "not quite golden", "-", "+");
        // 0.75 is > 0.7 -> is_golden() = true
        assert_eq!(trail.golden_repairs().len(), 1);
        assert!((trail.total_gold() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn no_repaired_breaks_returns_empty_golden() {
        let trail = GoldenTrail::new();
        assert!(trail.golden_repairs().is_empty());
        assert!(trail.most_insightful_repair().is_none());
    }
}