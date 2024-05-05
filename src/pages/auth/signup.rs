use crate::authentication;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Signup(action: Action<authentication::Signup, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=action>
            <h1 class="title">"Sign Up"</h1>
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
                    <input
                        class="input"
                        type="password"
                        placeholder="Password"
                        name="password"
                        id="password"
                    />

                </div>
            </div>
            <div class="field">
                <label for="password_confirmation">"Confirm Password:"</label>
                <div class="control">
                    <input
                        class="input"
                        type="password"
                        placeholder="Password again"
                        name="password_confirmation"
                        id="password_confirmation"
                    />

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
                        "Sign Up"
                    </button>
                </p>
                <p class="control">

                    <A href="/home/login" class="button">
                        Login
                    </A>
                </p>
            </div>
        </ActionForm>
    }
}
