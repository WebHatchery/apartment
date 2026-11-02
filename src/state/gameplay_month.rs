//! The closed month's result: ledger totals plus the few events that changed
//! the building, gathered once so the summary card and the history drawer
//! show the same thing. Also the footer's "unresolved right now" status.

use crate::text::{t, tr};

use crate::economy::MonthlyReport;
use crate::simulation::{EventSeverity, GameEvent};

use super::gameplay::GameplayState;

/// Most summary lines shown before pointing at the history.
const MAX_LINES: usize = 6;

#[derive(Clone, Debug)]
pub struct SummaryLine {
    pub text: String,
    pub severity: EventSeverity,
}

impl SummaryLine {
    fn new(text: impl Into<String>, severity: EventSeverity) -> Self {
        Self {
            text: text.into(),
            severity,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonthSummary {
    pub month: u32,
    pub report: MonthlyReport,
    pub lines: Vec<SummaryLine>,
}

/// Meaningful tenant and building events, aggregated so a busy month reads
/// in a few lines (rent and applications are counted, not listed).
fn summarize_events<'a>(events: impl Iterator<Item = &'a GameEvent>) -> Vec<SummaryLine> {
    let mut paid = 0;
    let mut missed = Vec::new();
    let mut applications = 0;
    let mut lines = Vec::new();
    for event in events {
        match event {
            GameEvent::RentPaid { .. } => paid += 1,
            GameEvent::RentMissed { tenant_name, .. } => missed.push(tenant_name.clone()),
            GameEvent::NewApplication { .. } => applications += 1,
            GameEvent::TenantMovedIn { .. }
            | GameEvent::TenantMovedOut { .. }
            | GameEvent::UpgradeCompleted { .. }
            | GameEvent::TenantDamage { .. }
            | GameEvent::CriticalCondition { .. }
            | GameEvent::PipeBurst { .. }
            | GameEvent::BoilerFailure { .. }
            | GameEvent::StructuralIssue { .. }
            | GameEvent::Inspection { .. }
            | GameEvent::Heatwave { .. }
            | GameEvent::Gentrification { .. }
            | GameEvent::StaffAction { .. } => {
                lines.push(SummaryLine::new(event.message(), event.severity()));
            }
            _ => {}
        }
    }
    let mut summary = Vec::new();
    if paid > 0 {
        summary.push(SummaryLine::new(
            tr("month.rent_paid", &[("count", &paid)]),
            EventSeverity::Positive,
        ));
    }
    if !missed.is_empty() {
        summary.push(SummaryLine::new(
            tr("month.missed_rent", &[("names", &missed.join(", "))]),
            EventSeverity::Negative,
        ));
    }
    // Problems first, then everything else in the order it happened.
    lines.sort_by_key(|line| match line.severity {
        EventSeverity::Negative => 0,
        EventSeverity::Warning => 1,
        _ => 2,
    });
    summary.extend(lines);
    if applications > 0 {
        summary.push(SummaryLine::new(
            tr("month.applications", &[("count", &applications)]),
            EventSeverity::Info,
        ));
    }
    if summary.len() > MAX_LINES {
        let hidden = summary.len() - (MAX_LINES - 1);
        summary.truncate(MAX_LINES - 1);
        summary.push(SummaryLine::new(
            tr("month.more_lines", &[("count", &hidden)]),
            EventSeverity::Info,
        ));
    }
    summary
}

impl GameplayState {
    /// Record the closed month and show its summary before anything else.
    pub(super) fn open_month_summary(&mut self, report: MonthlyReport) {
        let lines = summarize_events(self.event_log.events_at(self.current_tick));
        self.latest_month = Some(MonthSummary {
            month: self.current_tick,
            report,
            lines,
        });
        self.month_summary_open = true;
    }

    /// The most pressing issue that is still true now, for the footer. Old
    /// events never linger here; calm buildings show nothing.
    pub fn current_issue(&self) -> Option<SummaryLine> {
        let thresholds = &self.config.thresholds;
        let critical = self
            .building
            .apartments
            .iter()
            .filter(|unit| unit.condition < thresholds.critical_condition)
            .count();
        if critical > 0 {
            return Some(SummaryLine::new(
                tr("status.critical_units", &[("count", &critical)]),
                EventSeverity::Negative,
            ));
        }
        if self.funds.balance < 0 {
            return Some(SummaryLine::new(
                t("status.overdrawn"),
                EventSeverity::Negative,
            ));
        }
        let requests = self
            .tenant_stories
            .values()
            .filter(|story| story.pending_request.is_some())
            .count();
        if requests > 0 {
            return Some(SummaryLine::new(
                tr("status.requests_waiting", &[("count", &requests)]),
                EventSeverity::Warning,
            ));
        }
        let unlisted = self
            .building
            .apartments
            .iter()
            .filter(|unit| {
                unit.is_vacant()
                    && !unit.is_listed_for_lease
                    && !self.building.is_unit_sold(unit.id)
            })
            .count();
        if unlisted > 0 {
            return Some(SummaryLine::new(
                tr("status.unlisted_vacancies", &[("count", &unlisted)]),
                EventSeverity::Warning,
            ));
        }
        let poor = self
            .building
            .apartments
            .iter()
            .filter(|unit| unit.condition < thresholds.poor_condition)
            .count();
        (poor > 0).then(|| {
            SummaryLine::new(
                tr("status.needs_repair", &[("count", &poor)]),
                EventSeverity::Warning,
            )
        })
    }
}
