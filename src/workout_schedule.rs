use chrono::{DateTime, Datelike, Duration, IsoWeek, Local, NaiveDate, Weekday};
use leptos::{html::Div, leptos_dom::logging::console_log, *};
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use rrule::{RRule, Validated};

pub trait WorkoutStep: std::fmt::Debug {}
#[derive(Debug)]
/// A template for a single workout, e.g. a bicycle ride or a weight session.
/// Includes all the different steps involved.
pub struct WorkoutTemplate {
    /// The unique name of this workout.
    pub name: String,
    /// The steps that make up this workout.
    pub steps: Vec<Box<dyn WorkoutStep>>,
}
/// A schedule for a workout template.
/// Defines when the workout happens.
pub struct WorkoutSchedule {
    /// The workout template this schedule is for.
    pub template_id: i64,
    /// The date of the first occurence of this schedule.
    pub start_date: DateTime<Local>,
    /// The ical rrule that defines when this schedule occurs.
    pub reccurence: RRule<Validated>,
    /// Whether this schedule is currently active or already over.
    pub finished: bool,
}

/// An occurence of a schedule that already happened.
/// Modifying a schedule does not modify past occurences.
pub struct PastOccurence {
    /// The schedule this occurence is for.
    pub schedule_id: i64,
    /// The date of the occurence.
    pub date: DateTime<Local>,
}
/// An override for an instance of a schedule.
/// E.g. if the parameters don't follow normal scaling.
pub struct ScheduleOverride {
    /// The schedule this override is for.
    pub schedule_id: i64,
    /// Which instance in the schedule this override is for.
    pub instance: u16,
}
// /// A workout plan.
// /// Contains workout schedules that make up a workout plan.
// pub struct WorkoutPlan {
//     /// The name of this plan.
//     pub name: String,
//     /// The workout schedules included in this plan.
//     pub schedules: Vec<WorkoutSchedule>,
//     /// Past occurences of schedules of this plan.
//     pub past_occurences: Vec<PastOccurence>,
//     /// Manual overrides of workout occurences.
//     pub overrides: Vec<ScheduleOverride>,
//     /// The scaling to be applied throughout this plan.
//     pub scaling_schedule: Vec<ScalingEntry>,
// }
/// A scaling entry for a week of a workout plan.
/// Defines how the volume of workouts should be increased\decreased during this week.
pub struct ScalingEntry {
    /// first day of week this entry is for.
    pub date: DateTime<Local>,
    /// The scaling factor in additive percent.
    pub factor: i32,
}

#[component]
pub fn WorkoutDay(week: IsoWeek, today: DateTime<Local>, day: Weekday) -> impl IntoView {
    view! {
        <div class="col s1 center-align">
            <div class="row">
                <div class=move || {
                    format!(
                        "col s12 white-text {}",
                        if week == today.iso_week() && today.weekday() == day {
                            "blue darken-2"
                        } else {
                            "indigo darken-1"
                        },
                    )
                }>
                    {move || {
                        format!(
                            "{}",
                            NaiveDate::from_isoywd_opt(week.year(), week.week(), day)
                                .unwrap()
                                .format("%d"),
                        )
                    }}

                </div>
            </div>

        </div>
    }
}

#[component]
pub fn WorkoutCalendar() -> impl IntoView {
    let today = Local::now();
    let weeks: Vec<_> = (0..8)
        .map(|w| (today + Duration::weeks(w - 1)).iso_week())
        .collect();
    let weeks = create_rw_signal(weeks);
    let calendar_list_el = create_node_ref::<Div>();
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks.with_untracked(|wk| wk.last().unwrap().clone());
            let newest =
                NaiveDate::from_isoywd_opt(newest.year(), newest.week(), Weekday::Mon).unwrap();
            weeks.update(|v| v.extend((1..9).map(|w| (newest + Duration::weeks(w)).iso_week())));
        },
        UseInfiniteScrollOptions::default().direction(leptos_use::core::Direction::Bottom),
    );
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks.with_untracked(|wk| wk.iter().next().unwrap().clone());
            let newest =
                NaiveDate::from_isoywd_opt(newest.year(), newest.week(), Weekday::Mon).unwrap();
            weeks.update(|v| {
                for wk in (1..2).map(|w| (newest - Duration::weeks(w)).iso_week()) {
                    v.insert(0, wk);
                }
                // *v = v
                //     .splice(
                //         0..0,
                //         (1..9)
                //             .rev()
                //             .map(|w| (newest - Duration::weeks(w)).iso_week()),
                //     )
                //     .collect()
            });
            // if let Some(el) = calendar_list_el.get() {
            //     el.set_scroll_top(50);
            // }
        },
        UseInfiniteScrollOptions::default()
            .direction(leptos_use::core::Direction::Top)
            .interval(250.0),
    );
    view! {
        <div class="workout-calendar">
            <div class="calendar-row calendar-header white-text blue darken-1">
                <div class="col s1 center-align">
                    <h5>Week</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Mon</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Tue</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Wed</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Thu</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Fri</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Sat</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Sun</h5>
                </div>
                <div class="col s1 center-align">
                    <h5>Load</h5>
                </div>
            </div>
            <div class="calendar-container" node_ref=calendar_list_el>
                <div class="calendar-body">
                    <For each=move || weeks.get() key=|i| format!("{:?}", i) let:item>
                        <div class="calendar-row cal-content">
                            <div class="col s1 center-align valign-wrapper p-6">
                                {item.year()} - {item.week()}
                            </div>
                            <WorkoutDay week=item today=today day=Weekday::Mon/>
                            <WorkoutDay week=item today=today day=Weekday::Tue/>
                            <WorkoutDay week=item today=today day=Weekday::Wed/>
                            <WorkoutDay week=item today=today day=Weekday::Thu/>
                            <WorkoutDay week=item today=today day=Weekday::Fri/>
                            <WorkoutDay week=item today=today day=Weekday::Sat/>
                            <WorkoutDay week=item today=today day=Weekday::Sun/>
                            <div class="col s1 center-align">Load</div>

                        </div>
                    </For>
                </div>
            </div>
        </div>
    }
}
