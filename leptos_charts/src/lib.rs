use leptos::*;

#[component]
pub fn BarChart<'data, T: Into<f64> + Clone>(
    data: &'data [&'data T],
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    let values: Vec<f64> = data.iter().map(|&d| (*d).clone().into()).collect();
    let num_bars = values.len() as f64;
    let max = values.iter().max_by(|a, b| a.total_cmp(b)).unwrap();

    let bars: Vec<_> = values
        .iter()
        .enumerate()
        .map(|(index, &entry)| {
            view! {
                <rect
                    x=(100.0 / num_bars * index as f64)
                    y=0
                    width=(80.0 / num_bars)
                    height=(100.0 * entry / *max)
                ></rect>
            }
        })
        .collect();

    view! {
        <svg viewBox="0 0 100 100" {..attrs}>
            <g transform="matrix(1 0 0 -1 0 100)">{bars}</g>
        </svg>
    }
}
