mod fitness_level_chart;
mod heartrate_distribution_chart;
mod heartrate_summary_chart;
mod slope_speed_chart;
mod training_load_chart;

use chrono::{Duration, Local, NaiveDate, TimeZone};
use fitness_level_chart::FitnessLevelChart;
use heartrate_distribution_chart::HeartrateDistributionChart;
use heartrate_summary_chart::HeartrateZoneSummaryChart;
use leptos::*;
use training_load_chart::TrainingLoadChart;

use slope_speed_chart::SlopeSpeedChart;
#[component]
pub fn Overview() -> impl IntoView {
    //overview page
    let from_date = create_rw_signal(Some(
        (Local::now() - Duration::try_days(120).unwrap())
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
    ));
    let to_date = create_rw_signal(Some(
        Local::now().date_naive().format("%Y-%m-%d").to_string(),
    ));
    let from_memo = create_memo(move |_| {
        from_date().and_then(|d| {
            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                .map(|d| {
                    Local
                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                        .unwrap()
                })
                .ok()
        })
    });
    let to_memo = create_memo(move |_| {
        to_date().and_then(|d| {
            NaiveDate::parse_from_str(d.as_str(), "%Y-%m-%d")
                .map(|d| {
                    Local
                        .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
                        .unwrap()
                })
                .ok()
        })
    });
    view! {
        <div class="container is-fluid">
            <div class="columns">
                <div class="column">

                    <div class="field">
                        <label for="from_date">From</label>
                        <div class="control">
                            <input
                                class="input"
                                type="date"
                                value=from_date
                                on:change=move |ev| {
                                    from_date
                                        .update(|v| {
                                            *v = Some(event_target_value(&ev));
                                        })
                                }
                            />

                        </div>
                    </div>
                </div>
                <div class="column">
                    <div class="field">
                        <label for="to_date">To</label>
                        <div class="control">
                            <input
                                class="input"
                                type="date"
                                value=to_date
                                on:change=move |ev| {
                                    to_date
                                        .update(|v| {
                                            *v = Some(event_target_value(&ev));
                                        })
                                }
                            />

                        </div>
                    </div>

                </div>
            </div>
            <div class="columns is-multiline is-variable is-1">
                <div class="column is-full-mobile is-half-desktop is-one-third-fullhd">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Hearrate Zones</p>
                        </div>
                        <div class="card-content">
                            <div class="content">
                                <HeartrateZoneSummaryChart from=from_memo to=to_memo/>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="column is-full-mobile is-half-desktop is-one-third-fullhd">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Training Load</p>
                        </div>
                        <div class="card-content ">
                            <TrainingLoadChart from=from_memo to=to_memo/>
                        </div>
                    </div>
                </div>
                <div class="column is-full-mobile is-half-desktop is-one-third-fullhd">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Fitness Level</p>
                        </div>
                        <div class="card-content ">
                            <FitnessLevelChart from=from_memo to=to_memo/>
                        </div>
                    </div>
                </div>
                <div class="column is-full-mobile is-half-desktop is-one-third-fullhd">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Heartrate Distribution</p>
                        </div>
                        <div class="card-content ">
                            <HeartrateDistributionChart from=from_memo to=to_memo/>
                        </div>
                    </div>
                </div>
                <div class="column is-full-mobile is-half-desktop is-one-third-fullhd">
                    <div class="card is-fullwidth">
                        <div class="card-header">
                            <p class="card-header-title">Slope Speed</p>
                        </div>
                        <div class="card-content ">
                            <SlopeSpeedChart from=from_memo to=to_memo/>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
