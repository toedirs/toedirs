use leptos::*;
use leptos_router::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <nav class="navbar is-black" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <a href="#" class="navbar-item">
                    Toedi
                </a>
            </div>
            <div class="navbar-end">
                <div class="buttons">
                    <A href="/home/login" exact=true class="button is-primary">
                        Login
                    </A>
                    <A href="/home/signup" exact=true class="button">
                        Signup
                    </A>
                </div>
            </div>
        </nav>
        <main>
            <div class="container">
                <Outlet/>
            </div>
        </main>
    }
}
