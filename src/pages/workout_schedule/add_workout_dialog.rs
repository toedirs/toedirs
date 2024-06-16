use std::{
    collections::{HashMap, HashSet},
    iter,
};

#[cfg(feature = "ssr")]
use crate::app::{auth, pool};
use chrono::{DateTime, Local, NaiveDate, TimeZone, Weekday};
#[cfg(feature = "ssr")]
use itertools::Itertools;
use leptos::{ev::SubmitEvent, *};
use leptos_router::*;
use rrule::{NWeekday, RRule, Tz};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{postgres::*, *};
#[cfg(feature = "ssr")]
use std::str::FromStr;

use super::WorkoutType;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
pub struct WorkoutParameter {
    pub id: i64,
    pub name: String,
    pub value: i32,
    pub parameter_type: String,
    pub scaling: bool,
    pub position: i32,
}
#[cfg(feature = "ssr")]
impl PgHasArrayType for WorkoutParameter {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_record")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub parameters: Vec<WorkoutParameter>,
}

#[cfg(feature = "ssr")]
impl sqlx::FromRow<'_, PgRow> for WorkoutTemplate {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            template_name: row.get("template_name"),
            workout_type: WorkoutType::from_str(&row.get::<&str, _>("workout_type")).unwrap(),
            parameters: row.get::<Vec<WorkoutParameter>, _>("parameters"),
        })
    }
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
            templates.workout_type::text,
            ARRAY_AGG((params.id, params.name, params.value, params.parameter_type::TEXT, params.scaling, params.position) ORDER BY params.position) as "parameters" 
        FROM workout_templates as templates 
        INNER JOIN workout_parameters as params ON params.workout_template_id = templates.id
        WHERE templates.user_id = $1::bigint
        GROUP BY templates.id"#
    )
        .bind(user.id)
    .fetch_all(&pool)
    .await?;
    Ok(templates)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterOverride {
    id: i64,
    value: i32,
}

#[server(AddWorkout, "/api")]
pub async fn add_workout(
    workout_type: i32,
    start_date: DateTime<Local>,
    rrule: String,
    param: Option<Vec<ParameterOverride>>,
) -> Result<(), ServerFnError> {
    let pool = pool()?;
    let auth = auth()?;
    let user = auth
        .current_user
        .ok_or(ServerFnError::new("Not logged in".to_string()))?;
    let result = sqlx::query!(
        r#"INSERT INTO workout_instances (user_id, workout_template_id, start_date, rrule)
        VALUES ($1,$2,$3,$4)
        RETURNING id
        "#,
        user.id as _,
        workout_type,
        start_date,
        rrule
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("Error saving workout template: {}", e)))?;
    if let Some(param) = param {
        if param.len() > 0 {
            let instance_ids: Vec<i64> = std::iter::repeat(result.id).take(param.len()).collect();
            let (param_ids, param_values): (Vec<_>, Vec<_>) =
                param.iter().map(|p| (p.id, p.value)).multiunzip();
            sqlx::query!(
                r#"INSERT INTO parameter_links
        SELECT *
        FROM UNNEST($1::bigint[],$2::bigint[], $3::int[])
        "#,
                &instance_ids[..],
                &param_ids[..],
                &param_values[..]
            )
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Error saving parameter overrides:{}", e)))?;
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq)]
pub enum EndType {
    Occurences,
    EndDate,
}

