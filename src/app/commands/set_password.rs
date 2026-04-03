use crate::server::auth::password::{
    generate_random_password, generate_token_key, get_stored_auth, hash_password,
    set_password_hash_and_revoke_sessions, set_stored_auth,
};
use crate::store::lifecycle::KOSHELF_DB_FILENAME;
use crate::store::sqlite::{open_koshelf_pool, run_koshelf_migrations};
use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;

pub(crate) async fn set_password(
    data_path: PathBuf,
    password_arg: Option<String>,
    random: bool,
    overwrite: bool,
) -> Result<()> {
    std::fs::create_dir_all(&data_path).with_context(|| {
        format!(
            "Failed to create data directory for set-password command at {}",
            data_path.display()
        )
    })?;

    let koshelf_db_path = data_path.join(KOSHELF_DB_FILENAME);

    let koshelf_pool = open_koshelf_pool(&koshelf_db_path)
        .await
        .context("Failed to open KoShelf app DB")?;
    run_koshelf_migrations(&koshelf_pool)
        .await
        .context("Failed to run KoShelf app DB migrations")?;

    let stored_auth = get_stored_auth(&koshelf_pool).await?;
    if stored_auth.is_some() && !overwrite {
        info!(
            "Authentication password is already initialized. Re-run with --overwrite to replace it."
        );
        return Ok(());
    }

    let (new_password, is_random_password) = if random {
        (generate_random_password()?, true)
    } else {
        let password = match password_arg {
            Some(value) => value,
            None => {
                let first = rpassword::prompt_password("New password: ")
                    .context("Failed to read password from terminal")?;
                let second = rpassword::prompt_password("Confirm new password: ")
                    .context("Failed to read password confirmation from terminal")?;
                if first != second {
                    anyhow::bail!("Passwords do not match")
                }
                first
            }
        };
        (password, false)
    };

    let new_hash = hash_password(&new_password)?;

    match stored_auth {
        Some((_stored_hash, _stored_token_key)) => {
            set_password_hash_and_revoke_sessions(&koshelf_pool, &new_hash, None).await?;
        }
        None => {
            let token_key = generate_token_key()?;
            set_stored_auth(&koshelf_pool, &new_hash, &token_key).await?;
        }
    }

    if is_random_password {
        eprintln!();
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!("SET-PASSWORD");
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!("Generated authentication password: {}", new_password);
        eprintln!("This password will not be shown again. Save it now.");
        eprintln!(
            "--------------------------------------------------------------------------------"
        );
        eprintln!();
    }

    info!(
        "Authentication password updated successfully for data path {}",
        data_path.display()
    );

    Ok(())
}
