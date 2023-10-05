use leptos::*;

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
pub fn BarChart(
    values: ReadSignal<Vec<f64>>,
    // colors: Option<&'chart [&'chart str]>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let num_bars = create_memo(move |_| values.get().len() as f64);
    let max = create_memo(move |_| {
        values
            .get()
            .iter()
            .max_by(|a, b| a.total_cmp(&b))
            .map(|v| v.clone())
            .unwrap()
    });
    let values = create_memo(move |_| {
        values
            .get()
            .into_iter()
            .zip(CATPPUCCIN_COLORS.into_iter().cycle())
            .enumerate()
            .collect::<Vec<(usize, (f64, &&str))>>()
    });

    view! {
        <svg viewBox="0 0 100 100" {..attrs}>
            <g transform="matrix(1 0 0 -1 0 100)">
                {move || values.get().into_iter().map(|(i, (v, color))|view!{
                    <rect
                        x=move || (100.0 / num_bars.get() * i as f64)
                        y=0
                        width=move || (80.0 / num_bars.get())
                        height=move || (100.0 * v / max.get())
                        fill=*color
                        fill-opacity="0.6"
                        stroke=*color
                        stroke-width="0.5"
                    />
                }).collect_view()}
            </g>
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
