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
            .fold(f64::NEG_INFINITY, f64::min);
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

    view! {
        <svg  {..attrs}>
            <line x1="9.8%" y1="0%" x2="9.8%" y2="100%" stroke="black" stroke-width="1px" vector-effect="non-scaling-stroke"/>
            <line x1="7%" y1="10%" x2="9.8%" y2="10%" stroke="black" strocke-width="1px" vector-effect="non-scaling-stroke"/>
            <text x="6.9%" y="10%" font-size="20px" dy="5px" text-anchor="end" vector-effect="non-scaling-stroke">10</text>
            <svg x="10%" width="90%" height="100%" viewBox="0 0 100 100" preserveAspectRatio="none">
                <g transform="matrix(1 0 0 -1 0 100)">
                    {move || values.get().into_iter().map(|(i, (v, color))|view!{
                        <rect
                            x=move || (5.0  + 95.0 / num_bars.get() * i as f64)
                            y=0
                            width=move || (80.0 / num_bars.get())
                            height=move || (100.0 * v / max.get())
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
