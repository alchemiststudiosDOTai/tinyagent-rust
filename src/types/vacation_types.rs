use crate::completion_schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Structured vacation plan returned by the vacation planner agent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[completion_schema]
pub struct VacationPlan {
    /// Destination city and country (e.g., "Paris, France")
    pub destination: String,
    /// Number of nights included in the trip
    pub nights: u32,
    /// Optional ISO-8601 start date for the trip
    pub start_date: Option<String>,
    /// Number of travelers the plan is designed for
    pub travelers: Option<u32>,
    /// Total estimated budget for the full trip in the selected currency
    pub total_budget: f64,
    /// Estimated cost per traveler when the total is split evenly
    pub budget_per_person: Option<f64>,
    /// Currency code used for all monetary fields (e.g., "USD")
    pub currency: Option<String>,
    /// Day-by-day itinerary with planned activities
    pub itinerary: Vec<DayPlan>,
    /// Recommended lodging information (hotel, neighborhood, notes)
    pub accommodation: Option<String>,
    /// Summary of transportation logistics (flights, trains, passes)
    pub transportation: Option<String>,
    /// Key highlights or must-see experiences for the trip
    pub highlights: Vec<String>,
    /// Optional breakdown of the total budget by category
    pub budget_breakdown: Option<BudgetBreakdown>,
    /// Additional planning notes, tips, or follow-up actions
    pub notes: Option<String>,
}

/// Per-day itinerary details with activities and budget estimates.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DayPlan {
    /// 1-based day counter within the itinerary
    pub day: u32,
    /// Short summary or theme for the day
    pub title: Option<String>,
    /// Primary activities or attractions for the day in chronological order
    pub activities: Vec<String>,
    /// Estimated total spend for the day in the plan currency
    pub estimated_cost: f64,
    /// Optional notes about reservations, timing, or alternatives
    pub notes: Option<String>,
}

/// Optional budget category breakdown for the structured response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BudgetBreakdown {
    /// Total lodging cost across the trip
    pub lodging: Option<f64>,
    /// Total activity or excursion spend
    pub activities: Option<f64>,
    /// Food and dining spend estimate
    pub meals: Option<f64>,
    /// Transportation spend (local transit, trains, flights)
    pub transport: Option<f64>,
    /// Miscellaneous or contingency budget
    pub other: Option<f64>,
}
