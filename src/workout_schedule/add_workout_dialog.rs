use std::{collections::HashSet, error::Error, iter};

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use chrono::{DateTime, Local, TimeZone, Weekday};
use leptos::{ev::SubmitEvent, *};
use leptos_router::*;
use rrule::{NWeekday, RRule, Tz};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
use strum;
use thaw::*;

use crate::elements::select::Select;

use super::WorkoutType;

#[derive(Debug, Serialize, Deserialize, Clone)]
// #[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct WorkoutParameter {
    pub id: i64,
    pub name: String,
    pub value: i32,
    pub parameter_type: String,
    pub scaling: bool,
    pub position: i32,
}
#[cfg(feature = "ssr")]
impl sqlx::Type<Postgres> for WorkoutParameter {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("workout_parameter")
    }
}
#[cfg(feature = "ssr")]
impl<'r> Decode<'r, Postgres> for WorkoutParameter {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        let mut decoder = postgres::types::PgRecordDecoder::new(value)?;
        let id = decoder.try_decode::<i64>()?;
        let name = decoder.try_decode::<String>()?;
        let val = decoder.try_decode::<i32>()?;
        let parameter_type = decoder.try_decode::<String>()?;
        let scaling = decoder.try_decode::<bool>()?;
        let position = decoder.try_decode::<i32>()?;
        Ok(Self {
            id,
            name,
            value: val,
            parameter_type,
            scaling,
            position,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WorkoutParameterArray(Vec<WorkoutParameter>);

#[cfg(feature = "ssr")]
impl sqlx::Type<Postgres> for WorkoutParameterArray {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_workout_parameter")
    }
}
#[cfg(feature = "ssr")]
impl<'r> Decode<'r, Postgres> for WorkoutParameterArray {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn Error + 'static + Send + Sync>> {
        Ok(Self(Vec::<WorkoutParameter>::decode(value)?))
    }
}

// #[cfg(feature = "ssr")]
// impl PgHasArrayType for WorkoutParameter {
//     fn array_type_info() -> PgTypeInfo {
//         PgTypeInfo::with_name("_workout_parameter")
//     }
// }

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
    pub parameters: Option<WorkoutParameterArray>,
}

#[server(GetWorkoutTemplates, "/api")]
pub async fn get_workout_templates() -> Result<Vec<WorkoutTemplate>, ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let templates:Vec<WorkoutTemplate> = sqlx::query_as(
        r#"
        SELECT templates.id,
            templates.user_id,
            templates.template_name,
            templates.workout_type,
            ARRAY_AGG(ROW(params.id, params.name, params.value, params.parameter_type::TEXT, params.scaling, params.position)::workout_parameter) as "parameters" 
        FROM workout_templates as templates 
        INNER JOIN workout_parameter as params ON params.workout_template_id = templates.id
        WHERE templates.user_id = $1::bigint
        GROUP BY templates.id"#
    )
        .bind(user.id)
    .fetch_all(&pool)
    .await?;
    Ok(templates)
}

