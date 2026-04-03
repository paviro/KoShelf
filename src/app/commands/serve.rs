use crate::app::bootstrap::initialize_pipeline;
use crate::app::config::{ServeArgs, parse_trusted_proxy_nets};
use crate::pipeline::watcher::FileWatcher;
use crate::server::auth::AuthState;
use crate::server::auth::client_addr::ClientAddrResolver;
use crate::server::auth::password::{
    generate_random_password, generate_token_key, get_stored_auth, hash_password, set_stored_auth,
};
use crate::server::auth::rate_limit::login_rate_limiter;
use crate::server::auth::session::{cleanup_expired, paseto_key_from_bytes};
use crate::server::{WebServer, WebServerOptions, WriteCoordinator};
use crate::store::memory::{ReadingDataStore, SiteStore, UpdateNotifier};
use crate::store::sqlite::{open_koshelf_pool, run_koshelf_migrations};
use anyhow::{Context, Result};
use log::warn;
use std::sync::Arc;
use std::time::Duration;

pub(crate) async fn serve(args: ServeArgs) -> Result<()> {
    if let Err(e) = args.validate() {
        super::exit_validation_error("serve", e);
    }

    let output_dir = args
        .common
        .data_path
        .clone()
        .context("Serve mode requires --data-path")?;

    let state = initialize_pipeline(
        &args.common,
        output_dir.clone(),
        true,
        args.enable_auth,
        args.enable_writeback,
        false,
    )
    .await?;

    // ── Auth setup ───────────────────────────────────────────────────
    let koshelf_db_path = state
        .config
        .runtime_data_policy
        .koshelf_db_path()
        .context("Failed to resolve KoShelf DB path")?;

    let koshelf_pool = open_koshelf_pool(&koshelf_db_path)
        .await
        .context("Failed to open KoShelf app DB")?;

    run_koshelf_migrations(&koshelf_pool)
        .await
        .context("Failed to run KoShelf app DB migrations")?;

    let trusted_proxies = parse_trusted_proxy_nets(&args.trusted_proxies)?;
    let client_addr_resolver = Arc::new(ClientAddrResolver::new(trusted_proxies));

    let auth_state = if args.enable_auth {
        let token_key_bytes = if let Some((_stored_hash, stored_token_key)) =
            get_stored_auth(&koshelf_pool).await?
        {
            stored_token_key
        } else {
            let generated_password = generate_random_password()?;
            let password_hash = hash_password(&generated_password)?;
            let token_key = generate_token_key()?;
            set_stored_auth(&koshelf_pool, &password_hash, &token_key).await?;

            let data_path_hint = state
                .config
                .runtime_data_policy
                .persistent_data_dir()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<data-path>".to_string());

            eprintln!();
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!("AUTHENTICATION ENABLED (FIRST RUN)");
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!("Generated authentication password: {}", generated_password);
            eprintln!("This password will not be shown again. Save it now.");
            eprintln!(
                "To set a new one later, run: koshelf set-password --data-path {} --overwrite",
                data_path_hint
            );
            eprintln!(
                "--------------------------------------------------------------------------------"
            );
            eprintln!();

            token_key
        };

        cleanup_expired(&koshelf_pool).await?;

        let cleanup_pool = koshelf_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Err(error) = cleanup_expired(&cleanup_pool).await {
                    warn!("Failed to clean up expired sessions: {}", error);
                }
            }
        });

        let paseto_key = paseto_key_from_bytes(&token_key_bytes)?;

        Some(AuthState {
            pool: koshelf_pool.clone(),
            token_key: Arc::new(paseto_key),
            login_limiter: Arc::new(login_rate_limiter()),
            client_addr_resolver: client_addr_resolver.clone(),
        })
    } else {
        None
    };

    // ── Start server ─────────────────────────────────────────────────
    let revision_epoch = format!("serve_{}", &state.generated_at);
    let initial_generated_at = state.generated_at;

    let site_store = Arc::new(SiteStore::new());
    site_store.replace(state.site_data);

    let reading_data_store = Arc::new(ReadingDataStore::new());
    if let Some(rd) = state.reading_data {
        reading_data_store.replace(rd);
    }

    let update_notifier = UpdateNotifier::new(revision_epoch, initial_generated_at);

    let write_coordinator = if args.enable_writeback {
        Some(WriteCoordinator::new())
    } else {
        None
    };

    let timezone = state.config.time_config.timezone;

    let file_watcher = FileWatcher::new(
        state.config,
        Some(site_store.clone()),
        Some(reading_data_store.clone()),
        Some(update_notifier.clone()),
        Some(state.repo.clone()),
        write_coordinator.as_ref().map(|wc| wc.recent_writes()),
    );

    let web_server = WebServer::new(WebServerOptions {
        media_cache_dir: output_dir,
        port: args.port,
        site_store,
        reading_data_store,
        update_notifier,
        library_repo: state.repo,
        auth_state,
        write_coordinator,
        timezone,
    });

    tokio::select! {
        result = file_watcher.run() => {
            if let Err(e) = result {
                log::error!("File watcher error: {}", e);
            }
        }
        result = web_server.run() => {
            if let Err(e) = result {
                log::error!("Web server error: {}", e);
            }
        }
    }

    Ok(())
}
