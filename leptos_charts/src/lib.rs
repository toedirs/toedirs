#![feature(iter_map_windows)]
use std::{f64::consts::TAU, iter};

use itertools::Itertools;
use leptos::{svg::*, *};
use leptos_use::*;
use num_traits::ToPrimitive;

const CATPPUCCIN_COLORS: &[&str] = &[
    "#dc8a78", //rosewater
    "#8839ef", //Mauve
    "#fe640b", //Peach
    "#40a02b", //green
    "#04a5e5", //Sky
    "#ea76cb", //Pink
    "#1e66f5", //Blue
    "#d20f39", //Red
    "#df8e1d", //yellow
    "#209fb5", //Sapphire
    "#7287fd", //lavender
    "#e64553", //maroon
];

pub struct ChartOptions {
    pub max_ticks: u8,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self { max_ticks: 5u8 }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TickSpacing {
    min_point: f64,
    max_point: f64,
    spacing: f64,
    num_ticks: u8,
}

fn nice_num(num: f64, round: bool) -> f64 {
    let exponent = num.log10().floor();
    let fraction = num / 10.0f64.powf(exponent);
    let nice_fraction = if round {
        if fraction < 1.5 {
            1.0
        } else if fraction < 3.0 {
            2.0
        } else if fraction < 7.0 {
            5.0
        } else {
            10.0
        }
    } else {
        if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        }
    };
    nice_fraction * 10.0f64.powf(exponent)
}

fn nice_ticks(min: f64, max: f64, max_ticks: u8) -> TickSpacing {
    let range = nice_num(max - min, false);
    let spacing = nice_num(range / (max_ticks - 1) as f64, true);
    let min_point = (min / spacing).floor() * spacing;
    let max_point = (max / spacing).ceil() * spacing;
    let num_ticks = ((max_point - min_point) / spacing) as u8 + 1;
    TickSpacing {
        min_point,
        max_point,
        spacing,
        num_ticks,
    }
}

