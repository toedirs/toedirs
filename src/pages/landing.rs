use leptos::*;
#[component]
pub fn Landing() -> impl IntoView {
    view! {
        <div class="container">
            <h1 class="title">Welcome to Toedi</h1>
            <div class="columns">
                <div class="column is-flex">
                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Track your Training</p>
                        </div>
                        <div class="card-content">
                            <p>
                                Always stay on top of your training effort with easy to read charts and metrics
                            </p>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">

                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Based on Science</p>
                        </div>
                        <div class="card-content">
                            <p>
                                "Based on newest scientific research, presented in a transparent way. We don't just make up numbers and we explain exactly how our metrics are calculated"
                            </p>
                        </div>
                    </div>
                </div>
                <div class="column is-flex">

                    <div class="card">
                        <div class="card-header">
                            <p class="card-header-title">Open Source</p>
                        </div>
                        <div class="card-content">
                            <p>Fully Open-Source code, made by users for users</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
