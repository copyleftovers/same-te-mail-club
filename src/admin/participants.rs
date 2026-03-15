use leptos::prelude::*;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParticipantSummary {
    pub id: uuid::Uuid,
    pub phone: String,
    pub name: String,
    pub status: crate::types::UserStatus,
    pub onboarded: bool,
}

// Row type used by list_participants query (SSR only)
#[cfg(feature = "ssr")]
struct ParticipantRow {
    id: uuid::Uuid,
    phone: String,
    name: String,
    status: crate::types::UserStatus,
    onboarded: bool,
}

// ── Server functions ──────────────────────────────────────────────────────────

/// Register a new participant (admin only).
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin, phone is invalid, name is empty,
/// or the phone number already exists.
#[server]
pub async fn register_participant(phone: String, name: String) -> Result<(), ServerFnError> {
    use crate::{auth, phone as phone_mod, types::UserRole};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    let normalized = phone_mod::normalize(&phone)
        .map_err(|e| ServerFnError::new(format!("invalid phone: {e}")))?;

    let trimmed_name = name.trim().to_owned();
    if trimmed_name.is_empty() {
        return Err(ServerFnError::new("name is required"));
    }

    sqlx::query!(
        r#"INSERT INTO users (phone, name) VALUES ($1, $2)"#,
        normalized,
        trimmed_name,
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        // Unique violation on phone
        if let Some(db_err) = e.as_database_error()
            && db_err.code().as_deref() == Some("23505")
        {
            return ServerFnError::new("phone number already exists");
        }
        ServerFnError::new(format!("database error: {e}"))
    })?;

    Ok(())
}

/// List all participants (admin only).
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or DB fails.
#[server]
pub async fn list_participants() -> Result<Vec<ParticipantSummary>, ServerFnError> {
    use crate::{auth, types::UserRole};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let user = auth::current_user(&pool, &parts)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if user.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    let rows = sqlx::query_as!(
        ParticipantRow,
        r#"
        SELECT
            id,
            phone,
            name,
            status AS "status: crate::types::UserStatus",
            onboarded
        FROM users
        WHERE role = 'participant'
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| ParticipantSummary {
            id: r.id,
            phone: r.phone,
            name: r.name,
            status: r.status,
            onboarded: r.onboarded,
        })
        .collect())
}

/// Deactivate a participant account (admin only).
///
/// # Errors
///
/// Returns `Err` if the caller is not an admin or DB fails.
#[server]
pub async fn deactivate_participant(user_id: uuid::Uuid) -> Result<(), ServerFnError> {
    use crate::{auth, types::UserRole};
    use http::request::Parts;

    let pool = leptos::context::use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("no database pool in context"))?;
    let parts = leptos::context::use_context::<Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let caller = auth::current_user(&pool, &parts)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if caller.role != UserRole::Admin {
        return Err(ServerFnError::new("forbidden: admin only"));
    }

    sqlx::query!(
        "UPDATE users SET status = 'deactivated' WHERE id = $1",
        user_id,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::new(format!("database error: {e}")))?;

    Ok(())
}

// ── Component helpers ─────────────────────────────────────────────────────────

/// Registration form sub-component using `ActionForm`.
///
/// Reads form values from `FormData` at submit time (DOM-direct),
/// bypassing reactive signals for input values.
#[component]
fn RegisterForm(register_action: ServerAction<RegisterParticipant>) -> impl IntoView {
    // Hydration gate — prevent native form POST before WASM hydrates.
    let (hydrated, set_hydrated) = signal(false);
    Effect::new(move |_| {
        set_hydrated.set(true);
    });

    view! {
        <leptos::form::ActionForm action=register_action>
            <div>
                <label for="reg-phone">"Phone number (номер телефону)"</label>
                <input
                    id="reg-phone"
                    type="tel"
                    name="phone"
                    placeholder="+380XXXXXXXXX"
                />
            </div>
            <div>
                <label for="reg-name">"Name (ім'я для Nova Poshta)"</label>
                <input
                    id="reg-name"
                    type="text"
                    name="name"
                    placeholder="Іваненко Іван Іванович"
                />
            </div>
            <button type="submit" data-testid="register-button" disabled=move || !hydrated.get()>
                "Зареєструвати"
            </button>
        </leptos::form::ActionForm>
    }
}

/// Participant list sub-component.
#[component]
fn ParticipantList(
    participants: Resource<Result<Vec<ParticipantSummary>, ServerFnError>>,
    deactivate_action: ServerAction<DeactivateParticipant>,
) -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p>"Завантаження..."</p> }>
            {move || participants.get().map(|result| match result {
                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                Ok(list) => view! {
                    <table>
                        <thead>
                            <tr>
                                <th>"Ім'я"</th>
                                <th>"Телефон"</th>
                                <th>"Статус"</th>
                                <th>"Дії"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || list.clone()
                                key=|p| p.id
                                children=move |p| {
                                    let uid = p.id;
                                    let active = matches!(
                                        p.status,
                                        crate::types::UserStatus::Active
                                    );
                                    view! {
                                        <tr>
                                            <td>{p.name.clone()}</td>
                                            <td>{p.phone.clone()}</td>
                                            <td>
                                                {if active {
                                                    "Активний"
                                                } else {
                                                    "Деактивований"
                                                }}
                                            </td>
                                            <td>
                                                {if active {
                                                    view! {
                                                        <button
                                                            data-testid="deactivate-button"
                                                            on:click=move |_| {
                                                                deactivate_action.dispatch(
                                                                    DeactivateParticipant {
                                                                        user_id: uid,
                                                                    }
                                                                );
                                                            }
                                                        >
                                                            "Деактивувати"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="inactive">
                                                            "Деактивовано"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                }.into_any(),
            })}
        </Suspense>
    }
}

// ── Page component ────────────────────────────────────────────────────────────

/// Admin participants management page (Story 1.1).
#[component]
pub fn ParticipantsPage() -> impl IntoView {
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let register_action = ServerAction::<RegisterParticipant>::new();
    let deactivate_action = ServerAction::<DeactivateParticipant>::new();

    let participants = Resource::new(
        move || {
            (
                register_action.version().get(),
                deactivate_action.version().get(),
            )
        },
        |_| list_participants(),
    );

    Effect::new(move |_| {
        if let Some(result) = register_action.value().get() {
            match result {
                Ok(()) => {
                    set_error_msg.set(None);
                }
                Err(e) => {
                    set_error_msg.set(Some(e.to_string()));
                }
            }
        }
    });

    view! {
        <div class="admin-participants">
            <h1>"Учасники"</h1>

            <section>
                <h2>"Зареєструвати учасника"</h2>
                <RegisterForm
                    register_action=register_action
                />
                {move || error_msg.get().map(|msg| view! {
                    <p class="error">{msg}</p>
                })}
            </section>

            <section>
                <h2>"Список учасників"</h2>
                <ParticipantList
                    participants=participants
                    deactivate_action=deactivate_action
                />
            </section>
        </div>
    }
}
