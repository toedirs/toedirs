use chrono::NaiveDate;
use leptos::*;
use thaw::*;

#[component]
pub fn DatePickerRange(
    #[prop(into)] from: RwSignal<Option<NaiveDate>>,
    #[prop(into)] to: RwSignal<Option<NaiveDate>>,
    #[prop(into)] result: RwSignal<(Option<NaiveDate>, Option<NaiveDate>)>,
    #[prop(into)] show: RwSignal<bool>,
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
) -> impl IntoView {
    view! {
        <Show when=move || { show() } fallback=|| {}>
            <div class="modal" {..attrs.clone()}>
                <div class="modal-body">
                    <div class="modal-content">
                        <div class="row">
                            <div class="input-field col s6">
                                <DatePicker value=from attr:id="from_date"/>
                                <label for="from_date">From</label>
                            </div>
                            <div class="input-field col s6">
                                <DatePicker value=to attr:id="to_date"/>
                                <label for="to_date">To</label>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-footer">
                    <a
                        href="#!"
                        class="modal-close waves-effect waves-light btn-flat"
                        on:click=move |_| {
                            show.set(false);
                        }
                    >

                        Cancel
                    </a>
                    <a
                        href="#!"
                        class="modal-close waves-effect waves-light btn-flat"
                        on:click=move |_| {
                            result
                                .update(|res: &mut (Option<NaiveDate>, Option<NaiveDate>)| {
                                    *res = (from(), to());
                                });
                            show.set(false);
                        }
                    >

                        Ok
                    </a>
                </div>
            </div>
        </Show>
    }
}
