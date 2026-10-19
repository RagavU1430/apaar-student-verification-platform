//! Creates the demo accounts used by the prototype.
//!
//! Usage: `cargo run --bin seed_users`
//!
//! All demo accounts share the password `DemoPass123!`. The accounts are
//! synthetic and point at the seeded demo institutions only.

use aicte_apaar_api::auth::hash_password;
use aicte_apaar_api::config::Config;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const INSTITUTION_A: Uuid = Uuid::from_u128(0x11111111111141118111111111111111);
const INSTITUTION_B: Uuid = Uuid::from_u128(0x22222222222242228222222222222222);

struct SeedUser {
    login: &'static str,
    display_name: &'static str,
    role: &'static str,
    institution: Option<Uuid>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let users = [
        SeedUser {
            login: "aicte.admin",
            display_name: "AICTE Regulator",
            role: "AICTE_ADMIN",
            institution: None,
        },
        SeedUser {
            login: "alpha.admin",
            display_name: "Alpha Institution Admin",
            role: "INSTITUTION_ADMIN",
            institution: Some(INSTITUTION_A),
        },
        SeedUser {
            login: "alpha.operator",
            display_name: "Alpha Data Operator",
            role: "INSTITUTION_DATA_OPERATOR",
            institution: Some(INSTITUTION_A),
        },
        SeedUser {
            login: "beta.operator",
            display_name: "Beta Data Operator",
            role: "INSTITUTION_DATA_OPERATOR",
            institution: Some(INSTITUTION_B),
        },
        SeedUser {
            login: "verifier",
            display_name: "Authorized Verifier",
            role: "AUTHORIZED_VERIFIER",
            institution: None,
        },
        SeedUser {
            login: "auditor",
            display_name: "Read-only Auditor",
            role: "AUDITOR",
            institution: None,
        },
    ];

    let hash = hash_password("DemoPass123!")?;

    for user in users {
        sqlx::query(
            "INSERT INTO users (id, login_identifier, display_name, role, institution_id, password_hash, status)
             VALUES ($1,$2,$3,$4,$5,$6,'ACTIVE')
             ON CONFLICT (login_identifier) DO UPDATE
                SET display_name = EXCLUDED.display_name,
                    role = EXCLUDED.role,
                    institution_id = EXCLUDED.institution_id,
                    password_hash = EXCLUDED.password_hash,
                    status = 'ACTIVE'",
        )
        .bind(Uuid::new_v4())
        .bind(user.login)
        .bind(user.display_name)
        .bind(user.role)
        .bind(user.institution)
        .bind(&hash)
        .execute(&pool)
        .await?;
        println!("seeded {} ({})", user.login, user.role);
    }

    println!("\nAll demo accounts use the password: DemoPass123!");
    Ok(())
}