#[component]
pub fn BarChart<T>(
    values: MaybeSignal<Vec<T>>,
    options: ChartOptions,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView
where
    T: ToPrimitive + Clone + PartialOrd + 'static,
{
    let vals = values.clone();
    let num_bars = create_memo(move |_| vals.get().len() as f64);
    let vals = values.clone();
    let max = create_memo(move |_| {
        vals.get()
            .iter()
            .map(|v| v.to_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max)
    });
    let vals = values.clone();
    let min = create_memo(move |_| {
        let min = vals
            .get()
            .iter()
            .map(|v| v.to_f64().unwrap())
            .fold(f64::INFINITY, f64::min);
        if min < 0.0 {
            min
        } else {
            0.0
        }
    });
    let vals = values.clone();
    let values = create_memo(move |_| {
        vals.get()
            .into_iter()
            .map(|v| v.to_f64().unwrap())
            .zip(CATPPUCCIN_COLORS.into_iter().cycle())
            .enumerate()
            .collect::<Vec<(usize, (f64, &&str))>>()
    });
    let tick_config = create_memo(move |_| nice_ticks(min.get(), max.get(), options.max_ticks));
    let ticks = create_memo(move |_| {
        let ticks = tick_config.get();
        (0..ticks.num_ticks)
            .map(|i| ticks.min_point + i as f64 * ticks.spacing)
            .map(move |tick| {
                (
                    100.0 - tick / (ticks.max_point - ticks.min_point) * 100.0,
                    format!("{}", tick),
                )
            })
            .collect::<Vec<(f64, String)>>()
    });

    view! {
        <svg {..attrs}>
            <svg y="5%" height="90%" overflow="visible">
                <line
                    x1="9.8%"
                    y1="0%"
                    x2="9.8%"
                    y2="100%"
                    stroke="black"
                    stroke-width="1px"
                    vector-effect="non-scaling-stroke"
                ></line>
                {move || {
                    ticks
                        .get()
                        .into_iter()
                        .map(|(t, s)| {
                            view! {
                                <line
                                    x1="7%"
                                    y1=format!("{}%", t)
                                    x2="9.8%"
                                    y2=format!("{}%", t)
                                    stroke="black"
                                    strocke-width="1px"
                                    vector-effect="non-scaling-stroke"
                                ></line>
                                <text
                                    x="6.9%"
                                    y=format!("{}%", t)
                                    font-size="20px"
                                    dy="5px"
                                    text-anchor="end"
                                    vector-effect="non-scaling-stroke"
                                >
                                    {s}
                                </text>
                            }
                        })
                        .collect_view()
                }}
                {move || {
                    values
                        .get()
                        .into_iter()
                        .map(|(i, (v, color))| {
                            let el = create_node_ref::<Rect>();
                            let is_hovered = use_element_hover(el);
                            view! {
                                <svg
                                    x="10%"
                                    width="90%"
                                    height="100%"
                                    viewBox="0 0 100 100"
                                    preserveAspectRatio="none"
                                >
                                    <g transform="matrix(1 0 0 -1 0 100)">
                                        <rect
                                            node_ref=el
                                            x=move || (5.0 + 95.0 / num_bars.get() * i as f64)
                                            y=0
                                            width=move || (80.0 / num_bars.get())
                                            height=move || {
                                                (100.0 * (v - tick_config.get().min_point)
                                                    / (tick_config.get().max_point
                                                        - tick_config.get().min_point))
                                            }
                                            fill=*color
                                            fill-opacity=move || {
                                                if is_hovered.get() { "0.8" } else { "0.6" }
                                            }
                                            stroke=*color
                                            stroke-width=move || {
                                                if is_hovered.get() { "3px" } else { "1px" }
                                            }
                                            vector-effect="non-scaling-stroke"
                                        ></rect>
                                    </g>
                                </svg>
                                <Show when=move || is_hovered.get() fallback=|| ()>
                                    <text
                                        font-size="15px"
                                        vector-effect="non-scaling-stroke"
                                        x=move || {
                                            format!(
                                                "{}%", (15.0 + 85.0 / num_bars.get() * (i as f64 + 0.5))
                                            )
                                        }
                                        y=move || {
                                            format!(
                                                "{}%", (100.0 - 100.0 * (v - tick_config.get().min_point) /
                                                (tick_config.get().max_point - tick_config.get().min_point))
                                            )
                                        }
                                        dy="-5"
                                        dx="-9"
                                    >
                                        {v}
                                    </text>
                                </Show>
                            }
                        })
                        .collect_view()
                }}
            </svg>
        </svg>
    }
}

#[component]
pub fn PieChart<T>(
    values: MaybeSignal<Vec<T>>,
    options: ChartOptions,
    // colors: Option<&'chart [&'chart str]>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView
where
    T: ToPrimitive + Clone + PartialOrd + 'static,
{
    let values = create_memo(move |_| {
        values
            .get()
            .iter()
            .map(|v| v.to_f64().unwrap())
            .collect::<Vec<f64>>()
    });
    let sum = create_memo(move |_| values.get().iter().sum::<f64>());
    let sorted_values = create_memo(move |_| {
        iter::once((0.0, 99.0, 0.0))
            .chain(
                values
                    .get()
                    .into_iter()
                    .sorted_by(|a, b| f64::partial_cmp(a, b).unwrap())
                    .map(|f| (f, f / sum.get()))
                    .scan((0.0, 0.0), |state, v| {
                        *state = (v.0, state.1 + v.1);
                        Some(*state)
                    })
                    .map(|(f, v)| (f, (v * TAU).cos() * 99.0, (v * TAU).sin() * 99.0)),
            )
            .map_windows(|[from, to]| {
                (
                    to.0,
                    format!(
                        "M0 0 {from_x} {from_y} A100 100 0 0 1 {to_x} {to_y}Z",
                        from_x = from.1,
                        from_y = from.2,
                        to_x = to.1,
                        to_y = to.2
                    ),
                    ((from.1 + to.1) / 2.0)
                        / (f64::sqrt(
                            ((from.1 + to.1) / 2.0).powi(2) + ((from.2 + to.2) / 2.0).powi(2),
                        ))
                        * 85.0,
                    ((from.2 + to.2) / 2.0)
                        / (f64::sqrt(
                            ((from.1 + to.1) / 2.0).powi(2) + ((from.2 + to.2) / 2.0).powi(2),
                        ))
                        * 85.0,
                )
            })
            .zip(CATPPUCCIN_COLORS.into_iter().cycle())
            .collect::<Vec<((f64, String, f64, f64), &&str)>>()
    });

    view! {
        <svg {..attrs}>
            {move || {
                sorted_values
                    .get()
                    .into_iter()
                    .enumerate()
                    .map(|(i, ((value, path, label_x, label_y), color))| {
                        let el = create_node_ref::<Path>();
                        let is_hovered = use_element_hover(el);
                        view! {
                            <svg viewBox="0 0 200 200">
                                <g transform="translate(100,100)" stroke="#000" stroke-width="1">
                                    <mask id=format!("cut-path-{}", i)>
                                        <path
                                            d=path.clone()
                                            fill="white"
                                            stroke="black"
                                            stroke-width="2"
                                            vector-effect="non-scaling-stroke"
                                        ></path>
                                    </mask>
                                    <path
                                        node_ref=el
                                        d=path
                                        fill=*color
                                        fill-opacity=0.6
                                        stroke=*color
                                        stroke-width="2"
                                        vector-effect="non-scaling-stroke"
                                        mask=move || {
                                            if is_hovered.get() {
                                                "none".to_string()
                                            } else {
                                                format!("url(#cut-path-{})", i)
                                            }
                                        }
                                    ></path>
                                    <Show when=move || is_hovered.get() fallback=|| ()>
                                        <text
                                            font-size="15px"
                                            vector-effect="non-scaling-stroke"
                                            x=label_x
                                            y=label_y
                                        >
                                            <tspan
                                                text-anchor="middle"
                                                dominant-baseline="middle"
                                                color="#000"
                                            >
                                                {value}
                                            </tspan>
                                        </text>
                                    </Show>
                                </g>
                            </svg>
                        }
                    })
                    .collect_view()
            }}
        </svg>
    }
}
