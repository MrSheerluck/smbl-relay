use serde_json::{Value, json};
use std::{collections::HashMap, env, error::Error, fs, path::Path, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const LOCAL_DATABASE_ID: &str = "00000000-0000-0000-0000-000000000000";

fn setting(name: &str, local: &HashMap<String, String>) -> Result<String> {
    let value = env::var(name).ok().or_else(|| local.get(name).cloned());
    value.filter(|v| !v.trim().is_empty()).ok_or_else(|| {
        format!("Set {name} in your environment or .env.deploy before deploying.").into()
    })
}

fn validate_ids(account: &str, database: &str) -> Result<()> {
    if account.len() != 32 || !account.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(
            "CLOUDFLARE_ACCOUNT_ID must be your 32-character Cloudflare account ID.".into(),
        );
    }
    if database.len() != 36
        || !database.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
        || database == LOCAL_DATABASE_ID
    {
        return Err(
            "CLOUDFLARE_D1_DATABASE_ID must be your D1 database UUID, not the local placeholder."
                .into(),
        );
    }
    Ok(())
}

fn deployment_config(root: &Path, target: &str, database: &str, domain: &str) -> Result<Value> {
    let source = match target {
        "workers" => root.join("wrangler.jsonc"),
        "pages" => root.join("pages/wrangler.jsonc"),
        _ => return Err("Use workers or pages as the deployment target.".into()),
    };
    let mut config: Value = serde_json::from_str(&fs::read_to_string(&source)?)?;
    let base = source.parent().unwrap();
    // Generated configs live outside the project root, so keep asset paths absolute.
    for key in ["main", "pages_build_output_dir"] {
        if let Some(path) = config[key].as_str() {
            config[key] = json!(base.join(path));
        }
    }
    if let Some(path) = config["assets"]["directory"].as_str() {
        config["assets"]["directory"] = json!(base.join(path));
    }
    let binding = &mut config["d1_databases"][0];
    binding["database_id"] = json!(database);
    let migrations = binding["migrations_dir"]
        .as_str()
        .ok_or("Missing migrations_dir")?;
    binding["migrations_dir"] = json!(base.join(migrations));
    config.as_object_mut().unwrap().remove("account_id");
    config.as_object_mut().unwrap().remove("routes");
    if target == "workers" {
        config["workers_dev"] = json!(domain.is_empty());
        if !domain.is_empty() {
            let parsed = url::Url::parse(&format!("https://{domain}"))?;
            if parsed.host_str() != Some(domain) || parsed.path() != "/" || parsed.port().is_some()
            {
                return Err(
                    "RELAY_DOMAIN must be a hostname, without a URL scheme or path.".into(),
                );
            }
            config["routes"] = json!([{ "pattern": domain, "custom_domain": true }]);
        }
    }
    Ok(config)
}

fn run() -> Result<i32> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut args = env::args().skip(1);
    let target = args
        .next()
        .ok_or("Usage: cloudflare <workers|pages> <wrangler arguments>")?;
    let args: Vec<_> = args.collect();
    if args.is_empty()
        || args.iter().any(|a| {
            [
                "--config",
                "-c",
                "--cwd",
                "--env",
                "-e",
                "--experimental-new-config",
            ]
            .iter()
            .any(|flag| a == flag || a.starts_with(&format!("{flag}=")))
        })
    {
        return Err(
            "Supply a Wrangler command without overriding the deployment config or environment."
                .into(),
        );
    }
    let env_file = root.join(".env.deploy");
    let local = if env_file.exists() {
        dotenvy::from_path_iter(env_file)?.collect::<std::result::Result<HashMap<_, _>, _>>()?
    } else {
        HashMap::new()
    };
    let account = setting("CLOUDFLARE_ACCOUNT_ID", &local)?;
    let database = setting("CLOUDFLARE_D1_DATABASE_ID", &local)?;
    validate_ids(&account, &database)?;
    let domain = env::var("RELAY_DOMAIN")
        .ok()
        .or_else(|| local.get("RELAY_DOMAIN").cloned())
        .unwrap_or_default();
    let config = deployment_config(root, &target, &database, &domain)?;
    let generated = root.join(".wrangler/deploy").join(&target);
    fs::create_dir_all(&generated)?;
    let config_path = generated.join("wrangler.jsonc");
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

    let mut command = Command::new(root.join("node_modules/.bin/wrangler"));
    command
        .current_dir(root)
        .env("CLOUDFLARE_ACCOUNT_ID", account)
        .args(&args);
    if target == "pages" {
        command.current_dir(&generated);
        // Pages dev resolves its inner Worker config beside the built assets.
        // Pass the selected D1 binding explicitly to avoid the local template.
        if args.starts_with(&["pages".into(), "dev".into()]) {
            command.arg("--d1").arg(format!("DB={database}"));
        }
        if args.starts_with(&["pages".into(), "deploy".into()]) {
            command.arg("--project-name").arg(
                config["name"]
                    .as_str()
                    .ok_or("Missing Pages project name")?,
            );
        }
    } else {
        command.arg("--config").arg(config_path);
    }
    Ok(command.status()?.code().unwrap_or(1))
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("Deployment stopped: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ACCOUNT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DATABASE: &str = "12345678-1234-1234-1234-123456789abc";

    #[test]
    fn rejects_missing_or_placeholder_identifiers() {
        assert!(validate_ids("", DATABASE).is_err());
        assert!(validate_ids(ACCOUNT, "").is_err());
        assert!(validate_ids(ACCOUNT, LOCAL_DATABASE_ID).is_err());
        assert!(validate_ids(ACCOUNT, "your-database-id").is_err());
        assert!(validate_ids(ACCOUNT, DATABASE).is_ok());
    }

    #[test]
    fn both_targets_use_supplied_database_and_resolvable_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for target in ["workers", "pages"] {
            let config = deployment_config(root, target, DATABASE, "").unwrap();
            assert_eq!(config["d1_databases"][0]["database_id"], DATABASE);
            assert!(config.get("account_id").is_none());
            assert!(config.get("routes").is_none());
            assert!(
                Path::new(
                    config["d1_databases"][0]["migrations_dir"]
                        .as_str()
                        .unwrap()
                )
                .is_dir()
            );
            let asset_path = if target == "workers" {
                &config["assets"]["directory"]
            } else {
                &config["pages_build_output_dir"]
            };
            assert!(Path::new(asset_path.as_str().unwrap()).is_absolute());
        }
    }

    #[test]
    fn custom_domain_must_be_explicit_and_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let config = deployment_config(root, "workers", DATABASE, "relay.example.com").unwrap();
        assert_eq!(config["routes"][0]["pattern"], "relay.example.com");
        assert_eq!(config["workers_dev"], false);
        assert_eq!(
            deployment_config(root, "workers", DATABASE, "").unwrap()["workers_dev"],
            true
        );
        assert!(deployment_config(root, "workers", DATABASE, "https://relay.example.com").is_err());
    }
}
