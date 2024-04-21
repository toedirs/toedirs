use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    iter,
};

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use crate::workout_schedule::{
    add_template_dialog::CreateWorkoutDialog, add_workout_dialog::AddWorkoutDialog,
};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Weekday};
#[cfg(feature = "ssr")]
use chrono::{IsoWeek, TimeZone};
use humantime::format_duration;
use leptos::{html::Div, *};
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use rrule::{RRule, Validated};
#[cfg(feature = "ssr")]
use rrule::{RRuleSet, Tz, Unvalidated};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
#[cfg(feature = "ssr")]
use std::str::FromStr;

use self::add_workout_dialog::WorkoutParameter;

pub mod add_template_dialog;
pub mod add_workout_dialog;

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

/// A scaling entry for a week of a workout plan.
/// Defines how the volume of workouts should be increased\decreased during this week.
pub struct ScalingEntry {
    /// first day of week this entry is for.
    pub date: DateTime<Local>,
    /// The scaling factor in additive percent.
    pub factor: i32,
}

#[server]
pub async fn delete_workout_instance(instance_id: i64) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    sqlx::query!(
        r#"
        DELETE FROM workout_instances
        WHERE user_id=$1 and id=$2
        "#,
        user.id as _,
        instance_id
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't delete instance: {}", e)))?;

    Ok(())
}

#[server]
pub async fn delete_workout_occurence(
    instance_id: i64,
    week: (i32, u32),
    day: Weekday,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let _ = sqlx::query!(
        r#"
            SELECT 
                i.id
            FROM workout_instances i
            WHERE i.user_id=$1::bigint and i.id=$2
        "#,
        user.id as i32,
        instance_id
    )
    .fetch_one(&pool)
    .await?;

    let date = NaiveDate::from_isoywd_opt(week.0, week.1, day).unwrap();
    let date = Local
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO workout_exclusion_dates(workout_instance_id, exclusion_date)
        VALUES($1,$2)
        "#,
        instance_id as _,
        date
    )
    .execute(&pool)
    .await?;

    Ok(())
}

