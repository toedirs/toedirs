use chrono::{DateTime, Datelike, Duration, IsoWeek, Local, NaiveDate, Weekday};
use leptos::{
    ev::{Event, SubmitEvent},
    html::Div,
    leptos_dom::logging::console_log,
    *,
};
use leptos_router::*;
use leptos_use::{use_element_hover, use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use rrule::{RRule, Validated};
use web_sys::HtmlDivElement;

use crate::elements::select::Select;

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
                // for wk in (1..2).map(|w| (newest - Duration::weeks(w)).iso_week()) {
                //     v.insert(0, wk);
                // }
                // *v = v
                //     .splice(
                //         0..0,
                //         (1..3)
                //             .rev()
                //             .map(|w| (newest - Duration::weeks(w)).iso_week())
                //             .collect::<Vec<_>>(),
                //     )
                //     .collect()
                *v = std::iter::once(1)
                    .rev()
                    .map(|w| (newest - Duration::weeks(w)).iso_week())
                    .chain((*v).iter().map(|x| *x))
                    .collect();
            });
            if let Some(el) = calendar_list_el.get() {
                el.set_scroll_top(150);
            }
        },
        UseInfiniteScrollOptions::default()
            .direction(leptos_use::core::Direction::Top)
            .interval(250.0),
    );
    create_effect(move |_| {
        if let Some(d) = calendar_list_el.get() {
            d.set_scroll_top(1);
        }
    });
    let action_button = create_node_ref::<Div>();
    let action_is_hovered = use_element_hover(action_button);
    let show_add_workout = create_rw_signal(false);
    let show_create_workout = create_rw_signal(false);
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
            <div
                node_ref=action_button
                class=move || {
                    format!(
                        "fixed-action-btn direction-top {}",
                        if action_is_hovered.get() { "active" } else { "" },
                    )
                }
            >

                <a class="btn-floating btn-large teal">
                    <i class="large material-symbols-rounded">add</i>
                </a>
                <ul>
                    <li>
                        <a
                            class="btn-floating teal"
                            style="opacity:1;"
                            alt="Add workout template"
                            on:click=move |_| { show_create_workout.set(true) }
                        >
                            <i class="material-symbols-rounded">fitness_center</i>
                        </a>
                    </li>
                    <li>
                        <a
                            class="btn-floating teal"
                            style="opacity:1;"
                            alt="Add workout entry"
                            on:click=move |_| { show_add_workout.set(true) }
                        >
                            <i class="material-symbols-rounded">event</i>
                        </a>
                    </li>
                </ul>
            </div>
            <CreateWorkoutDialog show=show_create_workout/>
            <AddWorkoutDialog show=show_add_workout/>
        </div>
    }
}

#[component]
pub fn CreateWorkoutDialog(show: RwSignal<bool>) -> impl IntoView {
    let on_submit = move |_ev: SubmitEvent| {
        show.set(false);
    };
    let select_value = create_rw_signal("".to_string());
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <div
                class="modal"
                style="z-index: 1003; display: block; opacity: 1; top: 10%;overflow:visible;"
            >
                <Form
                    action="/api/upload_fit_file"
                    method="POST"
                    enctype="multipart/form-data".to_string()
                    on:submit=on_submit
                >
                    <div class="modal-content" style="overflow:visible;">
                        <h4 class="black-text">"Create workout"</h4>
                        <div class="row">
                            <div class="col s6 input-field">
                                <input id="name" name="name" type="text"/>
                                <label for="name">Name</label>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col s6 input-field">
                                // <select name="workout_type" id="workout_type">
                                <Select value=select_value>
                                    <option value="" disabled selected>
                                        Choose workout type
                                    </option>
                                    <option value="run">
                                        <i class="material-symbols-rounded">directions_run</i>
                                        Run
                                    </option>
                                    <option value="strength">
                                        <i class="material-symbols-rounded">fitness_center</i>
                                        Strength
                                    </option>
                                    <option value="cycling">
                                        <i class="material-symbols-rounded">directions_bike</i>
                                        Cycling
                                    </option>
                                    <option value="hiking">
                                        <i class="material-symbols-rounded">directions_walk</i>
                                        General Endurance
                                    </option>
                                    <option value="endurance">
                                        <i class="material-symbols-rounded">directions_walk</i>
                                        Hike
                                    </option>
                                // </select>
                                </Select>
                                <label>Type</label>
                            </div>
                        </div>
                    </div>
                </Form>
            </div>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| { show.set(false) }
            ></div>
        </Show>
    }
}

#[component]
pub fn AddWorkoutDialog(show: RwSignal<bool>) -> impl IntoView {
    let on_submit = move |_ev: SubmitEvent| {
        show.set(false);
    };
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <div
                class="modal bottom-sheet"
                style="z-index: 1003; display: block; opacity: 1; bottom: 0%"
            >
                <Form
                    action="/api/upload_fit_file"
                    method="POST"
                    enctype="multipart/form-data".to_string()
                    on:submit=on_submit
                >
                    <div class="modal-content">
                        <h4 class="black-text">"Add workout to calendar"</h4>
                        <div class="row"></div>
                    </div>
                </Form>
            </div>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| { show.set(false) }
            ></div>
        </Show>
    }
}
