use crate::hooks::use_hydrated;
use crate::i18n::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::error::db_err;

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
    use crate::{auth, phone as phone_mod};

    let (pool, _user) = auth::require_admin().await?;

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
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

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
    .map_err(db_err)?;

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
    use crate::auth;

    let (pool, _user) = auth::require_admin().await?;

    sqlx::query!(
        "UPDATE users SET status = 'deactivated' WHERE id = $1",
        user_id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?;

    // Revoke all active sessions immediately — deactivated user cannot continue
    // using existing browser sessions on the next request.
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", user_id)
        .execute(&pool)
        .await
        .map_err(db_err)?;

    Ok(())
}

// ── Component helpers ─────────────────────────────────────────────────────────

/// Registration form sub-component using `ActionForm`.
///
/// Reads form values from `FormData` at submit time (DOM-direct),
/// bypassing reactive signals for input values.
#[component]
fn RegisterForm(register_action: ServerAction<RegisterParticipant>) -> impl IntoView {
    let i18n = use_i18n();
    let hydrated = use_hydrated();

    view! {
        <leptos::form::ActionForm action=register_action>
            <div class="field">
                <label class="field-label" for="reg-phone">
                    {t!(i18n, participants_phone_label)}
                </label>
                <input
                    class="field-input"
                    id="reg-phone"
                    type="tel"
                    name="phone"
                    placeholder="+380XXXXXXXXX"
                    data-testid="reg-phone-input"
                    aria-describedby="action-error"
                />
            </div>
            <div class="field">
                <label class="field-label" for="reg-name">
                    {t!(i18n, participants_name_label)}
                </label>
                <input
                    class="field-input"
                    id="reg-name"
                    type="text"
                    name="name"
                    placeholder=move || t_string!(i18n, participants_name_placeholder)
                    data-testid="reg-name-input"
                    aria-describedby="action-error"
                />
            </div>
            <button
                class="btn"
                type="submit"
                data-testid="register-button"
                disabled=move || !hydrated.get()
            >
                {t!(i18n, participants_register_button)}
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
    let i18n = use_i18n();
    let hydrated = use_hydrated();

    view! {
        <Suspense fallback=move || {
            view! { <p>{t!(i18n, common_loading)}</p> }
        }>
            {move || {
                participants
                    .get()
                    .map(|result| match result {
                        Err(e) => view! { <p class="alert">{e.to_string()}</p> }.into_any(),
                        Ok(list) => {
                            view! {
                                <table class="data-table">
                                    <thead>
                                        <tr>
                                            <th>{t!(i18n, participants_table_name)}</th>
                                            <th>{t!(i18n, participants_table_phone)}</th>
                                            <th>{t!(i18n, participants_table_status)}</th>
                                            <th>{t!(i18n, participants_table_actions)}</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || list.clone()
                                            key=|p| p.id
                                            children=move |p| {
                                                let uid_str = p.id.to_string();
                                                let active = matches!(
                                                    p.status,
                                                    crate::types::UserStatus::Active
                                                );
                                                view! {
                                                    <tr data-testid="participant-row">
                                                        <td data-testid="participant-name-cell">
                                                            {p.name.clone()}
                                                        </td>
                                                        <td>{p.phone.clone()}</td>
                                                        <td>
                                                            {move || {
                                                                if active {
                                                                    view! {
                                                                        <span class="badge" data-status="active">
                                                                            {t!(i18n, participants_status_active)}
                                                                        </span>
                                                                    }
                                                                        .into_any()
                                                                } else {
                                                                    view! {
                                                                        <span class="badge" data-status="inactive">
                                                                            {t!(i18n, participants_status_deactivated)}
                                                                        </span>
                                                                    }
                                                                        .into_any()
                                                                }
                                                            }}
                                                        </td>
                                                        <td>
                                                            {if active {
                                                                view! {
                                                                    <leptos::form::ActionForm action=deactivate_action>
                                                                        <input type="hidden" name="user_id" value=uid_str />
                                                                        <button
                                                                            class="btn"
                                                                            data-variant="destructive"
                                                                            data-size="sm"
                                                                            type="submit"
                                                                            data-testid="deactivate-button"
                                                                            disabled=move || !hydrated.get()
                                                                        >
                                                                            {t!(i18n, participants_deactivate_button)}
                                                                        </button>
                                                                    </leptos::form::ActionForm>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <span
                                                                        class="badge"
                                                                        data-status="inactive"
                                                                        data-testid="inactive-status"
                                                                    >
                                                                        {t!(i18n, participants_deactivated_label)}
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            }}
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}

// ── Page component ────────────────────────────────────────────────────────────

/// Admin participants management page (Story 1.1).
#[component]
pub fn ParticipantsPage() -> impl IntoView {
    let i18n = use_i18n();
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
        <div class="prose-page">
            <h1>{t!(i18n, participants_page_title)}</h1>

            <section>
                <h2>{t!(i18n, participants_register_section_title)}</h2>
                <RegisterForm register_action=register_action />
                <div
                    id="action-error"
                    role="alert"
                    aria-live="assertive"
                    data-testid="action-error"
                >
                    {move || error_msg.get().map(|msg| view! { <p class="alert">{msg}</p> })}
                </div>
            </section>

            <section>
                <h2>{t!(i18n, participants_list_title)}</h2>
                <ParticipantList participants=participants deactivate_action=deactivate_action />
            </section>
        </div>
    }
}