#[component]
pub fn WorkoutDay(
    week: WorkoutWeek,
    today: DateTime<Local>,
    day: Weekday,
    #[prop(into)] on_change: Callback<()>,
) -> impl IntoView {
    let delete_instance = create_server_action::<DeleteWorkoutInstance>();
    let delete_occurence = create_server_action::<DeleteWorkoutOccurence>();
    create_effect(move |_| {
        // run callback if server action was run
        if let Some(_) = delete_instance.value().get() {
            on_change(());
        }
    });
    create_effect(move |_| {
        // run callback if server action was run
        if let Some(_) = delete_occurence.value().get() {
            on_change(());
        }
    });
    view! {
        <div class="column">
            <div class="columns" style="margin-bottom:0px;">
                <div class=move || {
                    format!(
                        "column has-text-white {}",
                        if week.week.0 == today.iso_week().year()
                            && week.week.1 == today.iso_week().week() && today.weekday() == day
                        {
                            "has-background-primary"
                        } else {
                            "has-background-link"
                        },
                    )
                }>
                    {move || {
                        format!(
                            "{}",
                            NaiveDate::from_isoywd_opt(week.week.0, week.week.1, day)
                                .unwrap()
                                .format("%d"),
                        )
                    }}

                </div>
            </div>
            <div class="columns">
                <div class="column is-full">
                    {week
                        .workouts
                        .get(&day)
                        .map(|w| {
                            let w = w.clone();
                            view! {
                                <div class="columns is-multiline center-align">
                                    {w
                                        .into_iter()
                                        .map(|e| {
                                            let show_info = create_rw_signal(false);
                                            let mut s = DefaultHasher::new();
                                            e.hash(&mut s);
                                            let color = s.finish() % 360;
                                            view! {
                                                <div class="column is-full center-align valign-wrapper">
                                                    <div
                                                        class="box center-align valign-wrapper"
                                                        style=format!(
                                                            "position:relative;background-color:hsl({},80%,85%)",
                                                            color,
                                                        )
                                                    >

                                                        {e.name.clone()}
                                                        <i
                                                            class="material-symbols-rounded"
                                                            on:mouseover=move |_| show_info.set(true)
                                                            on:mouseout=move |_| show_info.set(false)
                                                        >
                                                            info
                                                        </i>
                                                        <div class="dropdown is-hoverable">
                                                            <div class="dropdown-trigger" style="width:50px;">
                                                                <i
                                                                    aria-haspopup="true"
                                                                    aria-controls=format!("workout-dropdown-{}", e.id)
                                                                    class="material-symbols-rounded"
                                                                >

                                                                    arrow_drop_down
                                                                </i>
                                                            </div>
                                                            <div
                                                                class="dropdown-menu"
                                                                id=format!("workout-dropdown-{}", e.id)
                                                                role="menu"
                                                            >
                                                                <div class="dropdown-content">
                                                                    <a
                                                                        href="#"
                                                                        class="dropdown-item"
                                                                        on:click=move |_| {
                                                                            delete_instance
                                                                                .dispatch(DeleteWorkoutInstance {
                                                                                    instance_id: e.id,
                                                                                });
                                                                        }
                                                                    >

                                                                        Delete All
                                                                    </a>
                                                                    <a
                                                                        href="#"
                                                                        class="dropdown-item"
                                                                        on:click=move |_| {
                                                                            delete_occurence
                                                                                .dispatch(DeleteWorkoutOccurence {
                                                                                    instance_id: e.id,
                                                                                    week: week.week,
                                                                                    day: day,
                                                                                });
                                                                        }
                                                                    >

                                                                        Delete Occurence
                                                                    </a>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <Show when=move || { show_info() } fallback=|| {}>
                                                            <div class="tooltip-wrapper">
                                                                <div class="tooltip">

                                                                    {
                                                                        let mut steps = e.steps.clone();
                                                                        steps.sort_by(|a, b| a.position.cmp(&b.position));
                                                                        steps
                                                                            .iter()
                                                                            .map(|s| {
                                                                                view! {
                                                                                    <div class="columns">
                                                                                        <div class="column">{s.name.clone()}</div>
                                                                                        <div class="column">
                                                                                            {match s.param_type.as_str() {
                                                                                                "time_s" => {
                                                                                                    format_duration(
                                                                                                            std::time::Duration::new(s.value.clone() as _, 0),
                                                                                                        )
                                                                                                        .to_string()
                                                                                                }
                                                                                                _ => s.value.clone().to_string(),
                                                                                            }}

                                                                                        </div>
                                                                                        <div class="column">{s.param_type.clone()}</div>
                                                                                    </div>
                                                                                }
                                                                            })
                                                                            .collect::<Vec<_>>()
                                                                    }

                                                                </div>
                                                            </div>
                                                        </Show>
                                                    </div>
                                                </div>
                                            }
                                        })
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
pub struct WorkoutInstance {
    id: i64,
    user_id: i64,
    start_date: DateTime<Local>,
    rrule: String,
    active: bool,
    template: WorkoutTemplate,
    exclusion_dates: Vec<DateTime<Local>>,
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
            exclusion_dates: row.try_get("exclusion_dates").unwrap_or_default(),
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

#[cfg(feature = "ssr")]
pub async fn get_week_scaling(
    from: IsoWeek,
    to: IsoWeek,
) -> Result<HashMap<IsoWeek, i32>, ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;

    let result = sqlx::query!(
        r#"WITH weeks as (
            SELECT generate_series(
                date_trunc('week', $2::date),
                date_trunc('week', $3::date),
                '1 week'
            ) as start
        )
        SELECT COALESCE(weekly_scaling.scaling,0) as scaling, EXTRACT(year from weeks.start)::int4 as year, EXTRACT(week from weeks.start)::int4 as week
        FROM weeks
        LEFT JOIN weekly_scaling on weekly_scaling.year=EXTRACT(year from weeks.start) and weekly_scaling.week=EXTRACT(week from weeks.start)
         and user_id=$1::bigint 
        "#,
        user.id as _,
        NaiveDate::from_isoywd_opt(from.year(),from.week(),Weekday::Mon).unwrap(),
        NaiveDate::from_isoywd_opt(to.year(),to.week(),Weekday::Sun).unwrap(),
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't load weekly scaling: {}", e)))?;
    Ok(result
        .iter()
        .map(|r| {
            (
                NaiveDate::from_isoywd_opt(
                    r.year.unwrap(),
                    r.week.unwrap().try_into().unwrap(),
                    Weekday::Mon,
                )
                .unwrap()
                .iso_week(),
                r.scaling.unwrap(),
            )
        })
        .collect())
}

#[cfg(feature = "ssr")]
pub async fn get_workout_instances(
    _from: DateTime<Local>,
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
                ) as template,
                ARRAY_AGG(ex.exclusion_date) as exclusion_dates
            FROM workout_instances i
            INNER JOIN workout_templates t ON i.workout_template_id=t.id
            LEFT JOIN workout_exclusion_dates ex ON ex.workout_instance_id=i.id
            WHERE i.user_id=$1::bigint and i.active and i.start_date < $2
            GROUP BY i.id, i.user_id, i.start_date, i.rrule, i.active, t.id, t.user_id, t.template_name, t.workout_type
        "#,
    )
    .bind(user.id as i32)
    .bind(to)
    .fetch_all(&pool)
    .await?;
    Ok(rrules)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutInstanceWithScaling {
    id: i64,
    parameters: Vec<WorkoutParameter>,
    scaling: HashMap<String, f64>,
}
#[cfg(feature = "ssr")]
pub async fn get_instance_steps_with_scaling(
    instance_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<WorkoutInstanceWithScaling, ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let result = sqlx::query!(
        r#"WITH weeks as (
            SELECT generate_series(
                date_trunc('week', $1::date),
                date_trunc('week', $2::date),
                '1 week'
            ) as start
        )
        SELECT
            EXTRACT(year FROM weeks.start)::int as year,
            EXTRACT(week FROM weeks.start)::int as week,
             SUM(COALESCE(s.scaling,0)) OVER (ORDER BY EXTRACT(year FROM weeks.start),EXTRACT(week FROM weeks.start) )::float as scaling
        FROM weeks
        LEFT JOIN (
            SELECT year, week, user_id,
                CASE WHEN year=$3 and week=$4 THEN -- ignore scaling on first week
                    0
                ELSE
                    scaling
                END as scaling
            FROM weekly_scaling
            ) s ON s.year=EXTRACT(year FROM weeks.start) and s.week=EXTRACT(week FROM weeks.start)
        WHERE s.user_id IS NULL or s.user_id=$5"#,
        from,
        to,
        from.iso_week().year() as _,
        from.iso_week().week() as i32,
        user.id as _,
    ).fetch_all(&pool).await.map_err(|e|ServerFnError::new(format!("Couldn't load scaling: {}",e)))?;
    let scaling: HashMap<String, f64> = result
        .iter()
        .map(|r| {
            (
                format!("{}-{}", r.year.unwrap(), r.week.unwrap()),
                (r.scaling.unwrap() + 100.) / 100.,
            )
        })
        .collect();
    let result = sqlx::query!(
        r#"SELECT 
            p.id,
            p.name,
            COALESCE(l.value_override,p.value) as value,
            p.parameter_type::text,
            p.scaling,
            p.position
        FROM workout_instances i
        INNER JOIN workout_templates t ON i.workout_template_id=t.id
        INNER JOIN workout_parameters p ON p.workout_template_id=t.id
        LEFT JOIN parameter_links l ON l.parameter_id=p.id and l.instance_id=i.id
        WHERE i.id=$1 and i.user_id=$2"#,
        instance_id,
        user.id as _
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Couldn't get workout parameters: {}", e)))?;
    let parameters: Vec<_> = result
        .iter()
        .map(|r| WorkoutParameter {
            id: r.id,
            name: r.name.clone(),
            value: r.value.expect("value not found on parameter"),
            parameter_type: r.parameter_type.clone().unwrap(),
            scaling: r.scaling,
            position: r.position,
        })
        .collect();
    Ok(WorkoutInstanceWithScaling {
        id: instance_id,
        parameters,
        scaling,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutStep {
    name: String,
    value: i32,
    param_type: String,
    position: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workout {
    id: i64,
    name: String,
    steps: Vec<WorkoutStep>,
}

impl Hash for Workout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutWeek {
    week: (i32, u32),
    workouts: HashMap<Weekday, Vec<Workout>>,
    scaling: i32,
}
impl WorkoutWeek {
    pub fn key(&self) -> (i32, u32, usize, i32, usize) {
        (
            self.week.0,
            self.week.1,
            self.workouts.iter().map(|(_, v)| v.len()).sum(),
            self.scaling,
            self.workouts
                .iter()
                .map(|(_, v)| {
                    v.iter()
                        .map(|e| e.steps.iter().map(|s| s.value as usize).sum::<usize>())
                        .sum::<usize>()
                })
                .sum::<usize>(),
        )
    }
}
#[server]
pub async fn get_week_workouts(
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<WorkoutWeek>, ServerFnError> {
    let auth = auth()?;
    let _user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;

    let instances = get_workout_instances(
        Local
            .from_local_datetime(&from.and_hms_opt(0, 0, 0).unwrap())
            .unwrap(),
        Local
            .from_local_datetime(&to.and_hms_opt(0, 0, 0).unwrap())
            .unwrap(),
    )
    .await
    .unwrap();
    let scalings = get_week_scaling(from.iso_week(), to.iso_week())
        .await
        .unwrap();
    let mut weeks: HashMap<IsoWeek, HashMap<Weekday, Vec<Workout>>> = HashMap::new();
    // ensure each week has an entry
    for scaling in scalings.keys() {
        weeks.entry(*scaling).or_default();
    }

    for instance in instances {
        let rrule = RRuleSet::new(instance.start_date.with_timezone(&Tz::Local(Local)))
            .rrule(
                instance
                    .rrule
                    .parse::<RRule<Unvalidated>>()
                    .unwrap()
                    .validate(instance.start_date.with_timezone(&Tz::Local(Local)))
                    .unwrap(),
            )
            .set_exdates(
                instance
                    .exclusion_dates
                    .iter()
                    .map(|d| d.with_timezone(&Tz::Local(Local)))
                    .collect(),
            );
        let steps_and_scaling = get_instance_steps_with_scaling(
            instance.id,
            rrule
                .into_iter()
                .next()
                .map(|d| d.date_naive())
                .unwrap_or(instance.start_date.date_naive()),
            to,
        )
        .await
        .unwrap();
        let occurences = rrule
            .after(
                Tz::LOCAL
                    .from_local_datetime(&from.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap(),
            )
            .before(
                Tz::LOCAL
                    .from_local_datetime(&to.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap(),
            )
            .all_unchecked();
        for occurence in occurences {
            let steps: Vec<WorkoutStep> = steps_and_scaling
                .parameters
                .iter()
                .map(|p| WorkoutStep {
                    name: p.name.clone(),
                    param_type: p.parameter_type.clone(),
                    position: p.position,
                    value: if p.scaling {
                        (p.value as f64
                            * steps_and_scaling
                                .scaling
                                .get(&format!(
                                    "{}-{}",
                                    occurence.iso_week().year(),
                                    occurence.iso_week().week() as i32,
                                ))
                                .unwrap())
                        .round() as i32
                    } else {
                        p.value
                    },
                })
                .collect();
            let workout = Workout {
                id: instance.id,
                name: instance.template.template_name.clone(),
                steps,
            };

            weeks
                .entry(occurence.iso_week())
                .or_default()
                .entry(occurence.weekday())
                .or_default()
                .push(workout);
        }
    }
    let mut result: Vec<WorkoutWeek> = weeks
        .iter()
        .map(|(week, m)| WorkoutWeek {
            week: (week.year(), week.week()),
            workouts: m.clone(),
            scaling: *scalings.get(&week).unwrap_or(&0),
        })
        .collect();
    result.sort_by(|a, b| a.week.partial_cmp(&b.week).unwrap());
    Ok(result)
}

#[component]
pub fn WorkoutCalendar() -> impl IntoView {
    let today = Local::now();
    let weeks = create_rw_signal(Vec::<WorkoutWeek>::new());
    let calendar_list_el = create_node_ref::<Div>();
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks
                .with_untracked(|wk| wk.last().map(|l| l.week))
                .unwrap_or((today.iso_week().year(), today.iso_week().week() - 1));
            let newest = NaiveDate::from_isoywd_opt(newest.0, newest.1, Weekday::Mon).unwrap();

            let week_entries = get_week_workouts(
                newest + Duration::try_weeks(1).unwrap(),
                newest + Duration::try_weeks(9).unwrap(),
            )
            .await;
            if let Ok(week_entries) = week_entries {
                weeks.update(|v| v.extend(week_entries));
            }
        },
        UseInfiniteScrollOptions::default().direction(leptos_use::core::Direction::Bottom),
    );
    let _ = use_infinite_scroll_with_options(
        calendar_list_el,
        move |_| async move {
            let newest = weeks
                .with_untracked(|wk| wk.iter().next().map(|n| n.week))
                .unwrap_or((today.iso_week().year(), today.iso_week().week() - 1));
            let newest = NaiveDate::from_isoywd_opt(newest.0, newest.1, Weekday::Mon).unwrap();
            let week_entries = get_week_workouts(
                newest - Duration::try_weeks(1).unwrap(),
                newest - Duration::try_days(1).unwrap(),
            )
            .await;
            if let Some(new_week) = week_entries
                .ok()
                .and_then(|w| w.first().and_then(|f| Some(f.to_owned())))
            {
                weeks.update(|v| {
                    *v = iter::once(new_week)
                        .chain((*v).iter().map(|x| x.clone()))
                        .collect();
                });
                if let Some(el) = calendar_list_el.get_untracked() {
                    el.set_scroll_top(150);
                }
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
    let show_add_workout = create_rw_signal(false);
    let show_create_workout = create_rw_signal(false);

    let reload_calendar = move |_| {
        spawn_local(async move {
            let date_range =
                weeks.with_untracked(|wk| (wk.first().map(|w| w.week), wk.last().map(|w| w.week)));
            if let (Some(start), Some(end)) = date_range {
                let week_entries = get_week_workouts(
                    NaiveDate::from_isoywd_opt(start.0, start.1, Weekday::Mon).unwrap(),
                    NaiveDate::from_isoywd_opt(end.0, end.1, Weekday::Sun).unwrap(),
                )
                .await;
                if let Ok(week_entries) = week_entries {
                    weeks.update(|v| *v = week_entries);
                }
            }
        })
    };
    create_effect(move |_| {
        // run callback if server action was run
        if let Some(_) = set_scaling.value().get() {
            reload_calendar(());
        }
    });
    view! {
        <div class="workout-calendar">
            <div class="calendar-row calendar-header white-text blue darken-1">
                <div class="col center-align">
                    <h5>Week</h5>
                </div>
                <div class="col center-align">
                    <h5>Mon</h5>
                </div>
                <div class="col center-align">
                    <h5>Tue</h5>
                </div>
                <div class="col center-align">
                    <h5>Wed</h5>
                </div>
                <div class="col center-align">
                    <h5>Thu</h5>
                </div>
                <div class="col center-align">
                    <h5>Fri</h5>
                </div>
                <div class="col center-align">
                    <h5>Sat</h5>
                </div>
                <div class="col center-align">
                    <h5>Sun</h5>
                </div>
                <div class="col center-align">
                    <h5>Load</h5>
                </div>
            </div>
            <div class="calendar-container" node_ref=calendar_list_el>

                <div class="calendar-body">
                    <For each=move || weeks.get() key=|i| i.key() let:item>
                        <div class="calendar-row cal-content">
                            <div class="column center-align valign-wrapper">
                                {item.week.0} - {item.week.1}
                            </div>
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Mon
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Tue
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Wed
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Thu
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Fri
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Sat
                                on_change=reload_calendar
                            />
                            <WorkoutDay
                                week=item.clone()
                                today=today
                                day=Weekday::Sun
                                on_change=reload_calendar
                            />
                            <div class="column field">
                                <div class="control select">
                                    <select
                                        name=format!("load-{}-{}", item.week.0, item.week.1)

                                        id=format!("load-{}-{}", item.week.0, item.week.1)
                                        style="display:block;"
                                        on:input=move |ev| {
                                            let val = event_target_value(&ev).parse::<i32>();
                                            if let Ok(val) = val {
                                                set_scaling
                                                    .dispatch(SetWeekScaling {
                                                        year: item.week.0,
                                                        week: item.week.1.try_into().unwrap(),
                                                        scaling: val,
                                                    });
                                            }
                                        }
                                    >

                                        <option value="-50" selected=item.scaling == -50>
                                            -50%
                                        </option>
                                        <option value="-45" selected=item.scaling == -45>
                                            -45%
                                        </option>
                                        <option value="-40" selected=item.scaling == -40>
                                            -40%
                                        </option>
                                        <option value="-35" selected=item.scaling == -35>
                                            -35%
                                        </option>
                                        <option value="-30" selected=item.scaling == -30>
                                            -30%
                                        </option>
                                        <option value="-25" selected=item.scaling == -25>
                                            -25%
                                        </option>
                                        <option value="-20" selected=item.scaling == -20>
                                            -20%
                                        </option>
                                        <option value="-15" selected=item.scaling == -15>
                                            -15%
                                        </option>
                                        <option value="-10" selected=item.scaling == -10>
                                            -10%
                                        </option>
                                        <option value="-5" selected=item.scaling == -5>
                                            -5%
                                        </option>
                                        <option value="0" selected=item.scaling == -0>
                                            0%
                                        </option>
                                        <option value="5" selected=item.scaling == 5>
                                            5%
                                        </option>
                                        <option value="10" selected=item.scaling == 10>
                                            10%
                                        </option>
                                        <option value="15" selected=item.scaling == 15>
                                            15%
                                        </option>
                                        <option value="20" selected=item.scaling == 20>
                                            20%
                                        </option>
                                        <option value="25" selected=item.scaling == 25>
                                            25%
                                        </option>
                                        <option value="30" selected=item.scaling == 30>
                                            30%
                                        </option>
                                        <option value="35" selected=item.scaling == 35>
                                            35%
                                        </option>
                                        <option value="40" selected=item.scaling == 40>
                                            40%
                                        </option>
                                        <option value="45" selected=item.scaling == 45>
                                            45%
                                        </option>
                                        <option value="50" selected=item.scaling == 50>
                                            50%
                                        </option>
                                    </select>
                                </div>
                            </div>

                        </div>
                    </For>
                </div>
            </div>
            <div class="is-fab dropdown is-hoverable is-up">
                <span
                    class="icon is-large has-text-primary dropdown-trigger"
                    aria-haspopup="true"
                    aria-controls="calendar-action-menu"
                >
                    <i class="fas fa-plus-circle fa-3x"></i>
                </span>

                <div class="dropdown-menu" id="calendar-action-menu" role="menu">
                    <div class="dropdown-content">
                        <a
                            class="button dropdown-item"
                            alt="Add workout template"
                            on:click=move |_| { show_create_workout.set(true) }
                        >
                            <span class="icon is-small">
                                <i class="fas fa-dumbbell"></i>
                            </span>
                            <span>Add Template</span>
                        </a>
                        <a
                            class="button dropdown-item"
                            alt="Add workout entry"
                            on:click=move |_| { show_add_workout.set(true) }
                        >
                            <span class="icon is-small">
                                <i class="fas fa-calendar"></i>
                            </span>
                            <span>Add Workout Entry</span>
                        </a>
                    </div>
                </div>
            </div>
            <CreateWorkoutDialog show=show_create_workout/>
            <AddWorkoutDialog show=show_add_workout on_save=reload_calendar/>
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
