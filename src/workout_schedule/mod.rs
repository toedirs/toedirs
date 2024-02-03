use std::{
    collections::{HashMap, HashSet},
    iter,
};

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::workout_schedule::{
    add_template_dialog::CreateWorkoutDialog, add_workout_dialog::AddWorkoutDialog,
};
use chrono::{DateTime, Datelike, Days, Duration, IsoWeek, Local, NaiveDate, TimeZone, Weekday};
use leptos::{html::Div, *};
use leptos_router::*;
use leptos_use::{use_element_hover, use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use rrule::{RRule, RRuleSet, Tz, Unvalidated, Validated};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
use std::str::FromStr;
use strum;
use thaw::*;

use crate::elements::select::Select;

pub mod add_template_dialog;
pub mod add_workout_dialog;

pub trait WorkoutStep: std::fmt::Debug {}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type, sqlx::FromRow))]
/// A template for a single workout, e.g. a bicycle ride or a weight session.
/// Includes all the different steps involved.
pub struct WorkoutTemplate {
    /// unique id of the template.
    pub id: i64,
    /// user the template belongs to.
    pub user_id: i32,
    /// The unique name of this workout.
    pub template_name: String,
    /// the type of workout this is.
    pub workout_type: WorkoutType, // /// The steps that make up this workout.
                                   // pub steps: Vec<Box<dyn WorkoutStep>>,
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
pub fn WorkoutDay(week: WorkoutWeek, today: DateTime<Local>, day: Weekday) -> impl IntoView {
    view! {
        <div class="col s1 center-align">
            <div class="row" style="margin-bottom:0px;">
                <div class=move || {
                    format!(
                        "col s12 white-text {}",
                        if week.week == today.iso_week() && today.weekday() == day {
                            "blue darken-2"
                        } else {
                            "indigo darken-1"
                        },
                    )
                }>
                    {move || {
                        format!(
                            "{}",
                            NaiveDate::from_isoywd_opt(week.week.year(), week.week.week(), day)
                                .unwrap()
                                .format("%d"),
                        )
                    }}

                </div>
            </div>
            <div class="row">
                <div class="col s12">
                    {week
                        .workouts
                        .get(&day)
                        .map(|w| {
                            view! {
                                <div class="collection">
                                    {w
                                        .into_iter()
                                        .map(|e| view! { <li class="collection-item">{e}</li> })
                                        .collect::<Vec<_>>()}

                                </div>
                            }
                        })}

                </div>
            </div>

        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WorkoutInstance {
    id: i64,
    user_id: i64,
    start_date: DateTime<Local>,
    rrule: String,
    active: bool,
    template: WorkoutTemplate,
}
#[cfg(feature = "ssr")]
impl sqlx::FromRow<'_, PgRow> for WorkoutInstance {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        let template = WorkoutTemplate {
            id: row.get::<(i64, i32, String, String), &str>("template").0,
            user_id: row.get::<(i64, i32, String, String), &str>("template").1,
            template_name: row.get::<(i64, i32, String, String), &str>("template").2,
            workout_type: WorkoutType::from_str(
                &row.get::<(i64, i32, String, String), &str>("template").3,
            )
            .unwrap(),
        };
        Ok(Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            start_date: row.get("start_date"),
            rrule: row.get("rrule"),
            active: row.get("active"),
            template: template,
        })
    }
}

#[server]
pub async fn set_week_scaling(year: i32, week: i32, scaling: i32) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let result = sqlx::query!(
        r#"
            SELECT id
            FROM weekly_scaling
            WHERE user_id=$1::bigint and year=$2::int and week=$3::int
        "#,
        user.id as _,
        year,
        week
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't load weekly scaling: {}", e)))?;
    if let Some(res) = result {
        sqlx::query!(
            r#"UPDATE weekly_scaling
        SET scaling=$2
        WHERE id=$1"#,
            res.id,
            scaling
        )
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Couldn't update scaling: {}", e)))?;
    } else {
        sqlx::query!(
            r#"INSERT INTO weekly_scaling (user_id, year, week,scaling)
        VALUES ($1,$2,$3,$4)"#,
            user.id as _,
            year,
            week,
            scaling
        )
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::new(format!("Couldn't save scaling: {}", e)))?;
    }
    Ok(())
}

#[server(GetWorkoutInstances, "/api")]
pub async fn get_workout_instances(
    from: DateTime<Local>,
    to: DateTime<Local>,
) -> Result<Vec<WorkoutInstance>, ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let rrules: Vec<WorkoutInstance> = sqlx::query_as::<_, WorkoutInstance>(
        r#"
            SELECT 
                i.id,
                i.user_id::int8,
                i.start_date,
                i.rrule,
                i.active,
                (
                    t.id, 
                    t.user_id,
                    t.template_name,
                    t.workout_type::text
                ) as template
            FROM workout_instances i
            INNER JOIN workout_templates t ON i.workout_template_id=t.id
            WHERE i.user_id=$1::bigint and i.active and i.start_date < $2
        "#,
    )
    .bind(user.id as i32)
    .bind(to)
    .fetch_all(&pool)
    .await?;
    Ok(rrules)
}