#[component]
pub fn AddWorkoutDialog(
    show: RwSignal<bool>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let workout_templates = create_rw_signal(Vec::new());
    create_effect(move |_| {
        if show() {
            spawn_local(async move {
                let templates = get_workout_templates().await.unwrap_or(Vec::new());
                workout_templates.set(templates);
            });
        }
    });

    let workout_type = create_rw_signal("0".to_string());
    let parameter_override = create_rw_signal(HashMap::<i64, i32>::new());
    let start_date = create_rw_signal(Some(
        Local::now().date_naive().format("%Y-%m-%d").to_string(),
    ));
    let end_date = create_rw_signal(Some(
        Local::now().date_naive().format("%Y-%m-%d").to_string(),
    ));
    let occurences = create_rw_signal(1);
    let end_type = create_rw_signal(EndType::Occurences);
    let repetition_type = create_rw_signal("weekly".to_string());
    let repetition_frequency = create_rw_signal(1);
    let repetition_on_day = create_rw_signal(HashSet::<String>::new());
    let month_day = create_rw_signal(1);
    let repetition_rule = create_local_resource(
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
                    end.and_then(|d| {
                        NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                            .map(|d| {
                                Tz::LOCAL
                                    .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                                    .unwrap()
                            })
                            .ok()
                    })
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
                    start
                        .and_then(|d| {
                            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                                .map(|d| {
                                    Tz::LOCAL
                                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                                        .unwrap()
                                })
                                .ok()
                        })
                        .unwrap(),
                )
                .map(|r| r.to_string())
                .unwrap_or("".to_string())
        },
    );
    let add_workout_action = create_server_action::<AddWorkout>();
    create_effect(move |_| {
        // run callback if server action was run
        if let Some(_) = add_workout_action.value().get() {
            workout_type.set("0".to_string());
            parameter_override.set(HashMap::<i64, i32>::new());
            spawn_local(async move {
                // if we don't do this in a local spawn, it breaks and only shows a white page, probably because the signals get disposed?
                start_date.set(Some(
                    Local::now().date_naive().format("%Y-%m-%d").to_string(),
                ));
                end_date.set(Some(
                    Local::now().date_naive().format("%Y-%m-%d").to_string(),
                ));
                occurences.set(1);
                end_type.set(EndType::Occurences);
                repetition_type.set("weekly".to_string());
                repetition_frequency.set(1);
                repetition_on_day.set(HashSet::<String>::new());
                month_day.set(1);
            });

            show.set(false);
            on_save(());
        }
    });

    view! {
        <Show when=move || { show.get() } fallback=|| {}>
            <Form
                action=""
                on:submit=move |ev: SubmitEvent| {
                    ev.prevent_default();
                    let start_date = start_date
                        .get_untracked()
                        .and_then(|d| {
                            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                                .map(|d| {
                                    Local
                                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                                        .unwrap()
                                })
                                .ok()
                        })
                        .unwrap();
                    add_workout_action
                        .dispatch(AddWorkout {
                            workout_type: workout_type.get_untracked().parse::<i32>().unwrap(),
                            start_date: start_date,
                            rrule: repetition_rule.get().unwrap(),
                            param: Some(
                                parameter_override
                                    .get_untracked()
                                    .iter()
                                    .map(|(k, v)| ParameterOverride {
                                        id: *k,
                                        value: *v,
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                        });
                }
            >

                <div class=move || format!("modal {}", if show.get() { "is-active" } else { "" })>
                    <div class="modal-background" on:click=move |_| show.set(false)></div>
                    <div class="modal-card">
                        <div class="modal-card-head">
                            <p class="modal-card-title">"Add workout to calendar"</p>
                            <button
                                class="delete"
                                aria-label="close"
                                on:click=move |_| show.set(false)
                            ></button>
                        </div>
                        <div class="modal-card-body">
                            <div class="field">
                                <label class="label" for="workout_templ">
                                    Workout Template
                                </label>
                                <Suspense fallback=move || view! { "loading..." }>
                                    <div class="control">
                                        <div class="select">
                                            <select
                                                name="workout_templ"
                                                value=move || workout_type.get()
                                                on:change=move |ev| {
                                                    workout_type.update(|v| *v = event_target_value(&ev))
                                                }
                                            >

                                                {move || {
                                                    let options = iter::once((
                                                            "0".to_string(),
                                                            "Choose workout template".to_string(),
                                                            true,
                                                        ))
                                                        .chain(
                                                            workout_templates
                                                                .get()
                                                                .iter()
                                                                .map(|t| {
                                                                    (format!("{}", t.id), t.template_name.clone(), false)
                                                                }),
                                                        )
                                                        .collect::<Vec<_>>();
                                                    view! {
                                                        <For
                                                            each=move || options.clone()
                                                            key=move |i| i.0.clone()
                                                            let:item
                                                        >
                                                            <option
                                                                value=item.0.clone()
                                                                disabled=item.2
                                                                selected=move || item.0 == workout_type.get_untracked()
                                                            >
                                                                {item.1}
                                                            </option>
                                                        </For>
                                                    }
                                                }}

                                            </select>

                                        </div>
                                    </div>
                                </Suspense>
                            </div>
                            <div class="columns">
                                <div class="column is-fullwidth">
                                    <Show
                                        when=move || { workout_type.get() != "0" }
                                        fallback=|| view! {}
                                    >
                                        <h4>Steps</h4>
                                        {move || {
                                            workout_templates
                                                .get()
                                                .iter()
                                                .filter(|t| t.id.to_string() == workout_type.get())
                                                .next()
                                                .map(|t| {
                                                    t.parameters
                                                        .iter()
                                                        .enumerate()
                                                        .map(|(i, p)| {
                                                            let pp = p.clone();
                                                            view! {
                                                                {if i != 0 {
                                                                    view! { <div class="divider"></div> }.into_view()
                                                                } else {
                                                                    view! {}.into_view()
                                                                }}

                                                                <div class="field is-grouped">
                                                                    <p class="control is-vcentered">{p.name.clone()}</p>
                                                                    <p class="control is-vcentered">
                                                                        <input
                                                                            class="input"
                                                                            type="number"
                                                                            name=format!("param[{}][value]", i)
                                                                            value=p.value
                                                                            on:input=move |ev| {
                                                                                parameter_override
                                                                                    .update(|h| {
                                                                                        let val = event_target_value(&ev).parse();
                                                                                        if let Ok(val) = val {
                                                                                            h.entry(pp.id).and_modify(|v| *v = val).or_insert(val);
                                                                                        }
                                                                                    })
                                                                            }
                                                                        />

                                                                    </p>
                                                                    <p class="control is-vcentered">
                                                                        {p.parameter_type.clone()}
                                                                    </p>
                                                                    <p class="control is-vcentered">
                                                                        {if p.scaling { "scaling" } else { "" }}
                                                                    </p>
                                                                </div>
                                                            }
                                                        })
                                                        .collect_view()
                                                })
                                        }}

                                    </Show>
                                </div>
                            </div>
                            <div class="field">
                                <label class="label" for="rep_rule">
                                    Repetition Rule
                                </label>
                                <div class="control">
                                    <input class="input" value=repetition_rule/>
                                </div>
                            </div>
                            <div class="field">
                                <label class="label" for="start_date">
                                    Start Date
                                </label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="date"
                                        value=start_date
                                        on:change=move |ev| {
                                            start_date
                                                .update(|v| {
                                                    *v = Some(event_target_value(&ev));
                                                })
                                        }
                                    />

                                </div>
                            </div>
                            <div class="field is-grouped">
                                <p class="control">
                                    <span>"Repeat"</span>
                                </p>
                                <p class="control">
                                    <div class="select">
                                        <select
                                            value=repetition_type
                                            name="repetition_type"
                                            id="repetition_type"
                                            on:input=move |ev| {
                                                repetition_type.set(event_target_value(&ev))
                                            }
                                        >

                                            <option value="daily" selected=repetition_type() == "daily">
                                                Daily
                                            </option>
                                            <option
                                                value="weekly"
                                                selected=repetition_type() == "weekly"
                                            >
                                                Weekly
                                            </option>
                                            <option
                                                value="monthly"
                                                selected=repetition_type() == "monthly"
                                            >
                                                Monthly
                                            </option>
                                        </select>
                                    </div>
                                </p>
                                <p class="control">"every"</p>
                                <p class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        min=0
                                        value=repetition_frequency
                                        on:change=move |ev| {
                                            repetition_frequency
                                                .update(|v| {
                                                    *v = event_target_value(&ev).parse().unwrap();
                                                })
                                        }
                                    />

                                </p>
                                <p class="control">
                                    {move || match repetition_type.get().as_str() {
                                        "weekly" => "weeks",
                                        "daily" => "days",
                                        _ => "months",
                                    }}

                                </p>
                            </div>
                            <Show when=move || { repetition_type.get() == "weekly" } fallback=|| {}>
                                <div class="field is-grouped">
                                    <p class="control">"On"</p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="monday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Mon
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="tuesday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Tue
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="wednesday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Wed
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="thursday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Thu
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="friday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Fri
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="saturday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Sat
                                        </label>
                                    </p>
                                    <p class="control">
                                        <label class="checkbox">
                                            <input
                                                type="checkbox"
                                                value="sunday"
                                                on:change=move |ev| {
                                                    repetition_on_day
                                                        .update(|r| {
                                                            let val = event_target_value(&ev);
                                                            if event_target_checked(&ev) {
                                                                r.insert(val);
                                                            } else {
                                                                r.remove(&val);
                                                            }
                                                        })
                                                }
                                            />

                                            Sun
                                        </label>
                                    </p>
                                </div>
                            </Show>
                            <Show
                                when=move || { repetition_type.get() == "monthly" }
                                fallback=|| {}
                            >
                                <div class="field is-grouped">
                                    <p class="control">On day</p>
                                    <p class="control">
                                        <input
                                            class="input"
                                            type="number"
                                            value=month_day
                                            min=1
                                            max=31
                                            on:change=move |ev| {
                                                month_day
                                                    .update(|v| {
                                                        *v = event_target_value(&ev).parse().unwrap();
                                                    })
                                            }
                                        />

                                    </p>
                                </div>
                            </Show>
                            <div class="columns">
                                <div class="column is-narrow">"End"</div>
                                <div class="column">
                                    <div class="columns">
                                        <div class="column">
                                            <div class="field is-grouped">
                                                <p class="control">
                                                    <label class="radio">
                                                        <input
                                                            name="end"
                                                            type="radio"
                                                            on:click=move |_| end_type.set(EndType::Occurences)
                                                            checked
                                                        />
                                                    </label>
                                                </p>
                                                <p class="control">"After"</p>
                                                <p class="control">
                                                    <input
                                                        class="input"
                                                        type="number"
                                                        value=occurences
                                                        min=1
                                                        max=31
                                                        on:change=move |ev| {
                                                            occurences
                                                                .update(|v| {
                                                                    *v = event_target_value(&ev).parse().unwrap();
                                                                })
                                                        }
                                                    />

                                                </p>
                                                <p class="control">"occurences"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="columns">
                                        <div class="column">
                                            <div class="column">
                                                <div class="field is-grouped">
                                                    <p class="control">

                                                        <label class="radio">
                                                            <input
                                                                name="end"
                                                                type="radio"
                                                                on:click=move |_| end_type.set(EndType::EndDate)
                                                            />
                                                        </label>
                                                    </p>
                                                    <p class="control">"On date"</p>
                                                    <p class="control">
                                                        <input
                                                            class="input"
                                                            type="date"
                                                            value=end_date
                                                            on:change=move |ev| {
                                                                end_date
                                                                    .update(|v| {
                                                                        *v = Some(event_target_value(&ev));
                                                                    })
                                                            }
                                                        />

                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="modal-card-foot">
                            <button class="button" on:click=move |_| show.set(false)>
                                Cancel
                            </button>
                            <button type="submit" class="button is-success">
                                <i class="material-symbols-rounded right">save</i>
                                Add
                            </button>
                        </div>
                    </div>
                </div>
            </Form>
            <div
                class="modal-overlay"
                style="z-index: 1002; display: block; opacity: 0.5;"
                on:click=move |_| show.set(false)
            ></div>
        </Show>
    }
}