#[server(AddWorkout, "/api")]
pub async fn add_workout(
    workout_type: i32,
    start_date: DateTime<Local>,
    rrule: String,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    sqlx::query!(
        r#"INSERT INTO workout_instances (user_id, workout_template_id, start_date, rrule)
        VALUES ($1,$2,$3,$4)
        "#,
        user.id as _,
        workout_type,
        start_date,
        rrule
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Error saving workout template: {}", e)))?;
    Ok(())
}

#[derive(Clone, PartialEq)]
pub enum EndType {
    Occurences,
    EndDate,
}

#[component]
pub fn AddWorkoutDialog(show: RwSignal<bool>) -> impl IntoView {
    let workout_templates = create_resource(show, |value| async move {
        if value {
            get_workout_templates().await.unwrap_or_default()
        } else {
            Vec::new()
        }
    });
    let workout_type = create_rw_signal("0".to_string());
    let start_date = create_rw_signal(Some(Local::now().date_naive()));
    let end_date = create_rw_signal(Some(Local::now().date_naive()));
    let occurences = create_rw_signal(1);
    let end_type = create_rw_signal(EndType::Occurences);
    let repetition_type = create_rw_signal("weekly".to_string());
    let repetition_frequency = create_rw_signal(1);
    let repetition_on_day = create_rw_signal(HashSet::<String>::new());
    let month_day = create_rw_signal(1);
    let repetition_rule = create_resource(
        move || {
            (
                start_date.get(),
                end_date.get(),
                occurences.get(),
                end_type.get(),
                repetition_type.get(),
                repetition_frequency.get(),
                repetition_on_day.get(),
                month_day.get(),
            )
        },
        move |(
            start,
            end,
            occurences,
            end_type,
            repetition_type,
            repetition_freq,
            repetition_on_day,
            month_day,
        )| async move {
            let mut rrule = match repetition_type.as_str() {
                "daily" => RRule::new(rrule::Frequency::Daily),
                "weekly" => RRule::new(rrule::Frequency::Weekly),
                "monthly" => RRule::new(rrule::Frequency::Monthly),
                _ => unreachable!(),
            };
            rrule = match end_type {
                EndType::Occurences => rrule.count(occurences),
                EndType::EndDate => rrule.until(
                    Tz::LOCAL
                        .from_local_datetime(&end.unwrap().and_hms_opt(0, 0, 0).unwrap())
                        .unwrap(),
                ),
            };
            rrule = rrule.interval(repetition_freq);
            rrule = match rrule.get_freq() {
                rrule::Frequency::Monthly => rrule.by_month_day(vec![month_day]),
                rrule::Frequency::Weekly => {
                    let days: Vec<_> = repetition_on_day
                        .iter()
                        .map(|d| NWeekday::Every(d.parse::<Weekday>().unwrap()))
                        .collect();
                    rrule.by_weekday(days)
                }
                rrule::Frequency::Daily => rrule,
                _ => unreachable!(),
            };
            rrule
                .validate(
                    Tz::LOCAL
                        .from_local_datetime(&start.unwrap().and_hms_opt(0, 0, 0).unwrap())
                        .unwrap(),
                )
                .map(|r| r.to_string())
                .unwrap_or("".to_string())
        },
    );
    let add_workout_action = create_server_action::<AddWorkout>();

    view! {
        <Show when=move || { show() } fallback=|| {}>
            <Form
                action=""
                on:submit=move |ev: SubmitEvent| {
                    add_workout_action
                        .dispatch(AddWorkout {
                            workout_type: workout_type.get_untracked().parse::<i32>().unwrap(),
                            start_date: start_date()
                                .map(|d| {
                                    Local
                                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                                        .unwrap()
                                })
                                .unwrap(),
                            rrule: repetition_rule.get().unwrap(),
                        });
                    show.set(false);
                    ev.prevent_default();
                }
            >

                <div class="modal" style="z-index: 1003;">
                    <div class="modal-header">
                        <h4 class="black-text">"Add workout to calendar"</h4>
                    </div>
                    <div class="modal-body">
                        <div class="modal-content" style="overflow:scroll;">
                            <div class="row"></div>
                            <div class="row">
                                <Suspense fallback=move || view! { "loading..." }>
                                    <div class="col s6 input-field">
                                        // <select name="workout_type" id="workout_type">
                                        {move || {
                                            workout_templates
                                                .get()
                                                .map(|templates| {
                                                    let options = iter::once((
                                                            "0".to_string(),
                                                            "Choose workout template".to_string(),
                                                            true,
                                                        ))
                                                        .chain(
                                                            templates
                                                                .iter()
                                                                .map(|t| {
                                                                    (format!("{}", t.id), t.template_name.clone(), false)
                                                                }),
                                                        )
                                                        .collect::<Vec<_>>();
                                                    view! {
                                                        <Select
                                                            value=workout_type
                                                            name="workout_type".to_string()
                                                            options=Some(options)
                                                            attr:id="workout_templ"
                                                        >

                                                            {}
                                                        </Select>
                                                        <label for="workout_templ">Workout Template</label>
                                                    }
                                                })
                                        }}

                                    </div>
                                </Suspense>
                            </div>
                            <div class="row">
                                <input value=repetition_rule/>
                            </div>
                            <div class="row">
                                <div class="col s6 input-field">
                                    <DatePicker value=start_date attr:id="start_date"/>
                                    <label for="start_date">Start Date</label>
                                </div>
                            </div>
                            <div class="row">
                                <div class="col s2 input-field valign-wrapper">
                                    <span>"Repeat"</span>
                                </div>
                                <div class="col s2 input-field">
                                    <Select
                                        value=repetition_type
                                        name="repetition_type".to_string()
                                        options=None
                                        attr:id="repetition_type"
                                    >
                                        <option value="daily">Daily</option>
                                        <option value="weekly">Weekly</option>
                                        <option value="monthly">Monthly</option>
                                    </Select>
                                </div>
                                <div class="col s2 input-field valign-wrapper">"every"</div>
                                <div class="col s2 input-field">
                                    <InputNumber value=repetition_frequency step=1/>
                                </div>
                                <div class="col s2 input-field valign-wrapper">
                                    {move || match repetition_type.get().as_str() {
                                        "weekly" => "weeks",
                                        "daily" => "days",
                                        _ => "months",
                                    }}

                                </div>
                            </div>
                            <Show when=move || { repetition_type.get() == "weekly" } fallback=|| {}>
                                <div class="row">
                                    <div class="col s1">"On"</div>
                                    <div class="col s11">
                                        <CheckboxGroup value=repetition_on_day>
                                            <CheckboxItem label="Monday" key="monday"/>
                                            <CheckboxItem label="Tuesday" key="tuesday"/>
                                            <CheckboxItem label="Wednesday" key="wednesday"/>
                                            <CheckboxItem label="Thursday" key="thursday"/>
                                            <CheckboxItem label="Friday" key="friday"/>
                                            <CheckboxItem label="Saturday" key="saturday"/>
                                            <CheckboxItem label="Sunday" key="sunday"/>
                                        </CheckboxGroup>
                                    </div>
                                </div>
                            </Show>
                            <Show
                                when=move || { repetition_type.get() == "monthly" }
                                fallback=|| {}
                            >
                                <div class="row">
                                    <div class="col s1">On day</div>
                                    <div class="col s3 input-field">
                                        <InputNumber value=month_day step=1/>
                                    </div>
                                </div>
                            </Show>
                            <div class="row">
                                <div class="col s6 input-field">
                                    "End" <p>
                                        <label>
                                            <input
                                                name="end"
                                                type="radio"
                                                on:click=move |_| end_type.set(EndType::Occurences)
                                                checked
                                            />
                                            <span>
                                                "After" <InputNumber value=occurences step=1/> "occurences"
                                            </span>
                                        </label>
                                    </p> <p>
                                        <label>
                                            <input
                                                name="end"
                                                type="radio"
                                                on:click=move |_| end_type.set(EndType::EndDate)
                                            />
                                            <span>
                                                "On date" <DatePicker value=end_date attr:id="end_date"/>
                                            </span>
                                        </label>
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button type="submit" class="btn waves-effect waves-light">
                            <i class="material-symbols-rounded right">save</i>
                            Add
                        </button>
                    </div>
                </div>
            </Form>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| { show.set(false) }
            ></div>
        </Show>
    }
}