#[derive(Debug, Clone)]
pub struct WorkoutWeek {
    week: IsoWeek,
    workouts: HashMap<Weekday, Vec<String>>,
}
async fn get_week_workouts(week: IsoWeek) -> WorkoutWeek {
    let start = &NaiveDate::from_isoywd_opt(week.year(), week.week(), Weekday::Mon)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let end = &NaiveDate::from_isoywd_opt(week.year(), week.week(), Weekday::Sun)
        .unwrap()
        .checked_add_days(Days::new(1))
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .checked_sub_signed(Duration::milliseconds(1))
        .unwrap();
    let instances = get_workout_instances(
        Local.from_local_datetime(start).unwrap(),
        Local.from_local_datetime(end).unwrap(),
    )
    .await
    .unwrap();
    let mut workouts = HashMap::new();
    for instance in instances {
        let rrule = RRuleSet::new(instance.start_date.with_timezone(&Tz::Local(Local))).rrule(
            instance
                .rrule
                .parse::<RRule<Unvalidated>>()
                .unwrap()
                .validate(instance.start_date.with_timezone(&Tz::Local(Local)))
                .unwrap(),
        );
        let occurences = rrule
            .after(Tz::LOCAL.from_local_datetime(start).unwrap())
            .before(Tz::LOCAL.from_local_datetime(end).unwrap())
            .all_unchecked();
        for occurence in occurences {
            workouts
                .entry(occurence.weekday())
                .and_modify(|o: &mut Vec<_>| o.push(instance.template.template_name.clone()))
                .or_insert(vec![instance.template.template_name.clone()]);
        }
    }
    WorkoutWeek { week, workouts }
}

#[component]
pub fn WorkoutCalendar() -> impl IntoView {
    let today = Local::now();
    let weeks = create_rw_signal(Vec::<WorkoutWeek>::new());
    // spawn_local(async move {
    //     let mut week_entries: Vec<_> = Vec::new();
    //     for w in 0..8 {
    //         week_entries.push(get_week_workouts((today + Duration::weeks(w - 1)).iso_week()).await);
    //     }
    //     weeks.set(week_entries)
    // });
    let calendar_list_el = create_node_ref::<Div>();
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks
                .with_untracked(|wk| wk.last().map(|l| l.week))
                .unwrap_or(today.iso_week());
            let newest =
                NaiveDate::from_isoywd_opt(newest.year(), newest.week(), Weekday::Mon).unwrap();
            let mut week_entries: Vec<_> = Vec::new();
            for w in 1..9 {
                week_entries
                    .push(get_week_workouts((newest + Duration::weeks(w)).iso_week()).await);
            }
            weeks.update(|v| v.extend(week_entries));
        },
        UseInfiniteScrollOptions::default().direction(leptos_use::core::Direction::Bottom),
    );
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks
                .with_untracked(|wk| wk.iter().next().map(|n| n.week))
                .unwrap_or(today.iso_week());
            let newest =
                NaiveDate::from_isoywd_opt(newest.year(), newest.week(), Weekday::Mon).unwrap();
            let new_week = get_week_workouts((newest - Duration::weeks(1)).iso_week()).await;
            weeks.update(|v| {
                *v = iter::once(new_week)
                    .chain((*v).iter().map(|x| x.clone()))
                    .collect();
            });
            if let Some(el) = calendar_list_el.get_untracked() {
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
    let set_scaling = create_server_action::<SetWeekScaling>();
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
                    <For each=move || weeks.get() key=|i| format!("{:?}", i.week) let:item>
                        <div class="calendar-row cal-content">
                            <div class="col s1 center-align valign-wrapper p-6">
                                {item.week.year()} - {item.week.week()}
                            </div>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Mon/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Tue/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Wed/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Thu/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Fri/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Sat/>
                            <WorkoutDay week=item.clone() today=today day=Weekday::Sun/>
                            <div class="col s1 center-align">
                                <select
                                    name=format!("load-{}-{}", item.week.year(), item.week.week())
                                    id=format!("load-{}-{}", item.week.year(), item.week.week())
                                    style="display:block;"
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev).parse::<i32>();
                                        if let Ok(val) = val {
                                            set_scaling
                                                .dispatch(SetWeekScaling {
                                                    year: item.week.year(),
                                                    week: item.week.week().try_into().unwrap(),
                                                    scaling: val,
                                                });
                                        }
                                    }
                                >

                                    <option value="-50">-50%</option>
                                    <option value="-45">-45%</option>
                                    <option value="-40">-40%</option>
                                    <option value="-35">-35%</option>
                                    <option value="-30">-30%</option>
                                    <option value="-25">-25%</option>
                                    <option value="-20">-20%</option>
                                    <option value="-15">-15%</option>
                                    <option value="-10">-10%</option>
                                    <option value="-5">-5%</option>
                                    <option value="0" selected>
                                        0%
                                    </option>
                                    <option value="5">5%</option>
                                    <option value="10">10%</option>
                                    <option value="15">15%</option>
                                    <option value="20">20%</option>
                                    <option value="25">25%</option>
                                    <option value="25">25%</option>
                                    <option value="30">30%</option>
                                    <option value="35">35%</option>
                                    <option value="40">40%</option>
                                    <option value="45">45%</option>
                                    <option value="50">50%</option>
                                </select>
                            </div>

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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "workout_type", rename_all = "snake_case")
)]
#[derive(strum::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum WorkoutType {
    Run,
    Strength,
    Cycling,
    Hiking,
    Endurance,
}

#[cfg(feature = "ssr")]
impl TryFrom<String> for WorkoutType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "run" => Ok(Self::Run),
            "strength" => Ok(Self::Strength),
            "cycling" => Ok(Self::Cycling),
            "hiking" => Ok(Self::Hiking),
            "endurance" => Ok(Self::Endurance),
            _ => Err("Couldn't parse workout type".to_string()),
        }
    }
}
