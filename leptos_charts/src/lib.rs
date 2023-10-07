use leptos::*;
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

fn nice_ticks(min: f64, max: f64, max_ticks: Option<u8>) -> TickSpacing {
    let range = nice_num(max - min, false);
    let spacing = nice_num(range / (max_ticks.unwrap_or(5) - 1) as f64, true);
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
    values: ReadSignal<Vec<T>>,
    // colors: Option<&'chart [&'chart str]>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView
where
    T: ToPrimitive + Clone + PartialOrd + 'static,
{
    let num_bars = create_memo(move |_| values.get().len() as f64);
    let max = create_memo(move |_| {
        values
            .get()
            .iter()
            .map(|v| v.to_f64().unwrap())
            .fold(f64::NEG_INFINITY, f64::max)
    });
    let min = create_memo(move |_| {
        let min = values
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
    let values = create_memo(move |_| {
        values
            .get()
            .into_iter()
            .map(|v| v.to_f64().unwrap())
            .zip(CATPPUCCIN_COLORS.into_iter().cycle())
            .enumerate()
            .collect::<Vec<(usize, (f64, &&str))>>()
    });
    let tick_config = create_memo(move |_| nice_ticks(min.get(), max.get(), None));
    let ticks = create_memo(move |_| {
        let ticks = tick_config.get();
        leptos::logging::log!("{:?}", ticks);
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
        <svg  {..attrs}>
            <svg y="5%" height="90%" overflow="visible">
                <line x1="9.8%" y1="0%" x2="9.8%" y2="100%" stroke="black" stroke-width="1px" vector-effect="non-scaling-stroke"/>
                {move ||ticks.get().into_iter().map(|(t, s)|
                        view!{
                        <line x1="7%" y1=format!("{}%", t) x2="9.8%" y2=format!("{}%", t) stroke="black" strocke-width="1px" vector-effect="non-scaling-stroke"/>
                        <text x="6.9%" y=format!("{}%", t) font-size="20px" dy="5px" text-anchor="end" vector-effect="non-scaling-stroke">{s}</text>
                    }).collect_view()}
                <svg x="10%" width="90%" height="100%" viewBox="0 0 100 100" preserveAspectRatio="none">
                    <g transform="matrix(1 0 0 -1 0 100)">
                        {move || values.get().into_iter().map(|(i, (v, color))|view!{
                            <rect
                                x=move || (5.0  + 95.0 / num_bars.get() * i as f64)
                                y=0
                                width=move || (80.0 / num_bars.get())
                                height=move || (100.0 * (v - tick_config.get().min_point) / (tick_config.get().max_point - tick_config.get().min_point))
                                fill=*color
                                fill-opacity="0.6"
                                stroke=*color
                                stroke-width="1px"
                                vector-effect="non-scaling-stroke"
                            />
                        }).collect_view()}
                    </g>
                </svg>
            </svg>
        </svg>
    }
}
// #[component]
// pub fn BarChart(
//     values: ReadSignal<Vec<(usize, f64)>>,
//     #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
// ) -> impl IntoView {
//     let num_bars = create_memo(move |_| values.get().len() as f64);
//     let max = create_memo(move |_| {
//         values
//             .get()
//             .iter()
//             .max_by(|a, b| a.1.total_cmp(&b.1))
//             .map(|(i, v)| v.clone())
//             .unwrap()
//     });

//     view! {
//         <svg viewBox="0 0 100 100" {..attrs}>
//             <g transform="matrix(1 0 0 -1 0 100)">
//                 <For
//                     each=move||values.get()
//                     key=|v|v.0
//                     let:entry>
//                     <rect x=move || (100.0 / num_bars.get() * entry.0 as f64) y=0
//                     width=move || (80.0 / num_bars.get())
//                     height=move || (100.0 * entry.1 / max.get())
//                     ></rect>
//                 </For>
//             </g>
//         </svg>
//     }
// }
