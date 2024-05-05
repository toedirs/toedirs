use crate::authentication;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Login(action: Action<authentication::Login, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1 class="title">"Log In"</h1>
            <div class="field">
                <label for="username">"User ID:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="text"
                        placeholder="User ID"
                        maxlength="32"
                        name="username"
                        id="username"
                    />

                </div>
            </div>
            <div class="field">
                <label for="password">"Password:"</label>
                <div class="control">
                    <input class="input" type="password" placeholder="Password" name="password"/>

                </div>
            </div>
            <div class="field">
                <div class="control">
                    <label>
                        <input type="checkbox" name="remember"/>
                        "Remember me?"
                    </label>
                </div>
            </div>
            <div class="field is-grouped">
                <p class="control">
                    <button type="submit" class="button is-primary">
                        "Log In"
                    </button>
                </p>
                <p class="control">
                    <A href="/home/signup" class="button">
                        Signup
                    </A>
                </p>
            </div>
        </ActionForm>
    }
}
